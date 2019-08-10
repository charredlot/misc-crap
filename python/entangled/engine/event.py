from abc import ABC, abstractmethod

import heapq


class CombatEventBase(ABC):
    PRIORITY_LOW = 1
    PRIORITY_MEDIUM = 2
    PRIORITY_HIGH = 3

    def __init__(self, countdown: int, priority: int=PRIORITY_MEDIUM):
        self.countdown = countdown
        self.priority = priority

    def min_heap_key(self):
        # smallest countdown goes first, highest priority goes first
        return (self.countdown, -self.priority)

    def __repr__(self):
        return "CombatEvent(countdown={}, priority={})".format(
            self.countdown, self.priority
        )


class CombatEvent(CombatEventBase):
    pass


# using heapq makes changing the key value O(n) instead of O(log n) because the
# api only lets us do heapify...but we're not going to have that many events
class CombatEventQueue:
    def __init__(self):
        self.events = []
        self.counter = 0

    def push(self, event: CombatEvent):
        # if heap_key is (a, b), then the real heap key becomes
        # the tuple (a, b, counter, event) because python heapq is weird
        # we use the counter to break ties so that insert order is preserved
        self.counter += 1
        heap_key = event.min_heap_key()
        try:
            heap_item = (*heap_key, self.counter, event)
        except TypeError:
            # key is not a tuple
            heap_item = (heap_key, self.counter, event)

        heapq.heappush(self.events, heap_item)

    def push_all(self, events):
        for event in events:
            self.push(event)

    def pop(self):
        # event is always the last thing in the tuple
        heap_item = heapq.heappop(self.events)
        return heap_item[-1]
