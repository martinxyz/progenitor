#!/usr/bin/env python3
import numpy as np
import random

from progenitor.progenitor import Tumblers

def sigmoid(x):
  return 1 / (1 + np.exp(-x))

# def evaluate(x):

#     probs = sigmoid(x)
#     iterations = 500

#     score = 0
#     for _ in range(iterations):
#         sim = Tumblers(probs)
#         sim.steps(5)
#         score += sim.avg_visited()

#     cost = - score / iterations

#     print(f'{cost:.6f} for p={probs:.3f}')
#     return cost

# Now here is a good algo.
# import cma
# es = cma.CMAEvolutionStrategy(2 * [0], 0.5)
# es.optimize(evaluate)

# just random search for a good seed
evals = 500000
seed0 = random.randrange(1<<32)
for seed in range(seed0, seed0 + evals):
    sim = Tumblers(seed)
    sim.steps(4)
    loss = sim.loss()
    sim.steps(1)
    loss += sim.loss()
    if loss < 1000:
        sim.steps(800)
        loss *= 1 - sim.avg_visited()
        loss *= 20 + (sim.count_bots() - 5) ** 2
        print(loss, seed)
