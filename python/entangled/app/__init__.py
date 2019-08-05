
from flask import g, Flask

from app import routes
from level import AxialCoord, coords_circle, hex_grid_json, HexGrid, HexTile


def create_app(test_config=None):
    app = Flask(__name__, instance_relative_config=True)

    currentGrid = HexGrid()

    for coord in coords_circle(AxialCoord(0, 0), 3):
        currentGrid.add(HexTile(coord))

    with app.app_context():
        app.currentGrid = currentGrid

    app.register_blueprint(routes.bp)

    return app
