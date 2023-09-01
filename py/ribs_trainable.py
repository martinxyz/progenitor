import dataclasses
import numpy as np
import os
import time
import pickle
import blosc
import ray
import ray.tune
import ray.air

from ribs.archives import GridArchive
from ribs.emitters import GaussianEmitter, EvolutionStrategyEmitter
from ribs.schedulers import Scheduler

import matplotlib
matplotlib.use('agg')
import matplotlib.pyplot as plt
from ribs.visualize import grid_archive_heatmap

import ribs_task

@ray.remote
def evaluate(x, hyperparams, episodes):
    measures = ribs_task.evaluate(x, hyperparams, episodes)
    return (
        measures[ribs_task.measure_score],
        measures[ribs_task.measure_1],
        measures[ribs_task.measure_2],
    )

class RibsTrainable(ray.tune.Trainable):
    def setup(self, config):
        param_count = ribs_task.param_count()
        print('param_count:', param_count)
        self.hyperparams = config["rust"]

        # population_size = 120
        population_size = config["popsize"]
        # evaluations = 10_000_000

        self.episodes_per_eval = config["episodes_per_eval"]

        archive_dims=[50 * config["archive_scale"], 100 * config["archive_scale"]]
        archive_ranges=[
            ribs_task.measure_ranges[ribs_task.measure_1],
            ribs_task.measure_ranges[ribs_task.measure_2],
        ]

        # https://docs.pyribs.org/en/stable/tutorials/cma_mae.html
        self.archive = GridArchive(
            solution_dim=param_count,
            dims=archive_dims,
            ranges=archive_ranges,
            learning_rate=config["archive_learning_rate"],
            threshold_min=0.0
            # qd_score_offset=-600
        )

        # to be validated: does the separate result_archive really help?
        # (not using it may help filtering the evaluation noise, maybe?)
        self.result_archive = GridArchive(solution_dim=param_count, dims=archive_dims, ranges=archive_ranges)

        # emitters = [GaussianEmitter(archive, x0 = np.zeros(param_count), sigma = 0.5, batch_size=population_size)]
        emitters = [EvolutionStrategyEmitter(
            self.archive,
            x0 = np.zeros(param_count),
            sigma0 = 1.0,
            batch_size=population_size,
            #ranker='2imp' if i < 5 else 'rd',
            es="cma_es",
            # CMA-MAE
            ranker='imp',
            selection_rule='mu',
            restart_rule='basic',  # maybe also try 'no_improvement'? (I think 'basic' will never restart for my task. Or just increase the number of emitters instead, which probably serves a similar purpose...?)
        ) for _ in range(config["num_emitters"])]

        self.scheduler = Scheduler(self.archive, emitters, result_archive=self.result_archive)
        self.evals = 0
        self.episodes = 0
        self.generations = 0
        self.pending_reports = []

    def step(self):
        if self.pending_reports:
            return self.pending_reports.pop(0)

        # make the tensorboard x-axis ("steps") more useful, independent of hyperparams
        episodes_per_report = 10_000
        reports_done = self.episodes // episodes_per_report
        while self.episodes // episodes_per_report <= reports_done:
            solutions = self.scheduler.ask()
            futures = [evaluate.remote(x, self.hyperparams, self.episodes_per_eval) for x in solutions]
            if self.episodes < 5:
                print("evaluating", len(futures), "solutions")
            self.evals += len(futures)
            self.episodes += len(futures) * self.episodes_per_eval
            self.generations += 1
            results = np.array(ray.get(futures))
            objectives, measures = results[:, 0], results[:, 1:]
            self.scheduler.tell(objectives, measures)

            if self.generations % 100 == 1:
                futures_2 = [evaluate.remote(x, self.hyperparams, self.episodes_per_eval) for x in solutions]
                results_2 = np.array(ray.get(futures_2))
                objectives_2, measures_2 = results_2[:, 0], results_2[:, 1:]
                # print('measures, measures_2:', np.hstack((measures, measures_2)))
                # print('objectives, objectives_2:', np.hstack((measures, measures_2)))
                m_idx = self.archive.index_of(measures)
                m_idx_2 = self.archive.index_of(measures_2)
                print(f'expensive report at generation {self.generations} ({self.evals/1e6:.3f} M evals)')
                print(f'evals with stable measures (same archive bin): {(m_idx == m_idx_2).sum() / len(m_idx) * 100:.1f}% (N={len(futures_2)})')
                print('archive:', self.archive.stats)
                print('result_archive:', self.result_archive.stats)
                t0 = time.time()
                plot_archive(self.archive, f'archive-gen{self.generations:06}.png')
                plot_archive(self.result_archive, f'result_archive-gen{self.generations:06}.png')
                print(f'archive plots created in {time.time()-t0:.3} seconds')

        assert(self.archive.stats is not None)
        assert(self.result_archive.stats is not None)

        for _ in range(self.episodes // episodes_per_report - reports_done):
            self.pending_reports.append({
                **dataclasses.asdict(self.archive.stats),
                "result_norm_qd_score": self.result_archive.stats.norm_qd_score,
                "result_obj_max": self.result_archive.stats.obj_max,
                "result_obj_mean": self.result_archive.stats.obj_mean,
                "episodes": self.episodes,
                "evals": self.evals,
                "generations": self.generations,
            })
        return self.pending_reports.pop(0)

    def save_checkpoint(self, tmpdir):
        # compressed archive is e.g. ~33MB for 1k params (can be further compressed to 18MB)
        # (that's roughly the expected size)
        save_blosc(self.archive, os.path.join(tmpdir, "archive.pik.blosc"))
        save_blosc(self.result_archive, os.path.join(tmpdir, "result_archive.pik.blosc"))

        # full state is 0.5GB compressed; seems excessive given my expectation
        # that it holds just a few sep_cma_es emitter states plus the two archives
        # (At least resuming from node failure works this way!)
        #
        # full_state = (self.archive, self.result_archive, self.evals, self.episodes, self.generations, self.scheduler, self.pending_reports)
        # save_blosc(full_state, os.path.join(tmpdir, "full_state.pik.blosc"))

        return tmpdir

    def load_checkpoint(self, tmpdir):
        raise NotImplementedError
        # Checkpointing (to S3) just doesn't work reliably.
        # full_state = load_blosc(os.path.join(tmpdir, "full_state.pik.blosc"))
        # (self.archive, self.result_archive, self.evals, self.episodes, self.generations, self.scheduler, self.pending_reports) = full_state

def save_blosc(obj, filename):
    data = pickle.dumps(obj)
    data = blosc.compress(data)
    with open(filename, 'wb') as f:
        f.write(data)

def load_blosc(filename):
    with open(filename, 'rb') as f:
        data = f.read()
    data = blosc.decompress(data)
    return pickle.loads(data)

def plot_archive(archive, filename):
    plt.clf()
    plt.title(ribs_task.measure_score)
    plt.xlabel(ribs_task.measure_1)
    plt.ylabel(ribs_task.measure_2)
    grid_archive_heatmap(archive)
    plt.savefig(filename)
