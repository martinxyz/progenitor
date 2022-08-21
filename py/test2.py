#!/usr/bin/env python3
# broken... no more Cells export...
import numpy as np
from progenitor import Cells

# from progenitor import tile_size
tile_size = 4

print('FIXME: tile_size:', tile_size)
cells = Cells()

data = np.random.randint(0, 3, size=(tile_size, tile_size), dtype='u8')
data = data.astype('bool')
# cells.get_data()
print('set_data() with', data.mean(), data.shape, data.dtype)
cells.set_data(data)
print('get_data()')
d = cells.get_data()
print(d.mean(), d.shape, d.dtype)
