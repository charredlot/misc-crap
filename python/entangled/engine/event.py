from abc import ABC, abstractmethod

import heapq


class CombatEventBase(ABC):
    PRIORITY_LOW = 10
    PRIORITY_MEDIUM = 20
    PRIORITY_HIGH = 30

    def __init__(self, countdown: int, priority: int = PRIORITY_MEDIUM):
        self.countdown = countdown
        self.priority = priority

    def __repr__(self):
        return "{}(countdown={}, priority={})".format(
            type(self), self.countdown, self.priority
        )


class CombatEvent(CombatEventBase):
    pass


# using heapq makes changing the key value O(n) instead of O(log n) because the
# api only lets us do heapify...but we're not going to have that many events
class CombatEventQueue:
    def __init__(self):
        self.events = []
        self.counter = 0
        self.timestamp = 0

    def push(self, event: CombatEventBase):
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

    def pop(self) -> CombatEventBase:
        # timestamp moves on every pop event
        timestamp, _, _, event = heapq.heappop(self.events)
        self.timestamp = timestamp
        return event
