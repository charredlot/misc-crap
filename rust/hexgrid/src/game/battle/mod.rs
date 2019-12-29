use crate::hex::grid::HexGrid;
use event::{Event, EventOrder, EventQueue, EventTime, NullEvent, Priority};
use input::{Input};

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
}

impl Battle {
    pub fn new(grid: HexGrid) -> Battle {
        Battle{
            grid: grid,
            events: EventQueue::new(),
            curr_event: Box::new(NullEvent{}),
            needs_input: false,
        }
    }

    pub fn insert_event(&mut self,
                        offset: EventTime,
                        priority: Priority,
                        event: Box<dyn Event>) -> EventOrder {
        self.events.insert(offset, priority, event)
    }

    pub fn peek_event(&self) -> &dyn Event {
        &*self.curr_event
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
