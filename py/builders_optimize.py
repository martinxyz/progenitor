#!/usr/bin/env python3
import numpy as np
import os
import cma
import ray
from ray import tune

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
def evaluate(x, episodes=10_000, stats=False):
    Builders = progenitor.mod.Builders

    score = 0
    for i in range(episodes):
        sim = Builders(x)
        sim.steps(1000)
        if stats and i == 0:
            sim.print_stats()
        score += sim.score()
        # score = max(score, sim.score())

    cost = - score / episodes
    # cost = - score

    # print(f'{cost:.6f} for p={probs} - x={x[0]:.6f}')
    return cost

def save_array(filename, data):
    with open(os.path.join('output', filename), 'w') as f:
        np.savetxt(f, data)

def train(config):
    N = progenitor.mod.Builders.param_count

    episodes_budget = 1_000_000
    es = cma.CMAEvolutionStrategy(N * [0], 1.0, {
        'popsize': config["popsize"],
        'maxfevals': episodes_budget / config["episodes_per_eval"],
    })

    iteration = 0
    evaluation = 0
    next_report_at = 0
    while not es.stop():
        solutions = es.ask()
        # print('asked to evaluate', len(solutions), 'solutions')

        futures = [evaluate.remote(x, config["episodes_per_eval"]) for x in solutions]
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
            es.disp()
            next_report_at += 100_000
            # run an evaluation that is independent of hyperparams
            # note: technically, we don't need to wait for the result - could do this in parallel?
            mean_cost = ray.get(evaluate.remote(es.result.xfavorite, stats=True))

            tune.report(mean_accuracy=-mean_cost)
            # save_array(f'xfavorite-eval%07d.dat' % evaluation, es.result.xfavorite)
            # save_array(f'stds-eval%07d.dat' % evaluation, es.result.stds)


def main_tune():
    search_space = {
        "popsize": tune.lograndint(7, 150),
        "episodes_per_eval": tune.lograndint(5, 150)
    }
    resources_per_trial=tune.PlacementGroupFactory(
        [{'CPU': 2.0}] + [{'CPU': 1.0}] * 4
    )

    tuner = tune.Tuner(
        tune.with_resources(train, resources_per_trial),
        param_space=search_space,
        tune_config=tune.TuneConfig(num_samples=20)
    )

    tuner.fit()


def main_simple():
    train(config = {
        # my initial params:
        # "popsize": 18,
        # "episodes_per_eval": 400,
        # good (or at least "not bad") values, after tuning results:
        "popsize": 10,
        "episodes_per_eval": 75,
    })

if __name__ == '__main__':
    # main_simple()
    main_tune()
