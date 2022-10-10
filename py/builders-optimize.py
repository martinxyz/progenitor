#!/usr/bin/env python3
import numpy as np
import cma

from progenitor.progenitor import Builders

def evaluate(x, stats=False):
    iterations = 400

    score = 0
    for repetition in range(iterations):
        sim = Builders(x)
        sim.steps(1000)
        if stats and repetition == 0:
            sim.print_stats()
        score += sim.score()
        # score = max(score, sim.score())

    cost = - score / iterations
    # cost = - score

    # print(f'{cost:.6f} for p={probs} - x={x[0]:.6f}')
    return cost

N = Builders.param_count
es = cma.CMAEvolutionStrategy(N * [0], 1.0, {
    'maxfevals': 50_000
})
res = es.optimize(evaluate).result
print(res)
# mean solution, presumably better with noise
print(res.xfavorite)
