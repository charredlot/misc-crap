from urllib.parse import unquote
import json
import time

from flask import Blueprint, current_app, render_template

from app.game import get_combat
from combat import combat_json, MoveActiveUnitCommand
from engine.event import effect_json
from level import AxialCoord


bp = Blueprint("argh", __name__, template_folder="templates")


@bp.route("/")
def index():
    return render_template("index.html", time=time.time())


@bp.route("/combat_state")
def combat_state():
    current_app.combat = get_combat()
    return json.dumps(current_app.combat, default=combat_json, indent=2)


@bp.route("/combat_step")
def combat_step():
    curr_event, effects = current_app.combat.step()

    obj = combat_json(current_app.combat)
    obj["curr_event"] = curr_event.to_json()
    obj["effects"] = [effect_json(e) for e in effects]
    return json.dumps(obj, indent=2)


@bp.route("/move_radius/<unit_key>")
def move_radius(unit_key):
    unit_key = unquote(unit_key)
    combat = current_app.combat

    return json.dumps(
        [
            {"q": coord.q, "r": coord.r}
            for coord in combat.unit_move_coords(unit_key)
        ],
        indent=2,
    )


@bp.route("/move_active_unit/<q>/<r>")
def move_active_unit(q, r):
    q = int(q)
    r = int(r)
    combat = current_app.combat
    effects = combat.process_command(MoveActiveUnitCommand(AxialCoord(q, r)))

    obj = combat_json(combat)
    obj["curr_event"] = combat.curr_event.to_json()
    obj["effects"] = [effect_json(e) for e in effects]
    return json.dumps(obj, indent=2)
