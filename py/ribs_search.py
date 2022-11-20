#!/usr/bin/env python3
import numpy as np
import random
import ray
import pickle

from ribs.archives import GridArchive
from ribs.emitters import GaussianEmitter, EvolutionStrategyEmitter
from ribs.schedulers import Scheduler

import progenitor

@ray.remote
def evaluate(x):
    Builders = progenitor.mod.Builders
    episodes=100

    hyperparams = {
        "init_fac": 1.0, #config["init_fac"],
        "bias_fac": 0.1, #config["bias_fac"]
    }

    score = 0.0
    bc1 = 0.0
    bc2 = 0.0
    for _ in range(episodes):
        sim = Builders(x, **hyperparams)
        sim.steps(20)
        bc1 += sim.avg_visited()
        sim.steps(980)
        bc2 += np.log(sim.encounters() + 100)
        score += sim.score()

    score = score / episodes
    bc1 = bc1 / episodes
    bc2 = bc2 / episodes

    return (score, bc1, bc2)



param_count = progenitor.mod.Builders.param_count
# population_size = 500
# evaluations = 10_000
population_size = 120
# evaluations = 500
evaluations = 1000_000

archive = GridArchive(param_count, [40, 100], [(0, 0.08), (3, 10)])
# emitters = [EvolutionStrategyEmitter(
#     archive,
#     x0 = [0.0] * param_count,
#     sigma0 = 1.0,
#     ranker='2imp' if i < 5 else 'rd',
#     # batch_size=50,
#     batch_size=population_size,
#     # restart_rule='basic',
#     # selection_rule='mu',
# ) for i in range(10)]
emitters = [GaussianEmitter(
    archive,
    x0 = [0.0] * param_count,
    sigma = 0.5,
    batch_size=population_size,
)]
optimizer = Scheduler(archive, emitters)

for itr in range(evaluations // population_size):
    solutions = optimizer.ask()

    # bcs = np.array([evaluate(x) for x in solutions])

    futures = [evaluate.remote(x) for x in solutions]
    bcs = np.array(ray.get(futures))

    # print('bcs[0]', bcs[0])
    print('asked to evaluate', len(solutions), solutions)
    # evaluate(solutions, bcs)
    objectives, bcs = bcs[:, 0], bcs[:, 1:]

    optimizer.tell(objectives, bcs)

    best = archive.best_elite
    assert best is not None
    print(archive.stats)

    if itr % 20 == 0:
        fn = 'archive_autosave.pik'
        print('saving to', fn)
        with open(fn, 'wb') as f:
            pickle.dump(archive, f)

    print(f'({itr*population_size / evaluations * 100:.01f}%) best: {best.objective:.3f}, size: {len(archive)}')


import matplotlib.pyplot as plt
from ribs.visualize import grid_archive_heatmap

grid_archive_heatmap(archive)
plt.show()
