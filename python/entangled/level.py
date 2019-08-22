from collections import namedtuple
from typing import Dict, Iterable


AxialCoord = namedtuple("AxialCoord", ["q", "r"])
AxialEdge = namedtuple("AxialEdge", ["src", "dst"])

AXIAL_NEIGHBORS = frozenset(
    ((0, -1), (1, -1), (1, 0), (0, 1), (-1, 1), (-1, 0))
)


class HexTile:
    def __init__(self, coord: AxialCoord):
        self.coord = coord

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
            self.edge_weights = self.default_edge_weights()
        self._min_q = None
        self._max_r = None
        self._min_r = None
        self._max_r = None

    def default_edge_weights(self) -> Dict[AxialEdge, int]:
        edge_weights = {}
        for tile in self.tiles.values():
            for coord in adjacent_coords(tile.coord):
                if coord in self.tiles:
                    edge_weights[AxialEdge(tile.coord, coord)] = 1
        return edge_weights

    def add(self, tile: HexTile):
        for neighbor in adjacent_coords(tile.coord):
            if neighbor in self.tiles:
                self.edge_weights[AxialEdge(tile.coord, neighbor)] = 1
                self.edge_weights[AxialEdge(neighbor, tile.coord)] = 1
        self.tiles[tile.coord] = tile

    def get(self, coord: AxialCoord) -> HexTile:
        return self.tiles.get(coord)

    def __getitem__(self, coord: AxialCoord):
        return self.tiles[coord]

    def __contains__(self, coord: AxialCoord):
        return coord in self.tiles

    def __repr__(self):
        return "\n".join(str(coord) for coord in self.tiles.keys())


def coords_circle(center: AxialCoord, radius: int):
    # https://www.redblobgames.com/grids/hexagons/
    # XXX: looping over cube coords, but there might be a better way
    for x in range(-radius, radius + 1):
        dq = x
        for y in range(
            max(-radius, -x - radius), min(radius, -x + radius) + 1
        ):
            dr = 0 - x - y
            yield AxialCoord(center.q + dq, center.r + dr)


def are_coords_adjacent(a: AxialCoord, b: AxialCoord):
    dq = b.q - a.q
    dr = b.r - a.r
    return (dq, dr) in AXIAL_NEIGHBORS


def adjacent_coords(center: AxialCoord) -> Iterable[AxialCoord]:
    for dq, dr in AXIAL_NEIGHBORS:
        yield AxialCoord(center.q + dq, center.r + dr)
