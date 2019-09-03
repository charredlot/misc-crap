class CombatUnit:
    """
    Represents the unit's state for the current combat
    """

    PLAYER_CONTROL = 1
    CPU_CONTROL = 2

    def __init__(
        self,
        name: str,
        turn_countdown: int = 100,
        action_points: int = 6,
        control: int = CPU_CONTROL,
        friendly=False,
        actions: "ActionList" = None,
    ):
        self.name = name
        self.turn_countdown = turn_countdown
        self.action_points = action_points
        self.friendly = friendly
        self.control = control
        self.actions = actions

    def key(self):
        return self.name

    def __repr__(self):
        return "CombatUnit({})".format(self.name)

    def to_json(self, combat):
        obj = {
            key: getattr(self, key)
            for key in (
                "name",
                "turn_countdown",
                "action_points",
                "friendly",
                "control",
            )
        }
        obj["key"] = self.key()
        obj["actions"] = (
            self.actions.to_json(self, combat) if self.actions else {}
        )
        return obj


def action_json(action, unit, combat):
    unit_coord = combat.unit_key_to_coord[unit.key()]
    targetable = [coord for coord in action.targetable(unit_coord, combat)]
    aoe_for_coords = [
        {"q": coord.q, "r": coord.r, "aoe": list(action.aoe(coord, combat))}
        for coord in targetable
    ]
    return {"key": action.key, "targetable": targetable, "aoe": aoe_for_coords}


class ActionList:
    def __init__(self, top_level=None, folders=None):
        """
        @param top_level: iterable of CombatAction
        @param folders: dict of folder name to list of CombatAction
        """

        self.top_level = list(top_level) if top_level else ()
        self.folders = folders

    def to_json(self, unit, combat):
        obj = {
            "folders": {
                folder: [
                    action_json(action, unit, combat) for action in actions
                ]
                for folder, actions in self.folders.items()
            }
        }
        obj["top_level"] = [
            action_json(action, unit, combat) for action in self.top_level
        ]
        return obj
