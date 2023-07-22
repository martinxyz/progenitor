#!/usr/bin/env python3
import numpy as np
import os
import cma
import ray
import sys
from ray import tune
from ray.tune.schedulers import ASHAScheduler
from ray.air import session, RunConfig
import random
# from ray.air.checkpoint import Checkpoint

import progenitor
print(progenitor.__file__)
progenitor_version_expected = 1
assert progenitor.mod.version_check == progenitor_version_expected

# We cannot import progenitor.mod.Builders and then use it in the @ray.remote,
# apparently (I think the @ray.remote object fails to serialize). The attempts
# below do not help: (at least not when running local-only)
#
# ray.init(runtime_env={"py_modules": [progenitor]})
# ray.init(runtime_env={'py_modules': ['crates/python/progenitor']})
# ray.init(runtime_env={"py_modules": ["/home/martin/code/progenitor/target/wheels/progenitor-0.0.0-cp310-cp310-linux_x86_64.whl"]})
# ray.init(runtime_env={"pip": ["/home/martin/code/progenitor/target/wheels/progenitor-0.0.0-cp310-cp310-linux_x86_64.whl"]})
# ray.init()

def get_params(x, config):
    Params = progenitor.mod.Params
    hyperparams = {
        "init_fac": config["init_fac"],
        "bias_fac": config["bias_fac"],
        "memory_clamp": config["memory_clamp"],
        "memory_halftime": config["memory_halftime"],
        "actions_scale": config["actions_scale"],
    }
    return Params(x, **hyperparams)

@ray.remote
def evaluate(x, config, episodes, stats=False, seed=None):
    Builders = progenitor.mod.Builders
    assert progenitor.mod.version_check == progenitor_version_expected
    params = get_params(x, config)

    score = 0
    for i in range(episodes):
        if seed is not None:
            seed += 1
        sim = Builders(params, seed)
        sim.steps(3000)
        if stats and i == 0:
            sim.print_stats()
            # report those via tensorboard? calculate entropy of actions, too?
            # (Just return a metrics dict? Averaged/Stats?)
        score += sim.hoarding_score()

    cost = - score / episodes

    # print(f'{cost:.6f} for p={probs} - x={x[0]:.6f}')
    return cost

def save_array(filename, data):
    with open(os.path.join('output', filename), 'w') as f:
        np.savetxt(f, data)

def train(config, tuning=True):
    N = progenitor.mod.Builders.param_count

    x0 = N * [0]
    # x0 = np.load('/home/martin/ray_results/builders-restarted/train_c6318_00154_154_memory_halftime=1.6286,popsize=167.5861_2023-07-21_00-28-58/xfavorite-14995264.npy')
    sigma0 = 1.0

    # episodes_budget = 20_000_000
    es = cma.CMAEvolutionStrategy(x0, sigma0, {
        'popsize': config["popsize"],
        # 'maxfevals': episodes_budget / config["episodes_per_eval"],
    })

    iteration = 0
    evaluation = 0
    next_report_at = 0
    while not es.stop():
        solutions = es.ask()
        seed = random.randrange(1_000_000_000)
        futures = [evaluate.remote(x, config, episodes=config["episodes_per_eval"], seed=seed) for x in solutions]
        costs = ray.get(futures)

        evaluation += len(solutions)
        iteration += 1
        episodes = evaluation * config["episodes_per_eval"]

        es.tell(solutions, costs)

        while episodes > next_report_at:
            print()
            print(f'report at {episodes}: (past {next_report_at})')
            next_report_at += 10_000  # makes the tensorboard x-axis ("steps") more useful, independent of hyperparams

            # FIXME: this is hurting parallelism
            #        especially with large episodes_per_eval
            #        (no need waiting for this evaulation before starting the next generation)
            mean_cost = ray.get(evaluate.remote(es.result.xfavorite, config, episodes=500, stats=False))
            fn_prefix = 'output/'
            if tuning:
                fn_prefix = ''
                # cp = Checkpoint()
                # run an evaluation that is independent of hyperparams
                # note: technically, we don't need to wait for the result - could do this in parallel?

                session.report(metrics={
                    'score': -mean_cost,
                    'total_episodes': episodes,
                    # 'foobar': 5,
                })
                # }, checkpoint=Checkpoint())
                # should we also report a "iterations=iteration"? (is it special somehow?)

            else:
                print(f'score = {-mean_cost:.3f}')
                es.disp()


            np.save(f'{fn_prefix}xfavorite-{episodes}.npy', es.result.xfavorite)
            params_favourite = get_params(es.result.xfavorite, config)
            with open(f'{fn_prefix}xfavorite-{episodes}.params.bin', 'wb') as f:
                f.write(params_favourite.serialize())

            # save_array(f'xfavorite-eval%07d.dat' % evaluation, es.result.xfavorite)
            # save_array(f'stds-eval%07d.dat' % evaluation, es.result.stds)


def main_tune():
    run_name = 'builders-' + sys.argv[1]

    search_space = {
        "popsize": 200,  # plausible range: 50..?200? (large values have )
        # high popsize: lowers the chance to get a good result, but the few good ones get better
        #               (they fail because we stop them early...?)
        "episodes_per_eval": 16, # ("denoising" effect ~= popsize*episodes_per_eval)
        "init_fac": tune.loguniform(0.2, 3.0),  # (clear effect) plausible range: 0.2..1.5
        "bias_fac": 0.1, # plausible range: 0.01..0.9 (0.1 is fine.)
        "memory_clamp": tune.loguniform(0.8, 200.0),  # plausible range: 1.0..?>100?
        "memory_halftime": tune.loguniform(1.5, 100.0), # plausible range: 1.5..?~100?
        "actions_scale": tune.loguniform(1.5, 30.),  # plausible range: 2.0..20
        # "sigma0": tune.loguniform(0.2, 5.0),
    }
    max_t = 15_000_000
    tune_config = tune.TuneConfig(
        # num_samples=-1,
        num_samples=100,  # "runs" or "restarts"
        metric='score',
        mode='max',
        scheduler=ASHAScheduler(
            time_attr='total_episodes',
            grace_period=max_t // 100,  # training "time" allowed for every "sample" (run)
            max_t=max_t,      # training "time" allowed for the best run(s)
            reduction_factor=3,
            brackets=1,
        )
    )
    # resources_per_trial={'cpu': 1, 'gpu': 0}
    resources_per_trial=tune.PlacementGroupFactory(
        [{'CPU': 0.0}] + [{'CPU': 1.0}] * 8
        #-------------   ------------------
        # train() task,      ^ evaluate() tasks spawned by train().
        # does work once       They can use more CPUs, how many depends
        # per generation.      on the population_size (a hyperparam).
        # (short burst)        But those we reserve (but one) will idle during
        #                      each generation-evaluation. If we reserve
        #                      just one, utilization will be nearly 100% at the
        #                      beginning but the last few runs remaining will
        #                      use a single CPU each, when they could parallelize.
        #                      ...any good solution for that? (Fractional CPUs?)
        #
        # https://docs.ray.io/en/latest/tune/faq.html#how-do-i-set-resources
        # https://docs.ray.io/en/latest/ray-core/scheduling/placement-group.html
    )
    tuner = tune.Tuner(
        tune.with_resources(train, resources_per_trial),
        run_config=RunConfig(name=run_name),  # + timestamp?
        param_space=search_space,
        tune_config=tune_config
    )

    analysis = tuner.fit()
    # tune.run(keep_checkpoints_num=1, checkpoint_score_attr="accuracy")
    # best_trial = analysis.get_best_trial(metric="accuracy", mode="max", scope="all")
    # best_checkpoint = analysis.get_best_checkpoint(best_trial, metric="accuracy")

    df = analysis.get_dataframe()
    df = df.sort_values('score', ascending=False)
    df.to_csv('output/tuner_result.csv')
    # ...or just start tensorboard in ~/ray_results/

    print("Config of best run:", analysis.get_best_result().config)

def main_simple():
    train(config = {
        "popsize": 32,
        "episodes_per_eval": 16,
        "init_fac": 1.3,
        "bias_fac": 0.08,
        "memory_clamp": 10.0,
        "memory_halftime": 50.0,
        "actions_scale": 1.0,
    }, tuning=False)

if __name__ == '__main__':
    # main_simple()
    main_tune()
