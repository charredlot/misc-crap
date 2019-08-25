from abc import abstractmethod
from typing import Dict, Iterable, List, Optional, Tuple

import logging

from engine.command import Command
from engine.event import (
    CombatEvent,
    CombatEventEffect,
    CombatEventQueue,
    CommandableCombatEvent,
    ErrorEffect,
)
from level import AxialCoord, coords_circle, HexGrid
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

        self.unit_key_to_coord: Dict[str, AxialCoord] = {}
        self.coord_to_unit_key: Dict[AxialCoord, str] = {}

        self.unit_key_to_next_turn: Dict[str, UnitTurnCombatEvent] = {}

    def step(self) -> Iterable[CombatEventEffect]:
        if self.curr_event and not self.curr_event.is_done():
            raise Exception("{} needs commands".format(self.curr_event))

        event = self.event_queue.pop()
        if self.debug.print_events:
            logging.info("Pop and execute event: %s", event)

        effects = event.execute(self)

        self.curr_event = event
        return effects

    def process_command(
        self, command: "CombatCommand"
    ) -> Iterable[CombatEventEffect]:
        # XXX: this needs to be single-threaded with step, add lock later
        if not self.curr_event:
            logging.error("No event expecting commands: {}", command)
            return ()

        if not isinstance(self.curr_event, CommandableCombatEvent):
            logging.error(
                "Event {} doesn't need commands: {}", self.curr_event, command
            )
            return ()

        return command.apply(self)

    def place_unit(self, unit: Unit, coord: AxialCoord):
        unit_key = unit.key()
        self.units[unit_key] = unit

        if unit_key in self.unit_key_to_coord:
            raise Exception(
                "unit {} is already at {}".format(
                    unit_key, self.unit_key_to_coord[unit_key]
                )
            )

        if coord in self.coord_to_unit_key:
            raise Exception(
                "another unit {} already at {}".format(
                    self.coord_to_unit_key[coord], coord
                )
            )

        self.unit_key_to_coord[unit_key] = coord
        self.coord_to_unit_key[coord] = unit_key

        self.push_turn_event(unit, UnitTurnCombatEvent(unit))

    def push_turn_event(self, unit: Unit, turn: UnitTurnCombatEvent):
        self.unit_key_to_next_turn[unit.key()] = turn
        self.push_event(turn)

    def push_event(self, event: CombatEvent):
        if self.debug.print_events:
            logging.info("Push event: %s", event)

        self.event_queue.push(event)

    def unit_move_coords(
        self, unit_key: str
    ) -> Iterable[Tuple[AxialCoord, List[AxialCoord]]]:
        # tuple is coord and the shortest path from the unit to
        # the coord
        center = self.unit_key_to_coord[unit_key]
        next_turn = self.unit_key_to_next_turn[unit_key]
        # FIXME: this is actually broken because we need to know the action
        # point cost to make sure some of these paths make sense.
        return [
            (coord, path)
            for coord, path in (
                (coord, self.grid.shortest_path(center, coord))
                for coord in coords_circle(center, next_turn.action_points)
                if coord != center
            )
            if path
        ]


@to_json.register(Combat)
def combat_json(combat):
    return {
        "units": {key: unit_json(unit) for key, unit in combat.units.items()},
        "tiles": [
            {
                "q": coord.q,
                "r": coord.r,
                "unit_key": combat.coord_to_unit_key[coord]
                if coord in combat.coord_to_unit_key
                else None,
            }
            for coord, tile in combat.grid.tiles.items()
        ],
        "unit_key_to_coord": {
            unit_key: {"q": coord.q, "r": coord.r}
            for unit_key, coord in combat.unit_key_to_coord.items()
        },
        "events": [e.to_json() for e in combat.event_queue],
        "curr_event": combat.curr_event.to_json()
        if combat.curr_event
        else None,
    }


class CombatCommand(Command):
    @abstractmethod
    def apply(self, combat: Combat) -> Iterable[CombatEventEffect]:
        return ()


class MoveActiveUnitCommand(CombatCommand):
    """
    This is a normal move
    """

    def __init__(self, dest: AxialCoord):
        self.dest = dest

    def apply(self, combat: Combat) -> Iterable[CombatEventEffect]:
        if not isinstance(combat.curr_event, UnitTurnCombatEvent):
            return (
                ErrorEffect(
                    "{} is not a UnitTurnCombatEvent".format(combat.curr_event)
                ),
            )

        dest_tile = combat.grid.get(self.dest)
        if not dest_tile:
            return (ErrorEffect("{} not in grid".format(self.dest)),)

        # unit = self.curr_event.unit
        # src_tile = combat.unit_key_to_coord.get[unit.key()]
        # XXX: do path stuff here
        return ()
