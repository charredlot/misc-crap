use wasm_bindgen::prelude::*;

use std::collections::{BTreeMap};

use crate::game::battle::{Battle};
use crate::game::battle::input::{Input};

#[wasm_bindgen]
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, PartialOrd, Ord)]
pub enum Priority {
    /* order matters for Ord */
    Immediate = 1,
    BeforeTurn = 10,
    Turn = 20,
    AfterTurn = 30,
}

pub type EventTime = u64;

#[wasm_bindgen]
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, PartialOrd, Ord)]
pub struct EventOrder {
    /* order of fields matters for Ord */
    pub time: EventTime,
    pub priority: Priority,
    pub ctr: u64, /* ctr only to break ties */
}

pub type EventKey = EventOrder;

impl EventKey {
    /* makes implementing test stuff easier */
    pub fn zero() -> EventKey {
        EventKey{time: 0, priority: Priority::Immediate, ctr: 0}
    }
}

pub struct BoxEvent {
    pub event: Box<Event>,
}

pub trait Event {
    fn needs_input(&self) -> bool;
    fn activate(&self, battle: &mut Battle, input: Option<Box<dyn Input>>);

    /* some boilerplate but probably worth */
    fn event_key(&self) -> EventKey;
    fn set_event_key(&mut self, key: EventKey);
}

pub struct EventQueue {
    q: BTreeMap<EventOrder, Box<dyn Event>>,
    time: u64,
    ctr: u64,
}

impl EventQueue {
    pub fn new() -> EventQueue {
        EventQueue{q: BTreeMap::new(), time: 0, ctr: 0}
    }

    /* the return value will always be unique because BTreeMap requires it */
    pub fn insert(&mut self,
                  offset: EventTime,
                  priority: Priority,
                  mut event: Box<dyn Event>) -> EventOrder {
        self.ctr += 1;
        let order = EventOrder{
            time: self.time + offset,
            priority: priority,
            ctr: self.ctr,
        };
        event.set_event_key(order as EventKey);
        self.q.insert(order, event);

        return order;
    }

    pub fn advance(&mut self) -> (EventOrder, Box<dyn Event>) {
        /* XXX: there doesn't seem to be a better way to pop */
        let order = self.q.keys().next().unwrap().clone();
        /* should be a hard error if advance is called while empty */
        let event = self.q.remove(&order).unwrap();
        self.time = order.time;
        return (order, event);
    }
}

#[wasm_bindgen]
#[derive(Clone, Copy, Debug)]
pub struct NullEvent {
    pub key: EventKey,
}

impl NullEvent {
    pub fn new() -> NullEvent {
        NullEvent{
            key: EventKey{time: 0, priority: Priority::Immediate, ctr: 0},
        }
    }
}

impl Event for NullEvent {
    fn needs_input(&self) -> bool { return false; }

    fn activate(&self, battle: &mut Battle, input: Option<Box<dyn Input>>) {}

    fn event_key(&self) -> EventKey { self.key }

    fn set_event_key(&mut self, key: EventKey) { self.key = key; }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_order() {
        let lt = [
            (EventOrder{time: 1, priority: Priority::Turn, ctr: 2},
             EventOrder{time: 7, priority: Priority::Turn, ctr: 1}),
            (EventOrder{time: 3, priority: Priority::BeforeTurn, ctr: 2},
             EventOrder{time: 3, priority: Priority::Turn, ctr: 1}),
            (EventOrder{time: 3, priority: Priority::Turn, ctr: 2},
             EventOrder{time: 3, priority: Priority::AfterTurn, ctr: 1}),
            (EventOrder{time: 3, priority: Priority::BeforeTurn, ctr: 2},
             EventOrder{time: 3, priority: Priority::AfterTurn, ctr: 1}),
            (EventOrder{time: 3, priority: Priority::Turn, ctr: 1},
             EventOrder{time: 3, priority: Priority::Turn, ctr: 5}),
        ];

        for (l, r) in &lt {
            assert!(l < r);
        }

        let gt = [
            (EventOrder{time: 6, priority: Priority::AfterTurn, ctr: 2},
             EventOrder{time: 2, priority: Priority::Turn, ctr: 1}),
            (EventOrder{time: 2, priority: Priority::AfterTurn, ctr: 1},
             EventOrder{time: 2, priority: Priority::Turn, ctr: 1}),
            (EventOrder{time: 2, priority: Priority::Turn, ctr: 44},
             EventOrder{time: 2, priority: Priority::Turn, ctr: 12}),
        ];

        for (l, r) in &gt {
            assert!(l > r);
        }

        assert_eq!(
            EventOrder{time: 2, priority: Priority::Turn, ctr: 1},
            EventOrder{time: 2, priority: Priority::Turn, ctr: 1});
    }

    #[test]
    fn test_event_queue() {
        type TestEvent = u64;
        impl Event for TestEvent {
            fn needs_input(&self) -> bool { return false; }

            fn activate(&self,
                        battle: &mut Battle,
                        input: Option<Box<dyn Input>>) {}

            fn event_key(&self) -> EventKey { EventKey::zero() }

            fn set_event_key(&mut self, key: EventKey) {}
        }

        let tests = [
            vec!(
                (0, 10, Priority::Turn, 0 as TestEvent),
                (1, 10, Priority::Turn, 1 as TestEvent),
            ),
            vec!(
                (1, 10, Priority::Turn, 0 as TestEvent),
                (2, 10, Priority::Turn, 0 as TestEvent),
                (0, 10, Priority::BeforeTurn, 1 as TestEvent),
            ),
            vec!(
                (1, 12, Priority::Turn, 0 as TestEvent),
                (0, 11, Priority::Turn, 0 as TestEvent),
            ),
            vec!(
                (1, 13, Priority::Turn, 0 as TestEvent),
                (2, 14, Priority::Turn, 0 as TestEvent),
                (0, 12, Priority::Turn, 0 as TestEvent),
            ),
            vec!(
                (2, 12, Priority::AfterTurn, 0 as TestEvent),
                (1, 12, Priority::Turn, 0 as TestEvent),
                (0, 11, Priority::BeforeTurn, 0 as TestEvent),
            ),
            vec!(
                (1, 12, Priority::Turn, 0 as TestEvent),
                (0, 12, Priority::Immediate, 0 as TestEvent),
            ),
        ];

        for test_elems in &tests {
            let mut q = EventQueue::new();
            let mut inserted = Vec::new();
            let mut returned = Vec::new();

            for (_, offset, priority, event) in test_elems {
                inserted.push(
                    q.insert(
                        *offset as EventTime,
                        *priority,
                        Box::new(*event),
                    ),
                );
            }

            for _ in 0..test_elems.len() {
                let (order, _) = q.advance();
                returned.push(order);
            }

            for (i, elem) in test_elems.iter().enumerate() {
                let (expected_index, _, _, _) = elem;
                assert_eq!(inserted[i], returned[*expected_index as usize],
                           "event queue test {} failed", i);
            }
        }
    }
}
