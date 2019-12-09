use crate::hex::grid::HexGrid;
use event::EventQueue;

pub mod event;
pub mod turn;

pub type ActionPoints = u64;

pub struct Battle {
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
