use wasm_bindgen::prelude::*;

use super::{ActionPoints, HitPoints};
use super::event::EventTime;
use super::turn::Turn;

pub type BattleUnitKey = String;

#[wasm_bindgen]
#[derive(Clone, Debug)]
pub struct BattleUnitStats {
    /* key can't be pub https://github.com/rustwasm/wasm-bindgen/issues/1775 */
    key: BattleUnitKey,
    pub max_hp: HitPoints,
    pub turn_ap: ActionPoints,
    pub turn_time: EventTime,
}

#[wasm_bindgen]
impl BattleUnitStats {
    #[wasm_bindgen(getter)]
    pub fn key(&self) -> String {
        self.key.clone()
    }

    #[wasm_bindgen(setter)]
    pub fn set_key(&mut self, key: String) {
        self.key = key;
    }
}

#[wasm_bindgen]
#[derive(Clone, Debug)]
pub struct BattleUnit {
    /*
     * can't make this public for same reason key can't be public
     * https://github.com/rustwasm/wasm-bindgen/issues/1775
     */
    base: BattleUnitStats,

    curr_hp: HitPoints,
}

impl BattleUnit {
    pub fn new_turn(&self) -> Turn {
        let t = Turn::new(
            self.base.key.clone(),
            self.base.turn_ap,
            self.base.turn_time,
        );
        return t;
    }
}
