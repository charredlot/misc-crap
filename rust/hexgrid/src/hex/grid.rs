use wasm_bindgen::prelude::*;

use std::collections::{HashMap, HashSet};
use std::iter::FromIterator;

use super::coord::AxialCoord;

#[wasm_bindgen]
pub struct HexGrid {
    adjacent: HashMap<AxialCoord, HashMap<AxialCoord, EdgeCosts>>,
    tiles: HashSet<AxialCoord>,
}

/* might want to make this like a map in the future */
#[wasm_bindgen]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Hash)]
pub struct EdgeCosts {
    /* probably want floats, but they're not equalable or hashable */
    pub cost: u32,
}

impl EdgeCosts {
    pub fn merge(&mut self, other: EdgeCosts) {
        self.cost = other.cost;
    }
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

    pub fn insert_edge(&mut self,
                       src: AxialCoord,
                       dst: AxialCoord,
                       costs: EdgeCosts) -> bool {
        if !self.tiles.contains(&src) || !self.tiles.contains(&dst) {
            return false;
        }

        let dsts = self.adjacent.entry(src).or_insert_with(HashMap::new);
        dsts.entry(dst)
            .and_modify(|curr_costs| {
                curr_costs.merge(costs);
            })
            .or_insert(costs);
        return true;
    }

    pub fn get_edge_costs(&self,
                          src: AxialCoord,
                          dst: AxialCoord) -> Option<&EdgeCosts> {
        let dsts = self.adjacent.get(&src)?;
        let costs = dsts.get(&dst)?;
        Some(&costs)
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

    #[test]
    pub fn test_edge() {
        let mut grid = HexGrid::new();

        let src = AxialCoord{q: 2, r: 1};
        let dst = AxialCoord{q: 1, r: 1};
        grid.insert(src);
        grid.insert(dst);

        let costs = EdgeCosts{cost: 4};
        assert_eq!(grid.insert_edge(src, dst, costs), true);

        let result = grid.get_edge_costs(src, dst).unwrap();
        assert_eq!(result, &costs);
        assert_eq!(grid.get_edge_costs(AxialCoord{q: -1, r: 0},
                                       AxialCoord{q: 0, r: 0}),
                   None);
    }
}
