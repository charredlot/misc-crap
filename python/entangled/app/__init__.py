from flask import g, Flask

from app import routes
from level import AxialCoord, HexGrid


def create_app(test_config=None):
    app = Flask(__name__, instance_relative_config=True)

    currentGrid = HexGrid((
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
    ))

    with app.app_context():
        app.currentGrid = currentGrid

    app.register_blueprint(routes.bp)

    return app
