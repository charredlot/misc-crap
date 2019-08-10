from typing import Iterable, Optional

import logging

from engine.command import Command
from engine.event import (
    CombatEventBase,
    CombatEventEffect,
    CombatEventQueue,
    CommandableCombatEvent,
)
from level import HexGrid


class CombatDebug:
    print_events = False


class Combat:
    def __init__(self, grid: HexGrid, debug: CombatDebug = None):
        self.grid = grid
        self.debug = debug if debug else CombatDebug()

        self.event_queue = CombatEventQueue()
        self.curr_event: Optional[CombatEventBase] = None

    def step(self) -> Iterable[CombatEventEffect]:
        if self.curr_event and not self.curr_event.is_done():
            raise Exception("{} needs commands".format(self.curr_event))

        event = self.event_queue.pop()
        if self.debug.print_events:
            logging.info(event)

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
