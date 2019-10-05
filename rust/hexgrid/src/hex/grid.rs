use wasm_bindgen::prelude::*;

use std::collections::{HashMap, HashSet};
use std::iter::FromIterator;

use super::coord::AxialCoord;

#[wasm_bindgen]
pub struct HexGrid {
    adjacent: HashMap<AxialCoord, Vec<(AxialCoord, u32)>>,
    tiles: HashSet<AxialCoord>,
}

impl HexGrid {
    pub fn new() -> HexGrid {
        HexGrid{
            adjacent: HashMap::new(),
            tiles: HashSet::new(),
        }
    }

    pub fn insert(&mut self, coord: AxialCoord) {
        self.tiles.insert(coord);
    }
}

impl From<&[AxialCoord]> for HexGrid {
    fn from(coords: &[AxialCoord]) -> HexGrid {
        HexGrid{
            adjacent: HashMap::new(),
            tiles: HashSet::from_iter(coords.iter().copied()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_insert() {
        let mut grid = HexGrid::new();
        let coord = AxialCoord{q: 2, r: 3};
        grid.insert(coord);
        assert_eq!(grid.tiles.contains(&AxialCoord{q: 2, r: 3}), true);
        assert_eq!(grid.tiles.contains(&AxialCoord{q: -1, r: 3}), false);
    }
}
