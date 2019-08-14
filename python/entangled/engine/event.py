from abc import ABC, abstractmethod
from typing import Iterable

import heapq

from engine.command import Command


class CombatEventEffect(ABC):
    """
    For the UI to display what actually happened
    """

    pass


class CombatEvent(ABC):
    PRIORITY_LOW = 10
    PRIORITY_MEDIUM = 20
    PRIORITY_HIGH = 30

    def __init__(self, countdown: int, priority: int = PRIORITY_MEDIUM):
        self.countdown = countdown
        self.priority = priority

    @abstractmethod
    def execute(self, combat) -> Iterable[CombatEventEffect]:
        """
        @param combat: type Combat, but avoid circular import for now
        """
        pass

    def affected_units(self):
        """
        Mostly for the UI to display things

        @returns: an iterable of Unit that are affected by the events
        """
        return ()

    def is_done(self) -> bool:
        return True

    def __repr__(self):
        return "{}(countdown={}, priority={})".format(
            type(self).__name__, self.countdown, self.priority
        )


class CommandableCombatEvent(CombatEvent):
    """
    Represents a unit's turn that requires input commands.
    """

    def __init__(
        self, unit, countdown: int, priority: int = CombatEvent.PRIORITY_MEDIUM
    ):
        super(CommandableCombatEvent, self).__init__(countdown, priority)
        self.unit = unit

    @abstractmethod
    def execute_command(
        self, combat, command: Command
    ) -> Iterable[CombatEventEffect]:
        """
        @param combat: type Combat, but avoid circular import for now
        """
        pass

    def is_done(self) -> bool:
        # execute_command should make this eventually return True
        return False


# using heapq makes changing the key value O(n) instead of O(log n) because the
# api only lets us do heapify...but we're not going to have that many events
class CombatEventQueue:
    def __init__(self):
        self.events = []
        self.counter = 0
        self.timestamp = 0

    def push(self, event: CombatEvent):
        # the heap key is (timestamp, priority, counter, event) because
        # we can't do a custom comparator with python heapq
        # we use the counter to break ties so that insert order is preserved
        timestamp = self.timestamp + event.countdown

        self.counter += 1
        heap_item = (timestamp, -event.priority, self.counter, event)

        heapq.heappush(self.events, heap_item)

    def push_all(self, events):
        for event in events:
            self.push(event)

    def pop(self) -> CombatEvent:
        # timestamp moves on every pop event
        timestamp, _, _, event = heapq.heappop(self.events)
        self.timestamp = timestamp
        return event

    def __iter__(self):
        # make copy of list
        events = list(self.events)
        while events:
            t = heapq.heappop(events)
            yield t[-1]
