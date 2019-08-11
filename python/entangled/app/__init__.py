from flask import Flask

from app import routes
from combat import Combat
from level import AxialCoord, HexGrid
from unit import Unit


def create_app(test_config=None):
    app = Flask(__name__, instance_relative_config=True)

    grid = HexGrid(
        (
            AxialCoord(-3, -2),
            AxialCoord(-3, 0),
            AxialCoord(-2, -4),
            AxialCoord(-2, -3),
            AxialCoord(-2, 0),
            AxialCoord(-2, 1),
            AxialCoord(-2, 2),
            AxialCoord(-1, -4),
            AxialCoord(-1, -3),
            AxialCoord(-1, 0),
            AxialCoord(-1, 1),
            AxialCoord(-1, 2),
            AxialCoord(0, -4),
            AxialCoord(0, -3),
            AxialCoord(0, -2),
            AxialCoord(0, -1),
            AxialCoord(0, 0),
            AxialCoord(0, 1),
            AxialCoord(0, 2),
            AxialCoord(0, 3),
            AxialCoord(1, -2),
            AxialCoord(1, -1),
            AxialCoord(1, 0),
            AxialCoord(1, 1),
            AxialCoord(1, 2),
            AxialCoord(1, 3),
            AxialCoord(2, -4),
            AxialCoord(2, -3),
            AxialCoord(2, -2),
            AxialCoord(2, 0),
            AxialCoord(2, 1),
            AxialCoord(2, 2),
            AxialCoord(3, -5),
            AxialCoord(3, -4),
            AxialCoord(3, -3),
        )
    )
    combat = Combat(grid)
    combat.debug.print_events = True

    combat.place_unit(
        Unit("P1", friendly=True, control=Unit.PLAYER_CONTROL),
        AxialCoord(2, 2),
    )
    combat.place_unit(
        Unit("P2", friendly=True, control=Unit.PLAYER_CONTROL),
        AxialCoord(1, 3),
    )

    for i, coord in enumerate(
        (
            AxialCoord(-1, -3),
            AxialCoord(-2, -4),
            AxialCoord(-2, -3),
            AxialCoord(3, -4),
        )
    ):
        combat.place_unit(Unit("Mook {}".format(i)), coord)

    with app.app_context():
        app.combat = combat

    app.register_blueprint(routes.bp)

    return app
