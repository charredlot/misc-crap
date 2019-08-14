from typing import Dict, Iterable, Optional

import logging

from engine.command import Command
from engine.event import (
    CombatEvent,
    CombatEventEffect,
    CombatEventQueue,
    CommandableCombatEvent,
)
from level import AxialCoord, HexGrid
from unit import Unit, unit_json
from unit.event import UnitTurnCombatEvent
from util import to_json


class CombatDebug:
    print_events = False


class Combat:
    def __init__(self, grid: HexGrid, debug: CombatDebug = None):
        self.grid = grid
        self.debug = debug if debug else CombatDebug()

        self.event_queue = CombatEventQueue()
        self.curr_event: Optional[CombatEvent] = None

        self.units: Dict[str, Unit] = {}

    def step(self) -> Iterable[CombatEventEffect]:
        if self.curr_event and not self.curr_event.is_done():
            raise Exception("{} needs commands".format(self.curr_event))

        event = self.event_queue.pop()
        if self.debug.print_events:
            logging.info("Pop and execute event: %s", event)

        effects = event.execute(self)

        self.curr_event = event
        return effects

    def process_command(self, command: Command) -> Iterable[CombatEventEffect]:
        # XXX: this needs to be single-threaded with step, add lock later
        if not self.curr_event:
            logging.error("No event expecting commands: {}", command)
            return ()

        if not isinstance(self.curr_event, CommandableCombatEvent):
            logging.error(
                "Event {} doesn't need commands: {}", self.curr_event, command
            )
            return ()

        return self.curr_event.execute_command(self, command)

    def place_unit(self, unit: Unit, coord: AxialCoord):
        self.units[unit.name] = unit

        tile = self.grid.tiles[coord]
        tile.unit = unit
        unit.tile = tile

        self.push_event(UnitTurnCombatEvent(unit))

    def push_event(self, event: CombatEvent):
        if self.debug.print_events:
            logging.info("Push event: %s", event)

        self.event_queue.push(event)


@to_json.register(Combat)
def combat_json(combat):
    return {
        "units": {
            name: unit_json(unit) for name, unit in combat.units.items()
        },
        "tiles": [
            {
                "q": coord.q,
                "r": coord.r,
                "unit_name": tile.unit.name if tile.unit else None,
            }
            for coord, tile in combat.grid.tiles.items()
        ],
    }
