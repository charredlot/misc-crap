use crate::hex::grid::HexGrid;
use event::EventQueue;

pub mod event;
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
}

impl Battle {
    pub fn new(grid: HexGrid) -> Battle {
        Battle{
            grid: grid,
            events: EventQueue::new(),
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;
}
