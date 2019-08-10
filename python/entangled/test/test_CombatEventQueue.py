from random import sample
from unittest import TestCase

from engine.event import CombatEvent, CombatEventQueue


class CombatEventTest(CombatEvent):
    def execute(self, combat):
        return ()


class TestCombatEventQueue(TestCase):
    def _test_order(self, q: CombatEventQueue, events_in_order):
        for expected in events_in_order:
            popped = q.pop()
            self.assertEqual(popped, expected)

    def test_push(self):
        events = (
            CombatEventTest(countdown=3),
            CombatEventTest(countdown=1),
            CombatEventTest(countdown=2),
        )
        q = CombatEventQueue()
        q.push_all(events)

        self._test_order(q, sorted(events, key=lambda e: e.countdown))

    def test_advance_time(self):
        first = CombatEventTest(countdown=2)
        initial = [CombatEventTest(countdown=3), CombatEventTest(countdown=7)]
        q = CombatEventQueue()
        q.push_all(sample(initial, len(initial)))

        q.push(first)
        popped = q.pop()
        self.assertEqual(first, popped)

        # popping time should make the starting point 2
        additional = (
            CombatEventTest(countdown=2),
            CombatEventTest(countdown=3),
        )
        q.push_all(additional)

        events = (initial[0], *additional, initial[-1])
        self._test_order(q, events)

    def test_priority_secondary(self):
        ordered = (
            CombatEventTest(countdown=1, priority=CombatEvent.PRIORITY_LOW),
            CombatEventTest(countdown=2, priority=CombatEvent.PRIORITY_HIGH),
            CombatEventTest(countdown=3),
        )
        q = CombatEventQueue()
        q.push_all(sample(ordered, len(ordered)))

        self._test_order(q, ordered)

    def test_priority_breaks_tie(self):
        ordered = (
            CombatEventTest(countdown=1, priority=CombatEvent.PRIORITY_HIGH),
            CombatEventTest(countdown=1, priority=CombatEvent.PRIORITY_MEDIUM),
            CombatEventTest(countdown=1, priority=CombatEvent.PRIORITY_LOW),
            CombatEventTest(countdown=1000),
        )
        q = CombatEventQueue()
        q.push_all(sample(ordered, len(ordered)))

        self._test_order(q, ordered)

    def test_stability(self):
        # order of insert should be preserved on pop
        ordered = (
            CombatEventTest(countdown=3, priority=CombatEvent.PRIORITY_MEDIUM),
            CombatEventTest(countdown=8, priority=CombatEvent.PRIORITY_HIGH),
            CombatEventTest(countdown=8, priority=CombatEvent.PRIORITY_HIGH),
            CombatEventTest(countdown=8, priority=CombatEvent.PRIORITY_HIGH),
            CombatEventTest(countdown=8, priority=CombatEvent.PRIORITY_HIGH),
            CombatEventTest(countdown=12, priority=CombatEvent.PRIORITY_LOW),
        )
        q = CombatEventQueue()
        q.push_all(ordered)

        self._test_order(q, ordered)
