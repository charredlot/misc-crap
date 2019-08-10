from random import sample
from unittest import TestCase

from engine.event import CombatEvent, CombatEventQueue


class TestCombatEventQueue(TestCase):
    def _test_order(self, q: CombatEventQueue, events_in_order):
        for expected in events_in_order:
            popped = q.pop()
            self.assertEqual(popped, expected)

    def test_push(self):
        events = (
            CombatEvent(countdown=3),
            CombatEvent(countdown=1),
            CombatEvent(countdown=2),
        )
        q = CombatEventQueue()
        q.push_all(events)

        self._test_order(q, sorted(events, key=lambda e: e.countdown))

    def test_push_pop_push(self):
        events = [
            CombatEvent(countdown=3),
            CombatEvent(countdown=2),
            CombatEvent(countdown=1),
        ]
        q = CombatEventQueue()
        q.push_all(events)

        popped = q.pop()
        self.assertEqual(popped, events[-1])
        events = events[:-1]

        additional = [
            CombatEvent(countdown=1),
            CombatEvent(countdown=9),
        ]
        q.push_all(additional)

        events += additional
        self._test_order(q, sorted(events, key=lambda e: e.countdown))

    def test_priority_secondary(self):
        ordered = (
            CombatEvent(countdown=1, priority=CombatEvent.PRIORITY_LOW),
            CombatEvent(countdown=2, priority=CombatEvent.PRIORITY_HIGH),
            CombatEvent(countdown=3),
        )
        q = CombatEventQueue()
        q.push_all(sample(ordered, len(ordered)))

        self._test_order(q, ordered)

    def test_priority_breaks_tie(self):
        ordered = (
            CombatEvent(countdown=1, priority=CombatEvent.PRIORITY_HIGH),
            CombatEvent(countdown=1, priority=CombatEvent.PRIORITY_MEDIUM),
            CombatEvent(countdown=1, priority=CombatEvent.PRIORITY_LOW),
            CombatEvent(countdown=1000),
        )
        q = CombatEventQueue()
        q.push_all(sample(ordered, len(ordered)))

        self._test_order(q, ordered)

    def test_stability(self):
        # order of insert should be preserved on pop
        ordered = (
            CombatEvent(countdown=3, priority=CombatEvent.PRIORITY_MEDIUM),
            CombatEvent(countdown=8, priority=CombatEvent.PRIORITY_HIGH),
            CombatEvent(countdown=8, priority=CombatEvent.PRIORITY_HIGH),
            CombatEvent(countdown=8, priority=CombatEvent.PRIORITY_HIGH),
            CombatEvent(countdown=8, priority=CombatEvent.PRIORITY_HIGH),
            CombatEvent(countdown=12, priority=CombatEvent.PRIORITY_LOW),
        )
        q = CombatEventQueue()
        q.push_all(ordered)

        self._test_order(q, ordered)
