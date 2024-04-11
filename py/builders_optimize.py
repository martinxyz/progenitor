#!/usr/bin/env python3
import argparse
import os, sys
sys.path.append(os.path.dirname(__file__))

import ray
from ray import tune
import ray.tune.utils
from ray.tune.schedulers import ASHAScheduler, FIFOScheduler, PopulationBasedTraining
from ray.train import RunConfig, CheckpointConfig, FailureConfig
from builders_trainable import BuildersTrainable


def main_tune(args):
    run_name = 'builders-' + args.run_name

    config = {
        # "episodes_per_eval": tune.lograndint(25, 750),
        # "episodes_per_eval": tune.lograndint(50, 500),
        "episodes_per_eval": 60, # ("denoising" effect ~= popsize*episodes_per_eval)
        # high popsize: lowers the chance to get a good result, but the few good ones get better
        #               (maybe they fail only because we stop them early...?)
        "popsize": tune.lograndint(30, 200),  # plausible range: 85..250(?)
        # "popsize": 120,
        "rust": {
            # "actions_scale": 6.8,
            # "bias_fac": 0.1,
            # "init_fac": 0.65,
            # "n_hidden": 20,
            # "n_hidden2": 20,
            # "memory_clamp": 50,
            # "memory_halftime": 2.7,
            "actions_scale": tune.loguniform(1.0, 40.),  # plausible range: 2.0..20
            "bias_fac": 0.1, # plausible range: 0.01..0.9 (0.1 is fine, across many variants)
            "init_fac": tune.loguniform(0.1, 1.5),  # (clear effect) plausible range: 0.4..0.8
            "n_hidden": tune.lograndint(8, 20+1),
            # "n_hidden2": tune.lograndint(6, 10+1),
            "n_hidden2": 10,
            "memory_clamp": tune.loguniform(0.2, 60.0),  # plausible range: 1.0..50
            "memory_halftime": tune.loguniform(1.5, 16.0), # plausible range: 2..10
        },
    }

    if False:  # PBT
        config["cpus"] = 11  # cpus allocated per trial
        tune_config = tune.TuneConfig(
            num_samples=16,  # "runs" (or "restarts", or "trials", aka potentially-parallel trainings)
            metric='score',
            mode='max',
            scheduler=PopulationBasedTraining(
                time_attr="total_episodes",
                perturbation_interval=5_000_000,  # reduce to 2M maybe...? (5M gets to score 50 before first perturbation)
                hyperparam_mutations={
                    # distribution for resampling  (?? is it redundant ??)
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
                    # num_to_keep=3,
                    num_to_keep=None,  # keep all checkpoints
                    # ??? (Does probably affect which models PBT can choose...? Not sure.)
                    # maybe see: https://docs.ray.io/en/latest/tune/examples/pbt_visualization/pbt_visualization.html#create-the-tuner
                    checkpoint_frequency=50,
                    checkpoint_at_end=True,
                ),
                failure_config=FailureConfig(max_failures=5),
                storage_path=args.storage
            ),
            param_space=config,
            tune_config=tune_config
        )

    if False:  # ASHA
        max_episodes = 20_000_000
        config["cpus"] = 0  # break tasks out of trial's placement group, see BuildersTrainable
        tune_config = tune.TuneConfig(
            max_concurrent_trials=16,  # needed because we don't reserve any CPUs per-trial
            num_samples=81,  # "runs" or "restarts"
            metric='score',
            mode='max',
            scheduler=ASHAScheduler(
                time_attr='total_episodes',
                grace_period=max_episodes // 50,  # training "time" allowed for every "sample" (run)
                max_t=max_episodes,      # training "time" allowed for the best run(s)
                reduction_factor=3,
                brackets=1,
            )
        )
        tuner = tune.Tuner(
            BuildersTrainable,
            run_config=RunConfig(
                name=run_name,
                # stop={"mean_accuracy": 0.96, "training_iteration": 50} # don't think we need this here...
                checkpoint_config=CheckpointConfig(
                    # checkpoint_score_attribute="score", # deletes according to score (instead of oldest)
                    num_to_keep=None,  # None = keep all checkpoints
                    checkpoint_frequency=50,  # ???
                    checkpoint_at_end=True,
                ),
                failure_config=FailureConfig(max_failures=5),
                storage_path=args.storage
            ),
            param_space=config,
            tune_config=tune_config
        )

    if True:  # Random Search
        max_episodes = 20_000_000
        config["cpus"] = 11  # cpus allocated per trial
        tune_config = tune.TuneConfig(
            num_samples=100,  # "runs" or "restarts"
            metric='score',
            mode='max',
        )
        tuner = tune.Tuner(
            BuildersTrainable,
            run_config=RunConfig(
                name=run_name,
                checkpoint_config=CheckpointConfig(
                    checkpoint_frequency=0,  # 0 = disable
                    checkpoint_at_end=True,
                ),
                failure_config=FailureConfig(max_failures=5),
                storage_path=args.storage,
                stop={"total_episodes": max_episodes}
            ),
            param_space=config,
            tune_config=tune_config
        )

    if args.validate_only:
        ray.init()
        ray.tune.utils.validate_save_restore(BuildersTrainable, config)
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
    parser.add_argument('--storage', help='when running distributed, this is mandatory and must be a storage accessible to all workers (synced via nfs or a s3:// bucket)')
    parser.add_argument('--validate-only', default=False, action='store_true')
    args = parser.parse_args()

    main_tune(args)


if __name__ == '__main__':
    main()
