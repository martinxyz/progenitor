#!/usr/bin/env python3
import numpy as np
import cma

from progenitor.progenitor import Builders

def softmax(x):
    e_x = np.exp(x - np.max(x))
    return e_x / e_x.sum(axis=0)

def evaluate(x):

    probs = softmax(x)
    iterations = 500

    score = 0
    for _ in range(iterations):
        sim = Builders(probs)
        sim.steps(50)
        score += sim.score()

    cost = - score / iterations

    print(f'{cost:.6f} for p={probs} - x={x[0]:.6f}')
    return cost

es = cma.CMAEvolutionStrategy(4 * [0], 10.0)
es.optimize(evaluate)
