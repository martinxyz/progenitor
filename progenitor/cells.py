# from .progenitor import World as _World, CellContent as _CellContent
from .progenitor import get_tile_size, Cells
from .progenitor import Cells as _Cells
from typing import NamedTuple, List
import numpy as np

tile_size = get_tile_size()

class CellContent(NamedTuple):
    cell_type: int = 0
    child_count: int = 0
    particle: bool = False

class Cells():
    """Represents a hexagonal grid of cells.

    Currently only a hardcoded tile_size with toroidal tiling is supported.

    The API is designed to be extended for other geometries. (At least fixed
    borders of any shape without tiling; maybe also hexagonal mirroring.)
    """

    # def __init__(self, _storage=None):
    #     self._cells = _storage or _World()
    def __init__(self):
        self._cells = _Cells()

    # def copy(self):  # bad naming: only copies one channel
    #     copy = Cells()
    #     copy.set_data(self.get_data())
    #     return copy

    def get_bbox(self):
        """Get a minimal bounding box that includes all core cells"""
        return (0, 0, tile_size, tile_size)

    def get_data(self, x=0, y=0, w=tile_size, h=tile_size, channel='particles'):
        """Get a copy of the cell data in a region.

        Returns a new numpy array of type 'uint8' and shape (h, w).

        The region may be smaller or larger than one tile. The returned array
        uses offset coordinates. If y is even the array will be odd-rows-right,
        otherwise odd-rows-left.
        """
        return getattr(self._cells, 'get_' + channel)(x, y, w, h)

    def set_data(self, data, x=0, y=0, channel='particles'):
        """Set the cell data of a region.

        Data must be a numpy array of type 'uint8' and can have any 2d-shape.
        It is possible to write into mirrored locations, and to overwrite the
        same core cell multiple times. The last written value will have effect.

        Same offset coordinates apply as for get_data().
        """
        return getattr(self._cells, 'set_' + channel)(data.astype('uint8'), x, y)

    def get_mask(self, x=0, y=0, w=tile_size, h=tile_size):
        """Get the bitmask for core cells in a region.

        Returns a new numpy array of type 'bool' and shape (h, w).

        The mask is False for borders (if used) and for any tiled or mirrored
        copies of core cells.
        """
        res = np.zeros((h, w), dtype='bool')
        x0, y0 = max(-x, 0), max(-y, 0)
        x1, y1 = min(-x + tile_size, w), min(-y + tile_size, h)
        if x1 > x0 and y1 > y0:
            res[y0:y1, x0:x1] = True
        return res

    def apply_lut_filter(self, lut):
        self._cells.apply_lut_filter(lut)
        return self

    def count_lut_filter(self, lut) -> int:
        return self._cells.count_lut_filter(lut)

    # def set_cell(self, x: int, y: int, data: CellContent):
    #     cc = _CellContent()
    #     cc.cell_type = data.cell_type
    #     cc.child_count = data.child_count
    #     cc.particle = data.particle
    #     self._cells.set_cell(x, y, cc)

    # def get_cell(self, x: int, y: int) -> CellContent:
    #     return self._cells.get_cell(x, y)

    # def get_cells(self, x=0, y=0, w=tile_size, h=tile_size):
    #     # veeery slow
    #     result = []
    #     for yy in range(h):
    #         row = []
    #         result.append(row)
    #         for xx in range(w):
    #             row.append(self.get_cell(x + xx, y + yy))
    #     return result

    # def count_cells_of_type(self, cell_type: int) -> int:
    #     cnt = 0
    #     for row in self.get_cells():
    #         for cell in row:
    #             if cell.cell_type == cell_type:
    #                 cnt += 1
    #     return cnt
