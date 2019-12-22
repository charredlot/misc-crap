use wasm_bindgen::prelude::*;

use super::ActionPoints;
use super::event::{Event, EventOrder, EventQueue, EventTime, Priority};
use super::unit::{BattleUnit, BattleUnitKey};

#[wasm_bindgen]
pub struct Turn {
    /* key can't be pub https://github.com/rustwasm/wasm-bindgen/issues/1775 */
    unit_key: BattleUnitKey,
    pub ap: ActionPoints,
    pub time: EventTime,
}

#[wasm_bindgen]
impl Turn {
    #[wasm_bindgen(getter)]
    pub fn unit_key(&self) -> String {
        self.unit_key.clone()
    }

    #[wasm_bindgen(setter)]
    pub fn set_unit_key(&mut self, unit_key: String) {
        self.unit_key = unit_key;
    }
}

impl Turn {
    pub fn new(unit_key: BattleUnitKey,
            ap: ActionPoints,
            time: EventTime) -> Turn {
        Turn{
            unit_key: unit_key,
            ap: ap,
            time: time,
        }
    }
}

fn insert_turn(q: &mut EventQueue, turn: Turn) {
    q.insert(
        turn.time,
        Priority::Turn,
        Box::new(turn),
    );
}

impl Event for Turn {
    fn needs_input(&self) -> bool {
        return true;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[should_panic]
    fn test_event_queue_panic() {
        let mut q = EventQueue::new();
        for _ in 0..2 {
            insert_turn(&mut q, Turn{
                time: 10 as EventTime,
                ap: 0 as ActionPoints,
                unit_key: String::from("a") as BattleUnitKey,
            });
        }

        q.advance();

        /* this should fail since it should be waiting for input */
        q.advance();
    }

    #[test]
    fn test_event_queue() {
        let mut q = EventQueue::new();
        for _ in 0..2 {
            insert_turn(&mut q, Turn{
                time: 10 as EventTime,
                ap: 0 as ActionPoints,
                unit_key: String::from("a") as BattleUnitKey,
            });
        }

        let (_, e) = q.advance();
        assert!(e.needs_input());
        q.input_processed();
        q.advance();
    }
}
