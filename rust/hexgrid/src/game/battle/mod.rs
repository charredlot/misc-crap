use std::collections::{HashMap};

use crate::hex::grid::HexGrid;
use event::{Event, EventKey, EventOrder, EventQueue, EventTime, NullEvent, Priority};
use input::{Input};
use turn::{Turn};
use unit::{BattleUnit, BattleUnitKey};

pub mod event;
pub mod input;
pub mod turn;
pub mod unit;

pub type ActionPoints = u64;
pub type HitPoints = u64;

pub struct Battle {
    /*
     * there's some issue about Copy being required for pub structs
     * https://github.com/rustwasm/wasm-bindgen/issues/439
     */
    pub grid: HexGrid,
    events: EventQueue,
    curr_event: Box<dyn Event>,
    needs_input: bool,
    pub units: HashMap<BattleUnitKey, BattleUnit>,
    unit_key_ctr: u64,
    pub next_turn: HashMap<BattleUnitKey, EventKey>,
}

impl Battle {
    pub fn new(grid: HexGrid) -> Battle {
        Battle{
            grid: grid,
            events: EventQueue::new(),
            curr_event: Box::new(NullEvent::new()),
            needs_input: false,
            units: HashMap::new(),
            unit_key_ctr: 0,
            next_turn: HashMap::new(),
        }
    }

    pub fn insert_event(&mut self,
                        offset: EventTime,
                        priority: Priority,
                        event: Box<dyn Event>) -> EventKey {
        self.events.insert(offset, priority, event)
    }

    pub fn peek_event(&self) -> &dyn Event {
        &*self.curr_event
    }

    pub fn new_unit_key(&mut self, base_key: BattleUnitKey) -> BattleUnitKey {
        self.unit_key_ctr += 1;
        return format!("{}_{}", &base_key, self.unit_key_ctr);
    }

    pub fn insert_unit(&mut self, unit: BattleUnit) {
        let key = unit.key();

        assert!(!self.units.contains_key(&key));

        self.units.insert(key, unit);
    }

    pub fn insert_unit_turn(&mut self,
                            offset: EventTime,
                            turn: Turn) -> EventKey{
        let unit_key = turn.unit_key();
        assert!(self.units.contains_key(&unit_key));
        let key = self.insert_event(offset, Priority::Turn, Box::new(turn));
        self.next_turn.insert(unit_key, key);
        return key;
    }

    pub fn advance(&mut self,
                   input: Option<Box<dyn Input>>) -> EventOrder {
        assert!(!self.needs_input || input.is_some());

        let (order, event) = self.events.advance();
        let curr_event = std::mem::replace(&mut self.curr_event, event);
        curr_event.activate(self, input);

        self.needs_input = self.curr_event.needs_input();

        return order;
    }
}


#[cfg(test)]
mod tests {
    use crate::hex::grid::{HexGrid};
    use super::*;

    #[test]
    #[should_panic]
    fn test_input_event() {
        type TestInputEvent = u32;
        impl Event for TestInputEvent {
            fn needs_input(&self) -> bool { return true; }
            fn activate(&self,
                        battle: &mut Battle,
                        input: Option<Box<dyn Input>>) {}
            fn event_key(&self) -> EventKey { EventKey::zero() }
            fn set_event_key(&mut self, key: EventKey) {}
        }

        let mut b = Battle::new(HexGrid::new());
        for _ in 0..2 {
            b.events.insert(
                1 as EventTime,
                Priority::Turn,
                Box::new(0 as TestInputEvent),
            );
        }

        b.advance(None);

        /* this should fail since it should be waiting for input */
        b.advance(None);
    }
}
