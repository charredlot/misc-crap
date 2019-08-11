from engine.event import CombatEvent, CommandableCombatEvent


class UnitTurnCombatEvent(CommandableCombatEvent):
    def __init__(self, unit, priority: int = CombatEvent.PRIORITY_LOW):
        super(UnitTurnCombatEvent, self).__init__(
            unit.turn_countdown, priority
        )
        self.unit = unit
