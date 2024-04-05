#!/usr/bin/env python3
import numpy as np
import argparse
import os, sys
sys.path.append(os.path.dirname(__file__))

import ray
from ray import tune
import ray.tune.utils
from ray.tune.schedulers import ASHAScheduler, PopulationBasedTraining
from ray.train import RunConfig, CheckpointConfig, FailureConfig
from builders_trainable import BuildersTrainable


def main_tune(args):
    run_name = 'builders-' + args.run_name

    hyperparam_base = {
        "episodes_per_eval": 60,
        "popsize": 120,
        "seeding": "epoch",
        "rust": {
            "actions_scale": 6.8,
            "bias_fac": 0.1,
            "init_fac": 0.65,
            "n_hidden": 20,
            "n_hidden2": 20,
            "memory_clamp": 50,
            "memory_halftime": 2.7,
        },
    }

    tune_config = tune.TuneConfig(
        # max_concurrent_trials is needed if we don't reserve any resources per-trial.
        # (check with above resources_per_trial and scheduling_strategy)
        # max_concurrent_trials=16,
        num_samples=16,  # "runs" (or "restarts", or "trials", aka potentially-parallel trainings)
        metric='score',
        mode='max',
        scheduler=PopulationBasedTraining(
            time_attr="total_episodes",
            perturbation_interval=1_000_000,
            hyperparam_mutations={
                # distribution for resampling
                "episodes_per_eval": tune.lograndint(50, 500),
            },
        )
    )

    tuner = tune.Tuner(
            BuildersTrainable,
        run_config=RunConfig(
            name=run_name,
            # stop={"mean_accuracy": 0.96, "training_iteration": 50} # don't think we need this here...
            checkpoint_config=CheckpointConfig(
                # checkpoint_score_attribute="score", # deletes according to score (instead of oldest)
                num_to_keep=3,
                checkpoint_frequency=10,  # ???
                checkpoint_at_end=True,
            ),
            failure_config=FailureConfig(max_failures=3),
            storage_path=args.storage # s3 bucket for cloud run with checkpoints (or None for ~/ray_results/)
        ),
        param_space=hyperparam_base,
        tune_config=tune_config
    )

    if args.validate_only:
        ray.init()
        ray.tune.utils.validate_save_restore(BuildersTrainable, hyperparam_base)
        print('Passed save-restore validation.')
        return

    analysis = tuner.fit()
    # best_trial = analysis.get_best_trial(metric="accuracy", mode="max", scope="all")
    # best_checkpoint = analysis.get_best_checkpoint(best_trial, metric="accuracy")

    # df = analysis.get_dataframe()
    # df = df.sort_values('score', ascending=False)  # keyerror: score
    # df.to_csv('output/tuner_result.csv')
    # ...or just start tensorboard in ~/ray_results/

    print("Config of best run:", analysis.get_best_result().config)


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('run_name')
    parser.add_argument('--storage')  # must be accessible to all (synced via nfs, or cloud storage)
    parser.add_argument('--validate-only', default=False, action='store_true')
    args = parser.parse_args()

    main_tune(args)


if __name__ == '__main__':
    main()
