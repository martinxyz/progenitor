#!/usr/bin/env python3
import numpy as np

from progenitor.progenitor import Tumblers

def sigmoid(x):
  return 1 / (1 + np.exp(-x))

def evaluate(x):

    prob = sigmoid(x[0])
    iterations = 500

    score = 0
    for _ in range(iterations):
        sim = Tumblers(prob)
        sim.steps(50)
        score += sim.avg_visited()

    cost = - score / iterations

    print(f'{cost:.6f} for p={prob:.6f} - x={x[0]:.6f}')
    return cost

# l = []
# for logit in np.linspace(-3, +3, 100):
#     l.append(evaluate([logit]))
# --> Optimum is around x=-1.6, cost=-0.509, p=0.158

# This (BFGS) fails to optimize at all, not sure why.
# from scipy.optimize import minimize
# minimize(evaluate, x0=[0], options={
#     'maxiter': 10000,
# })

# Now here is a good algo.
import cma
es = cma.CMAEvolutionStrategy(2 * [0], 0.5)
es.optimize(evaluate)
