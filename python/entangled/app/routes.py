import json

from flask import Blueprint, current_app, render_template

from combat import combat_json


bp = Blueprint("argh", __name__, template_folder="templates")


@bp.route("/")
def index():
    return render_template("index.html")


@bp.route("/combat_state")
def combat_state():
    return json.dumps(current_app.combat, default=combat_json, indent=2)
