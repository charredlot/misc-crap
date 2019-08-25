from unittest import TestCase

from level import AxialCoord, AxialEdge, coords_circle, HexGrid


class TestHexGrid(TestCase):
    def test_edges(self):
        tests = (
            {"vertices": (AxialCoord(0, 0),), "edges": ()},
            {"vertices": (AxialCoord(0, 0), AxialCoord(9, 9)), "edges": ()},
            {
                "vertices": (AxialCoord(0, 0), AxialCoord(1, 0)),
                "edges": (
                    AxialEdge(AxialCoord(q=0, r=0), AxialCoord(q=1, r=0)),
                    AxialEdge(AxialCoord(q=1, r=0), AxialCoord(q=0, r=0)),
                ),
            },
            {
                "vertices": (
                    AxialCoord(0, 0),
                    AxialCoord(1, 0),
                    AxialCoord(1, 1),
                ),
                "edges": (
                    AxialEdge(AxialCoord(q=0, r=0), AxialCoord(q=1, r=0)),
                    AxialEdge(AxialCoord(q=1, r=1), AxialCoord(q=1, r=0)),
                    AxialEdge(AxialCoord(q=1, r=0), AxialCoord(q=1, r=1)),
                    AxialEdge(AxialCoord(q=1, r=0), AxialCoord(q=0, r=0)),
                ),
            },
        )

        for test in tests:
            grid = HexGrid(test["vertices"])
            edges = grid.default_edge_weights()
            self.assertEqual(set(edges.keys()), set(test["edges"]))

    def test_shortest_path(self):
        tests = (
            {
                "vertices": (AxialCoord(0, 0), AxialCoord(1, 0)),
                "source": AxialCoord(0, 0),
                "destination": AxialCoord(1, 0),
                "path": [AxialCoord(0, 0), AxialCoord(1, 0)],
            },
            {
                "vertices": (AxialCoord(0, 0), AxialCoord(2, 3)),
                "source": AxialCoord(0, 0),
                "destination": AxialCoord(2, 3),
                "path": [],
            },
            {
                "vertices": list(coords_circle(AxialCoord(0, 0), 1)),
                "source": AxialCoord(0, -1),
                "destination": AxialCoord(0, 1),
                "path": [
                    AxialCoord(0, -1),
                    AxialCoord(0, 0),
                    AxialCoord(0, 1),
                ],
            },
            {
                "vertices": list(coords_circle(AxialCoord(0, 0), 4)),
                "source": AxialCoord(0, 2),
                "destination": AxialCoord(-1, -1),
                "path": [
                    AxialCoord(0, 2),
                    AxialCoord(0, 1),
                    AxialCoord(0, 0),
                    AxialCoord(0, -1),
                    AxialCoord(-1, -1),
                ],
            },
        )

        for test in tests:
            grid = HexGrid(test["vertices"])
            path = grid.shortest_path(test["source"], test["destination"])
            self.assertEqual(path, test["path"])
