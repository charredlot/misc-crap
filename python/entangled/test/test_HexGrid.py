from unittest import TestCase

from level import AxialCoord, AxialEdge, HexGrid


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
