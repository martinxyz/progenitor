#!/usr/bin/env python3
import numpy as np
import os
import shutil
import glob
import cma
import cmaes
import ray
import sys
from ray import tune
from ray.tune.schedulers import ASHAScheduler
from ray.air import session, RunConfig
import random
# from ray.air.checkpoint import Checkpoint

import progenitor
version_check = 11
assert progenitor.mod.version_check == version_check, progenitor.__file__
# We cannot import progenitor.mod.Builders and then use it in the @ray.remote,
# apparently. (I think the @ray.remote object fails to serialize.)
# (Is this still true when starting with the submit-job.sh script?)

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
    assert progenitor.mod.version_check == version_check, f'wrong module version: {Builders.__file__}'
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
    print('param_count:', N)

    x0 = N * [0]
    # x0 = np.load('checkpoint-score-196.npy')
    sigma0 = 1.0

    # episodes_budget = 20_000_000
    if config["optimizer"] == 'cmaes-1':
        es = cma.CMAEvolutionStrategy(x0, sigma0, {
            'popsize': config["popsize"],
            # 'maxfevals': episodes_budget / config["episodes_per_eval"],
        })
        es.should_stop = es.stop
    elif config["optimizer"] == 'cmaes-2':
        # This gives slightly different (more variance? and slightly worse) results compared to 'cmaes-2'.
        # There must be some subtle difference...? could be the reporting (xfavorite vs es._mean)
        es = cmaes.CMA(mean=x0, sigma=sigma0, population_size=config["popsize"])
    elif config["optimizer"] == 'cmaes-2-lr':
        # This one fails horribly. Looks as if it optimizes in the wrong direction.
        # (May have been caused by my "epoch" seeding strategy.)
        es = cmaes.CMA(mean=x0, sigma=sigma0, population_size=config["popsize"], lr_adapt=True)
    elif config["optimizer"] == 'sep-cmaes':
        es = cmaes.SepCMA(mean=x0, sigma=sigma0, population_size=config["popsize"])
    else:
        raise NotImplementedError(config["optimizer"])


    iteration = 0
    evaluation = 0
    next_report_at = 0

    pending_report = None
    pending_report_at = None

    while not es.should_stop():
        seed = random.randrange(1_000_000_000) if config['seeding'] == 'epoch' else None
        if config["optimizer"] == 'cmaes-1':
            solutions = es.ask()
            futures = [evaluate.remote(x, config, episodes=config["episodes_per_eval"], seed=seed) for x in solutions]
        else:
            solutions = []
            futures = []
            for _ in range(es.population_size):
                # This is a performance bottleneck for cmaes.CMA. Time is spent
                # in numpy matrix operations; "popsize" size was <= 120.
                # Creating tasks as samples arrive helps a bit, I guess?
                x = es.ask()
                solutions.append(x)
                futures.append(evaluate.remote(x, config, episodes=config["episodes_per_eval"], seed=seed))
        costs = ray.get(futures)

        evaluation += len(solutions)
        iteration += 1
        episodes = evaluation * config["episodes_per_eval"]

        if config["optimizer"] == 'cmaes-1':
            es.tell(solutions, costs)
        else:
            es.tell(list(zip(solutions, costs)))

        def emit_report(r_episodes, r_mean_cost):
            if tuning:
                session.report(metrics={'score': -r_mean_cost, 'total_episodes': r_episodes})
            else:
                print(f'score = {-r_mean_cost:.3f}')

        while episodes > next_report_at:
            print()
            print(f'report at {episodes}: (past {next_report_at})')
            next_report_at += 100_000  # makes the tensorboard x-axis ("steps") more useful, independent of hyperparams

            if pending_report:
                emit_report(pending_report_at, ray.get(pending_report))
            r_x = es.result.xfavorite if config["optimizer"] == 'cmaes-1' else es._mean
            pending_report = evaluate.remote(r_x, config, episodes=500)
            pending_report_at = episodes

            if not tuning and config["optimizer"] == 'cmaes-1':
                es.disp()

            fn_prefix = '' if tuning else 'output/'
            if tuning:
                # only keep latest params per run
                os.makedirs('old', exist_ok=True)
                for fn in os.listdir('old'):
                    os.remove(f'old/{fn}')
                for fn in glob.glob(fn_prefix + 'xfavorite-*'):
                    os.rename(fn, f'old/{fn}')
            else:
                # ugh, those files end up somewhere in the /tmp/ray/ workdir...
                os.makedirs(fn_prefix, exist_ok=True)

            np.save(f'{fn_prefix}xfavorite-{episodes}.npy', r_x)
            params_favourite = get_params(r_x, config)
            with open(f'{fn_prefix}xfavorite-{episodes}.params.bin', 'wb') as f:
                f.write(params_favourite.serialize())

            # save_array(f'xfavorite-eval%07d.dat' % evaluation, es.result.xfavorite)
            # save_array(f'stds-eval%07d.dat' % evaluation, es.result.stds)


def main_tune():
    run_name = 'builders-' + sys.argv[1]

    search_space = {
        "popsize": 200,  # plausible range: 50..?200?
        # "popsize": tune.lograndint(15, 120),  # plausible range: 50..?200?
        # high popsize: lowers the chance to get a good result, but the few good ones get better
        #               (maybe they fail only because we stop them early...?)
        "episodes_per_eval": 60, # ("denoising" effect ~= popsize*episodes_per_eval)
        "init_fac": tune.loguniform(0.1, 1.5),  # (clear effect) plausible range: 0.2..1.3
        "bias_fac": 0.1, # plausible range: 0.01..0.9 (0.1 is fine.)
        # "memory_clamp": tune.loguniform(0.8, 200.0),  # plausible range: 1.0..?>100?
        "memory_clamp": 50.,  # plausible range: 1.0..?>100?
        "memory_halftime": tune.loguniform(1.5, 100.0), # plausible range: 1.5..?~100?
        "actions_scale": tune.loguniform(2.0, 20.),  # plausible range: 2.0..20
        # "optimizer": tune.choice(["cmaes-1", "cmaes-2"] + 3*["sep-cmaes"]),
        "optimizer": 'sep-cmaes',
        # "seeding": tune.choice(["random", "epoch"]),  # epoch-seeding seems the better idea; some weak evidence that it helps
        "seeding": "epoch",
    }
    max_t = 30_000_000
    tune_config = tune.TuneConfig(
        # num_samples=-1,
        num_samples=150,  # "runs" or "restarts"
        metric='score',
        mode='max',
        scheduler=ASHAScheduler(
            time_attr='total_episodes',
            grace_period=max_t // 30,  # training "time" allowed for every "sample" (run)
            max_t=max_t,      # training "time" allowed for the best run(s)
            reduction_factor=3,
            brackets=1,
        )
    )
    # resources_per_trial={'cpu': 1, 'gpu': 0}
    resources_per_trial=tune.PlacementGroupFactory(
        # [{'CPU': 0.0}] + [{'CPU': 1.0}] * 64  # for a single run with popsize=64*16
        [{'CPU': 0.0}] + [{'CPU': 1.0}] * 16
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
        tune.with_resources(
            train,
            resources=resources_per_trial
            # resources=lambda config: {"GPU": 1} if config["use_gpu"] else {"GPU": 0},
        ),
        run_config=RunConfig(name=run_name),  # + timestamp?
        param_space=search_space,
        tune_config=tune_config
    )

    analysis = tuner.fit()
    # tune.run(keep_checkpoints_num=1, checkpoint_score_attr="accuracy")
    # best_trial = analysis.get_best_trial(metric="accuracy", mode="max", scope="all")
    # best_checkpoint = analysis.get_best_checkpoint(best_trial, metric="accuracy")

    # df = analysis.get_dataframe()
    # df = df.sort_values('score', ascending=False)  # keyerror: score
    # df.to_csv('output/tuner_result.csv')
    # ...or just start tensorboard in ~/ray_results/

    print("Config of best run:", analysis.get_best_result().config)

def main_simple():
    train(config = {
        "actions_scale": 6,
        "bias_fac": 0.1,
        "episodes_per_eval": 300,
        "init_fac": 0.2,
        "memory_clamp": 50,
        "memory_halftime": 6,
        "optimizer": "sep-cmaes",
        "popsize": 197,
        "seeding": "epoch"
    }, tuning=False)

if __name__ == '__main__':
    # main_simple()
    main_tune()
