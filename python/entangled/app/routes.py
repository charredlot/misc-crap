import json
import time

from flask import Blueprint, current_app, render_template

from app.game import get_combat
from combat import combat_json
from engine.event import effect_json, event_json


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
    obj["curr_event"] = event_json(curr_event)
    obj["effects"] = [effect_json(e) for e in effects]
    return json.dumps(obj, indent=2)
