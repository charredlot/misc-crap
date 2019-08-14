from typing import Iterable

from level import AxialCoord
from engine.command import Command
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
        return (UnitTurnBeganEffect(self.unit),)

    def execute_command(
        self, combat, command: Command
    ) -> Iterable[CombatEventEffect]:
        return ()

    def is_done(self):
        return False

    def __repr__(self):
        return "{} for {}".format(super().__repr__(), self.unit)


class UnitMoveCommand(Command):
    """
    This is a normal move which is only to an adjacent tile
    """

    def __init__(self, dest: AxialCoord):
        self.dest = dest


class UnitTurnBeganEffect(CombatEventEffect):
    def __init__(self, unit):
        self.unit = unit
