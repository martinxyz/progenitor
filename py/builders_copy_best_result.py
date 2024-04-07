#!/usr/bin/env python3
import numpy as np
import os.path
import json
from glob import glob
import time
from builders_common import get_params, load_maybe_gz, load_pik_blosc

import progenitor


# directories = sorted(glob('/home/martin/ray_results/builders-*'), key=os.path.getmtime)
# experiment = directories[-1]
experiment = '/home/martin/ray_results/s3sync/builders-gen2-pbt-1'
print('experiment:', experiment)

# for run in sorted(glob(os.path.join('/home/martin/ray_results/', experiment, '*'))):
scores = []
for result in sorted(glob(os.path.join(experiment, '*', 'result.json'))):
    try:
        data = json.loads(open(result).readlines()[-1])
    except:
        print('skipping invalid:', result)
        continue
    scores.append((data['score'], result))

scores.sort()
scores.reverse()
scores = scores[:3]
for i, (score, result) in enumerate(scores):
    data = json.loads(open(result).readlines()[-2])
    print(data['trial_id'], data['total_episodes'], data['score'])
    print(data)
    data_dir = os.path.dirname(result)
    print(data_dir)

    files = glob(os.path.join(data_dir, f'**/xfavorite-*.params.bin'))
    if not files:
        # temporary, for runs that didn't save the above
        files = glob(os.path.join(data_dir, f'**/xfavorite.pik.blosc'))
    files = list(sorted(files, key=os.path.getmtime))
    latest_bin = files[-1]
    print(latest_bin)

    if '.params.bin' in latest_bin:
        params = load_maybe_gz(latest_bin)
    else:
        # temporary, for runs that didn't save the above
        print('warning: using workaround')
        assert latest_bin.endswith('xfavorite.pik.blosc')
        x = load_pik_blosc(latest_bin)
        params = get_params(x, data['config'])
        params = params.serialize()

    def evaluate(params, N):
        Builders = progenitor.mod.Builders
        scores = []
        for _ in range(N):
            sim = Builders(params, seed=None)
            sim.steps(3000)
            # sim.steps(10_000)
            scores.append(sim.hoarding_score)
        return np.array(scores)

    print('evaluating...')
    t0 = time.time()
    N=1000
    scores = evaluate(progenitor.mod.Params.deserialize(params), N)
    duration = time.time() - t0
    print('evaluation speed: {:0.3f} ms per eval'.format(duration / N * 1000))
    print(f'mean {scores.mean():.3f}, median {np.median(scores)},std {scores.std():.3f}, {scores.min()}..{scores.max()}')

    if i == 0:
        with open('/home/martin/code/progenitor/crates/progenitor/src/builders/optimized.params.bin', 'wb') as f:
            f.write(params)
        print('(copied)')

    print()

