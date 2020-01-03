use std::collections::{HashMap};

use wasm_bindgen::prelude::*;

use super::{ActionPoints, HitPoints};
use super::event::EventTime;
use super::turn::Turn;

pub type BattleUnitKey = String;

/* XXX: don't want to include a heavy json dependency yet, but later */
#[wasm_bindgen]
#[derive(Clone, Copy, Debug)]
pub struct BattleUnitStats {
    /* key can't be pub https://github.com/rustwasm/wasm-bindgen/issues/1775 */
    pub max_hp: HitPoints,
    pub turn_ap: ActionPoints,
    pub turn_time: EventTime,
}

#[wasm_bindgen]
impl BattleUnitStats {
    pub fn new(max_hp: HitPoints,
               turn_ap: ActionPoints,
               turn_time: EventTime) -> BattleUnitStats {
        BattleUnitStats{
            max_hp: max_hp,
            turn_ap: turn_ap,
            turn_time: turn_time,
        }
    }

    #[cfg(test)]
    pub fn new_for_test() -> BattleUnitStats {
        BattleUnitStats{
                          max_hp: 1 as HitPoints,
                          turn_ap: 1 as ActionPoints,
                          turn_time: 1 as EventTime,
        }
    }
}

#[derive(Clone, Debug)]
pub struct BattleUnitStatsDB {
    pub units: HashMap<BattleUnitKey, BattleUnitStats>,
}

#[wasm_bindgen]
#[derive(Clone, Debug)]
pub struct BattleUnit {
    /*
     * can't make key public
     * https://github.com/rustwasm/wasm-bindgen/issues/1775
     */
    key: BattleUnitKey,
    pub base: BattleUnitStats,

    pub curr_hp: HitPoints,
}

#[wasm_bindgen]
impl BattleUnit {
    pub fn from(key: BattleUnitKey, base: &BattleUnitStats) -> BattleUnit {
        BattleUnit{
            key: key,
            base: *base,
            curr_hp: base.max_hp,
        }
    }

    #[wasm_bindgen(getter)]
    pub fn key(&self) -> String {
        self.key.clone()
    }

    #[wasm_bindgen(setter)]
    pub fn set_key(&mut self, key: String) {
        self.key = key;
    }

    pub fn new_turn(&self) -> Turn {
        let t = Turn::new(
            self.key(),
            self.base.turn_ap,
        );
        return t;
    }
}
