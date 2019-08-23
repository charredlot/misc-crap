from collections import defaultdict, deque, namedtuple
from typing import Deque, Dict, Iterable, List, Set


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

    def shortest_path(
        self, src: AxialCoord, dst: AxialCoord
    ) -> List[AxialCoord]:
        # from A* search wiki
        best_prev: Dict[AxialCoord, AxialCoord] = {}

        # this is the g(x), cheapest cost to get to node x
        g_score: Dict[AxialCoord, float] = defaultdict(lambda: float("inf"))

        # f(x) = g(x) + h(x) where h is the heuristic
        f_score: Dict[AxialCoord, float] = defaultdict(lambda: float("inf"))

        # with min costs of 1, this should be an admissible heuristic
        def _heuristic(coord: AxialCoord):
            dq = coord.q - src.q
            dr = coord.r - src.r
            return (dq * dq) + (dr * dr)

        g_score[src] = 0
        f_score[src] = _heuristic(src)

        visited = set()

        # heap value is the fscore
        open_set: Set[AxialCoord] = set()
        open_set.add(src)
        while open_set:
            # XXX: not ideal, need a heap that supports removal
            curr = next(iter(sorted(open_set, key=lambda n: f_score[n])))
            open_set.remove(curr)
            if curr == dst:
                # work backwards to get the best path
                q: Deque[AxialCoord] = deque()
                q.append(curr)
                while curr in best_prev:
                    curr = best_prev[curr]
                    q.appendleft(curr)
                return list(q)

            visited.add(curr)
            for neighbor in adjacent_coords(curr):
                if neighbor not in self.tiles:
                    continue

                if neighbor in visited:
                    continue

                tentative_g_score = (
                    g_score[curr]
                    + self.edge_weights[AxialEdge(curr, neighbor)]
                )
                if tentative_g_score < g_score[neighbor]:
                    best_prev[neighbor] = curr
                    g_score[neighbor] = tentative_g_score
                    f_score[neighbor] = tentative_g_score + _heuristic(
                        neighbor
                    )

                open_set.add(neighbor)

        # no path
        return []

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
