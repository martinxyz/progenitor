#!/usr/bin/env python3
import numpy as np
import random

from ribs.archives import GridArchive
from ribs.emitters import GaussianEmitter
from ribs.schedulers import Scheduler

from progenitor.progenitor import Tumblers

def sigmoid(x):
  return 1 / (1 + np.exp(-x))

def evaluate(x):

    prob = sigmoid(x[0])
    # iterations = 500
    iterations = 50

    score = 0
    bc1 = 0
    for _ in range(iterations):
        sim = Tumblers(prob)
        sim.steps(5)
        bc1 += sim.avg_visited()
        sim.steps(50)
        score += sim.avg_visited()

    # XXX WIP - this doesn't make sense yet
    cost = - score / iterations
    bc1 = bc1 / iterations

    print(f'{cost:.6f} for p={prob:.6f} - x={x[0]:.6f}')
    return (cost, bc1, random.random())


param_count = 2
population_size = 500
evaluations = 10_000

archive = GridArchive(2, [20, 20], [(-1, 1), (-1, 1)])
# emitters = [ImprovementEmitter(archive, [0.0] * param_count, 1.0)]
emitters = [GaussianEmitter(
    archive,
    [0.5] * param_count, 0.2,
    bounds=param_count * [(0, 1)],
    batch_size=population_size,
)]
optimizer = Scheduler(archive, emitters)

for itr in range(evaluations // population_size):
    solutions = optimizer.ask()

    # bcs = np.zeros((len(solutions), 3))
    bcs = np.array([evaluate(x) for x in solutions])
    # evaluate(solutions, bcs)

    objectives, bcs = bcs[:, 0], bcs[:, 1:]

    optimizer.tell(objectives, bcs)


import matplotlib.pyplot as plt
from ribs.visualize import grid_archive_heatmap

grid_archive_heatmap(archive)
plt.show()
