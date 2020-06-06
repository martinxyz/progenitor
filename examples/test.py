#!/usr/bin/env python3
import numpy as np
import scipy.special as sc
import time
# from progenitor import Cells, tile_size, render_cells
from progenitor import mod

# target_edge_frequency = 0.15
# target_brigthness = 0.45
# filter_steps = 11
# size = mod.tile_size
size = mod.get_tile_size()
print('size:', size)

lut = np.zeros(2**7, dtype='uint8')
# cells = Cells()
# cells.apply_lut_filter(lut)

print([s for s in dir(mod) if not s.startswith('_')])
cells = mod.Cells()
print('cells.size', cells.size)
print('cells', cells)
# cells.apply_lut_filter([3, 4, 5, 33, 3])
cells.test_buffer_protocol(b'abcde')
cells.test_buffer_protocol(np.zeros((3, 118), dtype='u8'))
