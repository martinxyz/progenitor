import numpy as np
import cmaes
import ray
import ray.tune
from builders_common import get_params, save_gz, save_pik_blosc, load_pik_blosc

import progenitor
# We cannot import progenitor.mod.Builders and then use it in the @ray.remote,
# apparently. (I think the @ray.remote object fails to serialize.)
# (Is this still true when starting with the submit-job.sh script?)

@ray.remote
def evaluate_real(x, config, seeds: np.ndarray, stats=False):
    Builders = progenitor.mod.Builders
    params = get_params(x, config)

    scores = []
    for seed in seeds:
        sim = Builders(params, seed)
        sim.steps(10_000)
        scores.append(sim.hoarding_score)
        if stats:
            stats = False
            sim.print_stats()
            # report those via tensorboard? calculate entropy of actions, too?
            # (Just return a metrics dict? Averaged/Stats?)

    costs = - np.array(scores, dtype='float32')
    return costs


class BuildersTrainable(ray.tune.Trainable):
    @classmethod
    def default_resource_request(cls, config):
        resources_per_trial = [{'CPU': 0.01}] + [{'CPU': 1.0}] * config["cpus"]
        # resources_per_trial = tune.PlacementGroupFactory(
        #   [{'CPU': 0.0}] + [{'CPU': 1.0}] * 64  # for a single run with popsize=64*16
        #   [{'CPU': 0.0}] + []  # not allowed
        #  -------------   ------------------
        #   train() task,      ^ evaluate() tasks spawned by train().
        #   does work once       They can use more CPUs, how many depends
        #   per generation.      on the population_size (a hyperparam).
        #   (short burst)        (-> Run those outside of the placement group instead.)
        #
        #   https://docs.ray.io/en/latest/tune/faq.html#how-do-i-set-resources
        #   https://docs.ray.io/en/latest/ray-core/scheduling/placement-group.html
        #   https://discuss.ray.io/t/unable-to-saturate-cluster-with-asha-trials-cpu-bound/11941
        # )
        return ray.tune.PlacementGroupFactory(resources_per_trial)

    def setup(self, config):
        N = get_params(None, config).count_params()
        print('param_count:', N)

        self.optimizer = cmaes.SepCMA(
            mean=np.zeros(N),
            sigma=1.0,
            population_size=config["popsize"]
        )
        self.episodes = 0
        self.rng = np.random.default_rng()

    def step(self):
        # computation effort per step() should be independent of hyperparams; this will also be the tensorboard x-axis
        episodes_per_step = 100_000

        steps_done = self.episodes // episodes_per_step
        assert self.iteration == steps_done
        costs = []
        while self.episodes // episodes_per_step <= steps_done:
            episodes_evaluated, cost = self.train_one_iteration()
            costs.append(cost)
            self.episodes += episodes_evaluated
            assert episodes_evaluated <= episodes_per_step, "multiple step()s per iteration is not implemented"
        cost_population = np.mean(costs)
        cost_population_std = np.std(costs)

        # print()
        # print(f'reporting at {self.episodes} episodes (iter {self.iteration}):')
        result = self.validate_performance()
        result.update({
            'score_population': -cost_population,
            'score_population_std': cost_population_std,
            'total_episodes': self.episodes
        })
        # print(f'  average population cost: {result["score_population"]:.6f}')
        # print(f'  average validation cost: {result["score"]:.6f}')
        return result

    def train_one_iteration(self):
        config = self.config
        n_evals = config["episodes_per_eval"]

        seeds = self.rng.integers(1_000_000_000, size=n_evals)

        if config["cpus"] == 0:
            # break out of the training actor's placement group
            evaluate = evaluate_real.options(scheduling_strategy='DEFAULT')
        else:
            evaluate = evaluate_real

        solutions = []
        futures = []
        for _ in range(self.optimizer.population_size):
            # This is a performance bottleneck for cmaes.CMA. Time is spent
            # in numpy matrix operations; "popsize" size was <= 120.
            # Creating tasks as samples arrive helps a bit, I guess?
            x = self.optimizer.ask()
            solutions.append(x)
            futures.append(evaluate.remote(x, config, seeds))
        costs_per_eval = ray.get(futures)
        costs = [per_eval.mean() for per_eval in costs_per_eval]

        episodes_evaluated = len(solutions) * config["episodes_per_eval"]
        self.optimizer.tell(list(zip(solutions, costs)))
        return episodes_evaluated, np.mean(costs)

    def validate_performance(self) -> dict:
        validation_episodes = 512

        seeds = np.arange(validation_episodes)  # hardcoded seeds to reduce noise
        x = self.optimizer._mean

        if self.config["cpus"] == 0:
            # break out of the training actor's placement group
            evaluate = evaluate_real.options(scheduling_strategy='DEFAULT')
        else:
            evaluate = evaluate_real

        futures = []
        # seed_chunks = np.array_split(seeds, config["cpus"])  # bad: config["cpus"] may be zero
        seed_chunks = np.array_split(seeds, validation_episodes // 16)
        for s in seed_chunks:
            futures.append(evaluate.remote(x, self.config, s))
        costs_per_eval = ray.get(futures)
        costs = [per_eval.mean() for per_eval in costs_per_eval]

        # When running distributed, those files end up on the worker and get deleted with it.
        # (Okay for debugging or local runs.)
        # np.save(f'costs_per_eval-{episodes}.npy', costs_per_eval)
        # params_favourite = get_params(x, self.config)
        # save_gz(f'xfavorite-{self.episodes}.params.gz', params_favourite.serialize())

        score = -np.array(costs)

        return {
            'score': np.mean(score),
            'score_std': np.std(score),
        }

    def save_checkpoint(self, tmpdir):
        state = {
            "optimizer": self.optimizer,
            "episodes": self.episodes,
            # not restored, just for archival:
            "config": self.config,
        }
        save_pik_blosc(f'{tmpdir}/state.pik.blosc', state)  # ~48kB currently

        # not restored, but convenient for later use of the result:

        x = self.optimizer._mean
        # Useful as an easy backup maybe, with fewer dependencies? (currently unused)
        save_pik_blosc(f'{tmpdir}/xfavorite.pik.blosc', x)  # ~12kB currently

        # used for convenient re-evaluation (web-viewer or Python)
        params_favourite = get_params(x, self.config)
        save_gz(f'{tmpdir}/xfavorite-{self.episodes}.params.bin.gz', params_favourite.serialize())


    def load_checkpoint(self, tmpdir):
        state = load_pik_blosc(f'{tmpdir}/state.pik.blosc')
        self.optimizer = state["optimizer"]
        self.episodes = state["episodes"]
        print('restored checkpoint from episodes=', self.episodes)
