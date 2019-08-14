from collections import namedtuple


AxialCoord = namedtuple("AxialCoord", ["q", "r"])

AXIAL_NEIGHBORS = frozenset(
    ((0, -1), (1, -1), (1, 0), (0, 1), (-1, 1), (-1, 0))
)


class HexTile:
    def __init__(self, coord: AxialCoord):
        self.coord = coord
        self.unit = None

    def __hash__(self):
        return hash(self.coord)

    def __repr__(self):
        return "(q={}, r={})".format(self.coord.q, self.coord.r)


class HexGrid:
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


def coords_circle(center: AxialCoord, radius: int):
    # XXX: looping over cube coords, but there might be a better way
    for x in range(-radius, radius + 1):
        dq = x
        for y in range(
            max(-radius, -x - radius), min(radius, -x + radius) + 1
        ):
            dr = 0 - x - y
            yield AxialCoord(center.q + dq, center.r + dr)


def are_coords_adjacent(a: AxialCoord, b: AxialCoord):
    dx = b.q - a.q
    dy = b.r - a.r
    return (dx, dy) in AXIAL_NEIGHBORS
