from collections import defaultdict, deque, namedtuple
from math import gcd, sqrt
from typing import Deque, Dict, Iterable, List, Optional, Set, Tuple


AxialCoord = namedtuple("AxialCoord", ["q", "r"])
AxialDir = namedtuple("AxialDir", ["dq", "dr"])
AxialEdge = namedtuple("AxialEdge", ["src", "dst"])

AXIAL_NEIGHBORS = frozenset(
    ((0, -1), (1, -1), (1, 0), (0, 1), (-1, 1), (-1, 0))
)

_SQRT3 = sqrt(3)
_HALF_SQRT3 = _SQRT3 / 2.0


class HexTile:
    def __init__(self, coord: AxialCoord):
        self.coord = coord

    def __hash__(self):
        return hash(self.coord)

    def __repr__(self):
        return "(q={}, r={})".format(self.coord.q, self.coord.r)


class HexGrid:
    def __init__(self, coords=None, default_edge_weight=1):
        self._default_edge_weight = 1
        if not coords:
            self.tiles = dict()
        else:
            self.tiles = {coord: HexTile(coord) for coord in coords}
            # neighbors are adjacent, by default
            self.adjacencies = {
                coord: {
                    neighbor: self._default_edge_weight
                    for neighbor in neighbors_coords(coord)
                    if neighbor in self.tiles
                }
                for coord in self.tiles
            }
        self._min_q = None
        self._max_r = None
        self._min_r = None
        self._max_r = None

    def adjacent_coords(self, center: AxialCoord) -> Iterable[AxialCoord]:
        adjacent = self.adjacencies.get(center)
        if not adjacent:
            return []

        return list(adjacent)

    def get_edge_weight(
        self, src: AxialCoord, dst: AxialCoord
    ) -> Optional[int]:
        try:
            return self.adjacencies[src][dst]
        except KeyError:
            return None

    def edges(self):
        # dunno if this will be useful
        return (
            AxialEdge(coord, neighbor)
            for coord, adjacent_coords in self.adjacencies.items()
            for neighbor in adjacent_coords
        )

    def shortest_path(
        self, src: AxialCoord, dst: AxialCoord
    ) -> List[AxialCoord]:
        # from A* search wiki
        best_prev: Dict[AxialCoord, AxialCoord] = {}

        # this is the g(x), cheapest cost to get to node x
        g_score: Dict[AxialCoord, float] = defaultdict(lambda: float("inf"))

        # f(x) = g(x) + h(x) where h is the heuristic
        f_score: Dict[AxialCoord, float] = defaultdict(lambda: float("inf"))

        # with min costs of 1, this should be an admissible heuristic. need to
        # calculate the distance if we drew it, not using the coords because
        # they don't correspond to distance exactly
        # we'll add an extra 0.1 if it's not in a straight line on the current
        # best path.
        dst_x, dst_y = axial_to_cartesian(dst)

        def _heuristic(coord: AxialCoord):
            x, y = axial_to_cartesian(coord)
            dx = x - dst_x
            dy = y - dst_y
            # use straight line distance / 2 to give us some headroom
            return sqrt((dx * dx) + (dy * dy)) / 2.0

        g_score[src] = 0
        f_score[src] = 0

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

            best_dir = None
            curr_predecessor = best_prev.get(curr)
            if curr_predecessor:
                best_dir = direction_to(curr_predecessor, curr)

            visited.add(curr)
            for neighbor, weight in self.adjacencies.get(curr, {}).items():
                if neighbor not in self.tiles:
                    continue

                if neighbor in visited:
                    continue

                tentative_g_score = g_score[curr] + weight
                if tentative_g_score < g_score[neighbor]:
                    best_prev[neighbor] = curr
                    g_score[neighbor] = tentative_g_score

                    h_score = _heuristic(neighbor)
                    if best_dir:
                        curr_dir = direction_to(curr, neighbor)
                        if curr_dir != best_dir:
                            # this is a bit hacky, but we're just trying to
                            # prefer keeping in the same direction without
                            # biasing towards unoptimal paths
                            h_score *= 1.5
                    f_score[neighbor] = tentative_g_score + h_score

                open_set.add(neighbor)

        # no path
        return []

    def add(self, tile: HexTile):
        self.adjacencies[tile.coord] = {
            neighbor: self._default_edge_weight
            for neighbor in neighbors_coords(tile.coord)
            if neighbor in self.tiles
        }
        for neighbor in neighbors_coords(tile.coord):
            if neighbor not in self.tiles:
                continue

            adjacent = self.adjacencies.setdefault(neighbor, {})
            adjacent[tile.coord] = self._default_edge_weight

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


def are_coords_neighbors(a: AxialCoord, b: AxialCoord) -> bool:
    dq = b.q - a.q
    dr = b.r - a.r
    return (dq, dr) in AXIAL_NEIGHBORS


def neighbors_coords(center: AxialCoord) -> Iterable[AxialCoord]:
    for dq, dr in AXIAL_NEIGHBORS:
        yield AxialCoord(center.q + dq, center.r + dr)


def direction_to(src: AxialCoord, dst: AxialCoord) -> AxialDir:
    """
    Returns the direction in units of integral hexes.
    i.e. this doesn't normalize, so it's not a unit vector
    """
    dq = dst.q - src.q
    dr = dst.r - src.r
    divisor = gcd(dq, dr)
    return AxialDir(dq // divisor, dr // divisor)


def axial_to_cartesian(coord: AxialCoord) -> Tuple[float, float]:
    # radius for a hex is center to vertex distance. the distance between two
    # adjancent hexes is 2h, where h is the height of the equilateral triangle
    # h = radius * sqrt(3) / 2
    # => if we want 2h = 1, radius = 1 / sqrt(3)
    #
    # https://www.redblobgames.com/grids/hexagons/
    x = coord.q + (coord.r / 2)
    y = (coord.r * 3 / 2) / _SQRT3
    return (x, y)


def axial_json(coord: AxialCoord) -> Dict[str, int]:
    return {"q": coord.q, "r": coord.r}
