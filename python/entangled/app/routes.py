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


def _combat_step_json(combat, effects=None):
    obj = combat_json(current_app.combat)
    if effects is not None:
        obj["effects"] = [effect_json(e) for e in effects]
    return json.dumps(obj, indent=2)


@bp.route("/combat_state")
def combat_state():
    current_app.combat = get_combat()
    return _combat_step_json(current_app.combat)


@bp.route("/combat_step")
def combat_step():
    effects = current_app.combat.step()
    return _combat_step_json(current_app.combat, effects)


@bp.route("/move_coords/<unit_key>")
def move_coords(unit_key):
    unit_key = unquote(unit_key)
    combat = current_app.combat

    return json.dumps(
        [
            {
                "q": coord.q,
                "r": coord.r,
                "path": [
                    {"q": path_coord.q, "r": path_coord.r}
                    for path_coord in path
                ],
            }
            for coord, path in combat.unit_move_coords(unit_key)
        ],
        indent=2,
    )


@bp.route("/move_active_unit/<q>/<r>")
def move_active_unit(q, r):
    q = int(q)
    r = int(r)
    combat = current_app.combat
    effects = combat.process_command(MoveActiveUnitCommand(AxialCoord(q, r)))

    return _combat_step_json(combat, effects)
