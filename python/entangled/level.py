import json

from collections import namedtuple
from functools import singledispatch


AxialCoord = namedtuple("AxialCoord", ["q", "r"])

class Unit():
    PLAYER_CONTROL = 1
    CPU_CONTROL = 2

    def __init__(self, control=CPU_CONTROL, friendly=False):
        self.friendly = friendly
        self.control = control


class HexTile():
    def __init__(self, coord: AxialCoord):
        self.coord = coord
        self.unit = None

    def __hash__(self):
        return hash(self.coord)

    def __repr__(self):
        return "(q={}, r={})".format(self.coord.q, self.coord.r)


class HexGrid():
    def __init__(self, coords=None):
        if not coords:
            self.tiles = dict()
        else:
            self.tiles = {coord: HexTile(coord) for coord in coords}
        self._min_q = None
        self._max_r = None
        self._min_r = None
        self._max_r = None

    def add(self, tile: HexTile):
        self.tiles[tile.coord] = tile

    def get(self, q, r):
        return self.tiles.get(AxialCoord(q, r))

    def __repr__(self):
        return "\n".join(str(coord) for coord in self.tiles.keys())


@singledispatch
def to_json(val):
    return json.dumps(val)


@to_json.register(Unit)
def unit_json(unit):
    return {"friendly": unit.friendly,
            "control": unit.control}


@to_json.register(HexGrid)
def hex_grid_json(grid):
    return [{"q": coord.q,
             "r": coord.r,
             "unit": unit_json(tile.unit) if tile.unit else None}
            for coord, tile in grid.tiles.items()]


def coords_circle(center: AxialCoord, radius: int):
    # XXX: looping over cube coords, but there might be a better way
    for x in range(-radius, radius + 1):
        dq = x
        for y in range(max(-radius, -x - radius),
                       min(radius, -x + radius) + 1):
            dr = 0 - x - y
            yield AxialCoord(center.q + dq, center.r + dr)
