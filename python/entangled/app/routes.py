import json
import time

from flask import Blueprint, current_app, render_template, request, Response

from app.game import get_combat
from combat import combat_json, MoveActiveUnitCommand
from engine.event import effect_json
from level import AxialCoord


bp = Blueprint("argh", __name__, template_folder="templates")


@bp.route("/")
def index():
    return render_template("index.html", time=time.time())


def _combat_step_json(combat, effects=None):
    obj = combat_json(current_app.combat)
    if effects is not None:
        print(effects)
        obj["effects"] = [effect_json(e) for e in effects]
    return Response(json.dumps(obj, indent=2), mimetype="application/json")


@bp.route("/combat_state")
def combat_state():
    current_app.combat = get_combat()
    return _combat_step_json(current_app.combat)


@bp.route("/combat_step")
def combat_step():
    effects = current_app.combat.step()
    return _combat_step_json(current_app.combat, effects)


@bp.route("/move_active_unit", methods=["POST"])
def move_active_unit():
    path = [
        AxialCoord(coord["q"], coord["r"]) for coord in request.json["path"]
    ]
    combat = current_app.combat
    effects = combat.process_command(MoveActiveUnitCommand(path))

    return _combat_step_json(combat, effects)
