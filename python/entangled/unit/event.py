from typing import Iterable

from engine.event import CombatEvent, CombatEventEffect, CommandableCombatEvent


class UnitTurnCombatEvent(CommandableCombatEvent):
    def __init__(
        self,
        unit,
        turn_countdown: int = None,
        action_points: int = None,
        priority: int = CombatEvent.PRIORITY_LOW,
    ):
        super(UnitTurnCombatEvent, self).__init__(
            turn_countdown
            if turn_countdown is not None
            else unit.turn_countdown,
            priority,
        )
        self.unit = unit
        self.action_points = (
            action_points if action_points is not None else unit.action_points
        )

    def execute(self, combat) -> Iterable[CombatEventEffect]:
        # UI needs to know that turn started
        return (UnitTurnBeganEffect(self.unit, combat, self.action_points),)

    def affected_units(self):
        return (self.unit,)

    def to_json(self):
        obj = super().to_json()
        obj["action_points"] = self.action_points
        return obj

    def is_done(self):
        return False

    def __repr__(self):
        return "{} for {}".format(super().__repr__(), self.unit)


class UnitTurnBeganEffect(CombatEventEffect):
    def __init__(self, unit, combat, action_points):
        self.unit = unit
        self.move_coords = combat.unit_move_coords(self.unit, action_points)

    def to_json(self):
        obj = super().to_json()
        obj.update(
            {
                "unit_key": self.unit.key(),
                "move_coords": [
                    {
                        "q": coord.q,
                        "r": coord.r,
                        "path": [
                            {"q": path_coord.q, "r": path_coord.r}
                            for path_coord in path
                        ],
                    }
                    for coord, path in self.move_coords
                ],
            }
        )
        return obj
