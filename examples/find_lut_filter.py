#!/usr/bin/env python3
import numpy as np
import scipy.special as sc
import time
from pixeldrift import Cells, tile_size, render_cells

target_edge_frequency = 0.15
target_brigthness = 0.45
filter_steps = 11
size = tile_size


def generate_tile(lut, count=filter_steps):
    # generate binary noise image
    cells = Cells()
    # cells.set_data(np.random.randint(0, 4, (size, size)) == 1)
    cells.set_data(np.random.randint(0, 2, (size, size)) == 1)
    # apply the lut as filter several times
    for j in range(count):
        cells.apply_lut_filter(lut)
    return cells


def make_edge_detection_lut(neighbour):
    neighbour_mask = 1 << neighbour
    center_mask = 1 << 6  # FIXME: ...symbols, not magic numbers...
    lut = np.zeros(2**7, dtype='uint8')
    for key in range(2**7):
        if bool(key & center_mask) != bool(key & neighbour_mask):
            lut[key] = 1
    return lut

edge_luts = [make_edge_detection_lut(i) for i in range(3)]

def evaluate_lut(lut):
    cells = generate_tile(lut)

    edge_freqs = [cells.count_lut_filter(lut)
                  for lut in edge_luts]

    loss_edges = sum([(freq - target_edge_frequency)**2
                      for freq in edge_freqs])
    # additional penalty for breaking symmetry
    loss_edges += 2 * (edge_freqs[0] - edge_freqs[1])**2
    loss_edges += 2 * (edge_freqs[1] - edge_freqs[2])**2
    loss_edges += 2 * (edge_freqs[2] - edge_freqs[0])**2

    loss_brightness = 50 * (np.mean(cells.get_data()[cells.get_mask()]) - target_brigthness) ** 2
    # print('loss_edges:', loss_edges)
    # print('loss_brightness:', loss_brightness)
    return loss_edges + loss_brightness

def main():
    iterations = 100
    best_factor = 0.01
    population_size = 10000

    # A probability distribution over all (128-bit --> 1-bit) look-up tables,
    # to be optimized using the Cross-Entropoy Method (CEM).
    probs = np.ones(2**7) * 0.5

    for it in range(iterations):
        print('iteration', it)

        population = []
        for j in range(population_size):
            lut = (np.random.random(probs.shape) < probs)
            loss = evaluate_lut(lut)
            population.append((loss, lut))

        population.sort(key=lambda x: x[0])
        losses = np.array([ind[0] for ind in population])
        luts = np.array([ind[1] for ind in population])

        print(f'best loss: {losses[0]:.6f}')
        print(f'mean loss: {losses.mean():.6f}')
        noise = np.std([evaluate_lut(lut) for _ in range(10)])
        print(f'eval noise: {noise:.6f}')

        with open(f'outputs/best-lut-it{it:05}.txt', 'w') as f:
            np.savetxt(f, luts[0])
        cells = generate_tile(luts[0])
        t0 = time.time()
        render_cells(cells, f'outputs/best-lut-it{it:05}.png')
        print(f'render took {(time.time()-t0)*1000:.1f}ms')

        # estimate the new probability distribution from the best samples
        probs = luts[:int(population_size * best_factor), :].mean(0)
        print('some probs are', probs[:8])
        # probs = probs.clip(0.003, 0.997)  # add a minimum amount of noise

        def binary_entropy(x):
            return -(sc.xlogy(x, x) + sc.xlog1py(1 - x, -x)) / np.log(2)
        print('entropy is %.6f bits' % binary_entropy(probs).sum())


if __name__ == '__main__':
    main()
