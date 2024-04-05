#!/usr/bin/env python3
import numpy as np
import sys
import argparse
import os
from ray import tune, train
from ray.tune.schedulers import ASHAScheduler
import ribs_trainable

def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('run_name')
    parser.add_argument('--train_storage_dir')  # must be accessible to all (synced via nfs, or cloud storage)
    args = parser.parse_args()

    run_name = 'ribs-' + args.run_name

    search_space = {
        "rust": {
            "actions_scale": tune.loguniform(5.0, 10.),  # plausible range: 2.0..20
            "bias_fac": 0.1,
            "init_fac": tune.loguniform(0.3, 0.8),  # (clear effect) plausible range: 0.2..1.3
            "memory_clamp": 50,
            "memory_halftime": tune.loguniform(1.5, 8.0), # plausible range: 1.5..?~100?
        },
        # "popsize": tune.lograndint(20, 70),
        "popsize": 32,
        # "archive_learning_rate": tune.loguniform(0.02, 0.7),
        "archive_learning_rate": 0.05,
        # "archive_scale": tune.choice([1, 2, 3, 4]),  # checkpoints get too large (half a gigabyte...), also, lager seems slightly worse
        # "archive_scale": tune.choice([1, 2]),
        "archive_scale": 1,
        "es": "sep_cma_es",
        # "es": tune.choice([
        #     "cma_es",
        #     "sep_cma_es",
        #     # "lm_ma_es",
        # ]),
        # "num_emitters": tune.lograndint(5, 25),  # probably interacts with popsize; effect seems neutral (so far)
        "num_emitters": 16,
        # ...complicated effect... "episodes_per_eval": tune.lograndint(50, 500), # ("denoising" effect ~= popsize*episodes_per_eval (except for measure stability))
        # "episodes_per_eval": tune.lograndint(80, 180),  # lower seems better, but... it may just make the extremely lucky score more extreme
        "episodes_per_eval": 500,
        # "seeding": "epoch",  - not implemented yet (does it still make sense, with all the measure-noise?)
    }
    max_episodes = 200 * int(1e6)
    # max_episodes = 20 * int(1e6)
    tune_config = tune.TuneConfig(
        num_samples=9,  # "runs" or "restarts"
        metric='result_norm_qd_score',
        mode='max',
        scheduler=ASHAScheduler(
            time_attr='episodes',
            grace_period=max_episodes // 50,  # training "time" allowed for every "sample" (run)
            # grace_period=max_episodes // 2,  # training "time" allowed for every "sample" (run)
            max_t=max_episodes,      # training "time" allowed for the best run(s)
            reduction_factor=3,
            brackets=1,
        )
    )
    # resources_per_trial={'cpu': 1, 'gpu': 0}
    resources_per_trial=tune.PlacementGroupFactory(
        # [{'CPU': 0.0}] + [{'CPU': 1.0}] * 64  # for a single run with popsize=64*16
        [{'CPU': 0.0}] + [{'CPU': 1.0}] * 32
        #-------------   ------------------
        # train() task,      ^ evaluate() tasks spawned by train().
        # does work once       They can use more CPUs, how many depends
        # per generation.      on the population_size (a hyperparam).
        # (short burst)        But those we reserve (but one) will idle during
        #                      each generation-evaluation. If we reserve
        #                      just one, utilization will be nearly 100% at the
        #                      beginning but the last few runs remaining will
        #                      use a single CPU each, when they could parallelize.
        #                      ...any good solution for that? (Fractional CPUs?)
        #
        # https://docs.ray.io/en/latest/tune/faq.html#how-do-i-set-resources
        # https://docs.ray.io/en/latest/ray-core/scheduling/placement-group.html
        # , strategy='PACK'
        #
        # [{'CPU': 0.0}] + [{'CPU': 1.0}] * (32 * 8)
        ## ^ this doesn't work. even in the simple (1 job, 1 trial) ray seems to fail starting tasks fast enough to saturate things
        ## (Maybe it does something silly...? like setting up venvs for each task? or transfering pointless objects...?)
        ## Or, more likely, my tasks are too short. (How long are they? Should be at least 0.1s, better 1.0s or more.)
    )
    tuner = tune.Tuner(
        tune.with_resources(
            ribs_trainable.RibsTrainable,
            resources=resources_per_trial
            # resources=lambda config: {"GPU": 1} if config["use_gpu"] else {"GPU": 0},
        ),
        run_config=train.RunConfig(
            name=run_name,
            storage_path=args.train_storage_dir,
            log_to_file='log.txt',
            checkpoint_config=train.CheckpointConfig(
                checkpoint_frequency=50,
                checkpoint_at_end=True,
                num_to_keep=2,
            ),
            failure_config=train.FailureConfig(max_failures=3),
        ),
        param_space=search_space,
        tune_config=tune_config
    )
    analysis = tuner.fit()
    # tune.run(keep_checkpoints_num=1, checkpoint_score_attr="accuracy")
    # best_trial = analysis.get_best_trial(metric="accuracy", mode="max", scope="all")
    # best_checkpoint = analysis.get_best_checkpoint(best_trial, metric="accuracy")

    # df = analysis.get_dataframe()
    # df.to_csv('output/tuner_result.csv')
    # ...or just start tensorboard in ~/ray_results/
    print("Config of best run:", analysis.get_best_result().config)

if __name__ == '__main__':
    main()
