from urllib.parse import unquote
import json
import time

from flask import Blueprint, current_app, render_template

from app.game import get_combat
from combat import combat_json
from engine.event import effect_json
from level import coords_circle


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
    center = current_app.combat.unit_key_to_coord[unit_key]
    next_turn = current_app.combat.unit_key_to_next_turn[unit_key]
    combat = current_app.combat

    return json.dumps(
        [
            {"q": coord.q, "r": coord.r}
            for coord in coords_circle(center, next_turn.action_points)
            if (coord in combat.grid)
            and (coord not in combat.coord_to_unit_key)
        ],
        indent=2,
    )
