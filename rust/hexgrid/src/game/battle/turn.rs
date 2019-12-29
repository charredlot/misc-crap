use wasm_bindgen::prelude::*;

use crate::game::battle::input::{Input};
use super::ActionPoints;
use super::{Battle};
use super::event::{Event, EventOrder, EventQueue, EventTime, Priority};
use super::unit::{BattleUnit, BattleUnitKey};

#[wasm_bindgen]
#[derive(Clone, Debug)]
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

fn insert_turn(battle: &mut Battle, turn: Turn) {
    battle.insert_event(turn.time, Priority::Turn, Box::new(turn));
}

impl Event for Turn {
    fn needs_input(&self) -> bool {
        return true;
    }

    fn activate(&self, battle: &mut Battle, input: Option<Box<dyn Input>>) {
        /* TODO */
    }
}

#[cfg(test)]
mod tests {
    use crate::game::battle::{Battle};
    use crate::game::battle::input::{EndTurnInput};
    use crate::hex::grid::{HexGrid};
    use super::*;

    #[test]
    #[should_panic]
    fn test_input_panic() {
        let mut b = Battle::new(HexGrid::new());
        for _ in 0..2 {
            insert_turn(&mut b, Turn{
                time: 10 as EventTime,
                ap: 0 as ActionPoints,
                unit_key: String::from("a") as BattleUnitKey,
            });
        }

        b.advance(None);

        /* this should fail since it should be waiting for input */
        b.advance(None);
    }

    #[test]
    fn test_input() {
        let mut b = Battle::new(HexGrid::new());
        for _ in 0..2 {
            insert_turn(&mut b, Turn{
                time: 10 as EventTime,
                ap: 0 as ActionPoints,
                unit_key: String::from("a") as BattleUnitKey,
            });
        }

        let _ = b.advance(None);
        assert!(b.peek_event().needs_input());
        b.advance(Some(Box::new(EndTurnInput{})));
    }
}
