#!/usr/bin/env python3
import numpy as np
import os
import cma
import ray
from ray import tune
from ray.tune.schedulers import ASHAScheduler

import progenitor
print(progenitor.__file__)

# We cannot import progenitor.mod.Builders and then use it in the @ray.remote,
# apparently (I think the @ray.remote object fails to serialize). The attempts
# below do not help: (at least not when running local-only)
#
# ray.init(runtime_env={"py_modules": [progenitor]})
# ray.init(runtime_env={'py_modules': ['crates/python/progenitor']})
# ray.init(runtime_env={"py_modules": ["/home/martin/code/progenitor/target/wheels/progenitor-0.0.0-cp310-cp310-linux_x86_64.whl"]})
# ray.init(runtime_env={"pip": ["/home/martin/code/progenitor/target/wheels/progenitor-0.0.0-cp310-cp310-linux_x86_64.whl"]})
# ray.init()

@ray.remote
def evaluate(x, config, episodes, stats=False):
    Builders = progenitor.mod.Builders

    hyperparams = {
        "init_fac": config["init_fac"],
        "bias_fac": config["bias_fac"]
    }

    score = 0
    for i in range(episodes):
        sim = Builders(x, **hyperparams)
        sim.steps(1000)
        if stats and i == 0:
            sim.print_stats()
            # report those via tensorboard? calculate entropy of actions, too?
            # (Just return a metrics dict? Averaged/Stats?)
        score += sim.score()
        # score = max(score, sim.score())

    cost = - score / episodes

    # print(f'{cost:.6f} for p={probs} - x={x[0]:.6f}')
    return cost

def save_array(filename, data):
    with open(os.path.join('output', filename), 'w') as f:
        np.savetxt(f, data)

def train(config, tuning=True):
    N = progenitor.mod.Builders.param_count

    # episodes_budget = 20_000_000
    es = cma.CMAEvolutionStrategy(N * [0], 1.0, {
        'popsize': config["popsize"],
        # 'maxfevals': episodes_budget / config["episodes_per_eval"],
    })

    iteration = 0
    evaluation = 0
    next_report_at = 0
    while not es.stop():
        solutions = es.ask()
        futures = [evaluate.remote(x, config, episodes=config["episodes_per_eval"]) for x in solutions]
        # costs = [evaluate(x) for x in solutions]
        costs = ray.get(futures)

        evaluation += len(solutions)
        iteration += 1
        episodes = evaluation * config["episodes_per_eval"]
        # print(f'evaluation {evaluation} ({episodes} epoisodes)')
        # print('computed costs:', list(reversed(sorted(costs))))
        # print('avg costs:', np.mean(costs))

        es.tell(solutions, costs)

        while episodes > next_report_at:
            print()
            print(f'report at {episodes}: (past {next_report_at})')
            next_report_at += 10_000  # makes the tensorboard x-axis ("steps") more useful, independent of hyperparams
            if tuning:
                # run an evaluation that is independent of hyperparams
                # note: technically, we don't need to wait for the result - could do this in parallel?
                mean_cost = ray.get(evaluate.remote(es.result.xfavorite, config, episodes=500, stats=True))

                tune.report(score=-mean_cost, total_episodes=episodes) #, foobar=5.5
                # should we also report a "iterations=iteration"? (is it special somehow?)
            else:
                mean_cost = ray.get(evaluate.remote(es.result.xfavorite, config, episodes=500, stats=False))
                print(f'score = {-mean_cost:.3f}')
                es.disp()
                save_array(f'xfavorite-eval%07d.dat' % evaluation, es.result.xfavorite)
                save_array(f'stds-eval%07d.dat' % evaluation, es.result.stds)


def main_tune():
    search_space = {
        "popsize": tune.lograndint(7, 300),
        "episodes_per_eval": tune.lograndint(3, 300),
        # "popsize": 76,
        # "episodes_per_eval": 47,
        # "init_fac": tune.loguniform(0.20, 3.5),  # plausible good range: 0.25..3.0
        # "bias_fac": tune.loguniform(0.005, 0.25),  # plausible good range: 0.0..0.4
        "init_fac": tune.loguniform(0.05, 8.0),  # plausible good range: 0.25..3.0
        "bias_fac": tune.loguniform(0.005, 2.0),  # plausible good range: 0.0..0.4
    }
    tune_config = tune.TuneConfig(
        # num_samples=-1,
        num_samples=300,  # "runs" or "restarts"
        metric='score',
        mode='max',
        scheduler=ASHAScheduler(
            time_attr='total_episodes',
            grace_period=30_000,  # training "time" allowed for every "sample" (run)
            max_t=1_000_000,      # training "time" allowed for the best run(s)
            reduction_factor=3,
            brackets=1,
        )
    )
    # resources_per_trial={'cpu': 2, 'gpu': 0}
    resources_per_trial=tune.PlacementGroupFactory(
        [{'CPU': 1.0}] + [{'CPU': 1.0}] * 1
    )
    tuner = tune.Tuner(
        tune.with_resources(train, resources_per_trial),
        param_space=search_space,
        tune_config=tune_config
    )

    results = tuner.fit()

    df = results.get_dataframe()
    df = df.sort_values('score', ascending=False)
    df.to_csv('output/tuner_result.csv')
    # ...or just start tensorboard in ~/ray_results/

    print("Config of best run:", results.get_best_result().config)

def main_simple():
    train(config = {
        # My original params:
        # "popsize": 18,            <-- okay (CMA-ES default)
        # "episodes_per_eval": 400, <-- much too high
        # "init_fac": 1.0,          <-- great
        # "bias_fac": 0.1,          <-- great
        # Tuning result:
        "popsize": 32,
        "episodes_per_eval": 14,
        "init_fac": 1.3,
        "bias_fac": 0.08,
    }, tuning=False)

if __name__ == '__main__':
    main_simple()
    # main_tune()
