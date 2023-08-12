#!/usr/bin/env python3
import numpy as np
from tqdm import tqdm, trange
import ray
import pickle
import random
import os
import time

from ribs.archives import GridArchive
from ribs.emitters import GaussianEmitter, EvolutionStrategyEmitter
from ribs.schedulers import Scheduler

import progenitor
version_check = 12
assert progenitor.mod.version_check == version_check, progenitor.__file__

os.makedirs('output', exist_ok=True)

def get_params(x, config):
    Params = progenitor.mod.Params
    hyperparams = {
        "init_fac": config["init_fac"],
        "bias_fac": config["bias_fac"]
    }
    return Params(x, **hyperparams)

@ray.remote
def evaluate(x):
    Builders = progenitor.mod.Builders
    Params = progenitor.mod.Params
    assert progenitor.mod.version_check == version_check, f'wrong module version: {Builders.__file__}'

    episodes=500

    hyperparams = {
        "actions_scale": 6,
        "bias_fac": 0.1,
        "init_fac": 0.7,
        "memory_clamp": 50,
        "memory_halftime": 2.5,
    }

    score = 0.0
    measure1 = 0.0
    measure2 = 0.0
    measure3 = 0.0
    for _ in range(episodes):
        params = Params(x, **hyperparams)
        sim = Builders(params)
        steps = 2000
        sim.steps(steps)
        measure1 += sim.walls_nearby / steps  # / n_agents
        # measure2 += np.log(sim.encounters + 10)
        measure3 += sim.relative_wall_edges()
        score += sim.hoarding_score

    score /= episodes
    measure1 /= episodes
    # measure2 /= episodes
    measure3 /= episodes

    # return (score, measure1, measure2, measure3)
    return (score, measure1, measure3)



param_count = progenitor.mod.Builders.param_count
print('param_count:', param_count)
# population_size = 120
population_size = 48
evaluations = 10_000_000

archive_dims=[50, 100]
archive_ranges=[(0.0, 25.0), (0.7, 1.1)]

# https://docs.pyribs.org/en/stable/tutorials/cma_mae.html
archive = GridArchive(
    solution_dim=param_count,
    dims=archive_dims,
    ranges=archive_ranges,
    learning_rate=0.01,
    threshold_min=0.0
    # qd_score_offset=-600
)

# to be validated: does the separate result_archive really help?
# (not using it may help filtering the evaluation noise, maybe?)
result_archive = GridArchive(solution_dim=param_count, dims=archive_dims, ranges=archive_ranges)

# emitters = [GaussianEmitter(archive, x0 = np.zeros(param_count), sigma = 0.5, batch_size=population_size)]
emitters = [EvolutionStrategyEmitter(
    archive,
    x0 = np.zeros(param_count),
    sigma0 = 1.0,
    batch_size=population_size,
    #ranker='2imp' if i < 5 else 'rd',
    # CMA-MAE
    ranker='imp',
    selection_rule='mu',
    restart_rule='basic',  # maybe also try 'no_improvement'? (I think 'basic' will never restart for my task. Or just increase the number of emitters instead, which probably serves a similar purpose...?)
) for _ in range(5)]

scheduler = Scheduler(archive, emitters, result_archive=result_archive)

actual_total_evals = 0
start_time = time.time()

for itr in trange(evaluations // (population_size * len(emitters))):
    solutions = scheduler.ask()

    futures = [evaluate.remote(x) for x in solutions]
    actual_total_evals += len(futures)
    results = np.array(ray.get(futures))
    objectives, measures = results[:, 0], results[:, 1:]

    scheduler.tell(objectives, measures)

    if itr % 10 == 0:
        futures_2 = [evaluate.remote(x) for x in solutions]
        results_2 = np.array(ray.get(futures_2))
        objectives_2, measures_2 = results_2[:, 0], results_2[:, 1:]
        print('measures, measures_2:', np.hstack((measures, measures_2)))
        m_idx = archive.index_of(measures)
        m_idx_2 = archive.index_of(measures_2)
        print(f'stable measures: {(m_idx == m_idx_2).sum() / len(m_idx) * 100:.1f}%')

    best = archive.best_elite
    assert best is not None
    assert archive.stats
    print(archive.stats)  # this one is good...

    if itr % 1 == 0:
        tqdm.write(f"> {itr} itrs completed after {time.time() - start_time:.2f}s")
        tqdm.write(f"  - Size: {archive.stats.num_elites}")
        tqdm.write(f"  - Coverage: {archive.stats.coverage}")
        tqdm.write(f"  - QD Score: {archive.stats.qd_score}")
        tqdm.write(f"  - Max Obj: {archive.stats.obj_max}")
        tqdm.write(f"  - Mean Obj: {archive.stats.obj_mean}")
        tqdm.write(f"  - Actual Evals: {actual_total_evals}")

    if itr % 10 == 0:
        fn = 'output/archive_autosave.pik'
        # print('saving to', fn)
        with open(fn, 'wb') as f:
            pickle.dump(archive, f)


import matplotlib.pyplot as plt
from ribs.visualize import grid_archive_heatmap

grid_archive_heatmap(archive)
plt.ylabel("walls_nearby")
plt.xlabel("relative_wall_edges")
plt.show()

# plt.figure(figsize=(8, 6))
# grid_archive_heatmap(archive, vmin=-300, vmax=300)
# plt.gca().invert_yaxis()  # Makes more sense if larger velocities are on top.
# plt.ylabel("Impact y-velocity")
# plt.xlabel("Impact x-position")
