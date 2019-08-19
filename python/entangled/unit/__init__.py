from util import to_json


class Unit:
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
        return "Unit({})".format(self.name)


@to_json.register(Unit)
def unit_json(unit: Unit):
    return {
        key: getattr(unit, key)
        for key in (
            "name",
            "turn_countdown",
            "action_points",
            "friendly",
            "control",
        )
    }
