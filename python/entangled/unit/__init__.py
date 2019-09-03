from util import to_json


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
    ):
        self.name = name
        self.turn_countdown = turn_countdown
        self.action_points = action_points
        self.friendly = friendly
        self.control = control

    def key(self):
        return self.name

    def __repr__(self):
        return "CombatUnit({})".format(self.name)


@to_json.register(CombatUnit)
def unit_json(unit: CombatUnit):
    obj = {
        key: getattr(unit, key)
        for key in (
            "name",
            "turn_countdown",
            "action_points",
            "friendly",
            "control",
        )
    }
    obj["key"] = unit.key()
    return obj
