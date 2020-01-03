use wasm_bindgen::prelude::*;

use crate::game::battle::input::{Input};
use super::ActionPoints;
use super::{Battle};
use super::event::{Event, EventKey, EventOrder, EventQueue, EventTime, Priority};
use super::unit::{BattleUnit, BattleUnitKey, BattleUnitStats};

#[wasm_bindgen]
#[derive(Clone, Debug)]
pub struct Turn {
    /* key can't be pub https://github.com/rustwasm/wasm-bindgen/issues/1775 */
    unit_key: BattleUnitKey,
    pub ap: ActionPoints,
    pub key: EventKey,
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
    pub fn new(unit_key: BattleUnitKey, ap: ActionPoints) -> Turn {
        Turn{
            unit_key: unit_key,
            ap: ap,
            key: EventKey::zero(),
        }
    }
}

impl Event for Turn {
    fn needs_input(&self) -> bool {
        return true;
    }

    fn activate(&self, battle: &mut Battle, input: Option<Box<dyn Input>>) {
        /* TODO */
    }

    fn event_key(&self) -> EventKey { self.key }

    fn set_event_key(&mut self, key: EventKey) { self.key = key; }
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
        let unit_key = b.new_unit_key(String::from("a"));
        b.insert_unit(BattleUnit::from(unit_key.clone(),
                                       &BattleUnitStats::new_for_test()));
        for _ in 0..2 {
            b.insert_unit_turn(10 as EventTime,
                               Turn::new(
                                  unit_key.clone(),
                                  0 as ActionPoints,
                               ));
        }

        b.advance(None);

        /* this should fail since it should be waiting for input */
        b.advance(None);
    }

    #[test]
    fn test_input() {
        let mut b = Battle::new(HexGrid::new());
        let unit_key = b.new_unit_key(String::from("a"));
        b.insert_unit(BattleUnit::from(unit_key.clone(),
                                       &BattleUnitStats::new_for_test()));
        for _ in 0..2 {
            b.insert_unit_turn(10 as EventTime,
                               Turn::new(
                                   unit_key.clone(),
                                   0 as ActionPoints,
                               ));
        }

        let _ = b.advance(None);
        assert!(b.peek_event().needs_input());
        b.advance(Some(Box::new(EndTurnInput{})));
    }
}
