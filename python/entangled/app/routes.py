import json

from flask import Blueprint, current_app, g, render_template

from level import hex_grid_json


bp = Blueprint("argh", __name__, template_folder="templates")

@bp.route("/")
def index():
    return render_template("index.html")


@bp.route("/grid")
def grid():
    return json.dumps(current_app.currentGrid, default=hex_grid_json, indent=2)
