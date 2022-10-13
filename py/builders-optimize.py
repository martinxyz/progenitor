#!/usr/bin/env python3
import numpy as np
import os
import cma
import ray

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
def evaluate(x, stats=False):
    Builders = progenitor.mod.Builders

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

def save_array(filename, data):
    with open(os.path.join('output', filename), 'w') as f:
        np.savetxt(f, data)

def main():
    N = progenitor.mod.Builders.param_count
    es = cma.CMAEvolutionStrategy(N * [0], 1.0, {
        'maxfevals': 5_000
    })

    iteration = 0
    evaluation = 0
    while not es.stop():
        solutions = es.ask()
        print('asked to evaluate', len(solutions), 'solutions')

        futures = [evaluate.remote(x) for x in solutions]
        # costs = [evaluate(x) for x in solutions]
        costs = ray.get(futures)

        evaluation += len(solutions)
        iteration += 1
        print('evaluation', evaluation)
        print('computed costs:', list(reversed(sorted(costs))))
        print('avg costs:', np.mean(costs))

        es.tell(solutions, costs)
        es.disp()


        if iteration % 10 == 0:
            save_array(f'xfavorite-eval%07d.dat' % evaluation, es.result.xfavorite)
            save_array(f'stds-eval%07d.dat' % evaluation, es.result.stds)

    # res = es.result
    # print(res.xfavorite)


if __name__ == '__main__':
    main()
