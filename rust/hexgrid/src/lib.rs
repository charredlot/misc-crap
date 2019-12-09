use wasm_bindgen::prelude::*;
use game::battle::Battle;

mod game;
mod hex;
mod misc;

use hex::coord::AxialCoord;
use hex::grid::{EdgeCosts, HexGrid};

#[wasm_bindgen]
pub fn initial_battle() -> *mut Battle {
    let center = AxialCoord{q: 0, r: 0};
    let tiles = center.circle_coords(3);
    let mut grid = HexGrid::from(&tiles as &[AxialCoord]);
    grid.insert_edge_for_all_neighbors(EdgeCosts{cost: 1});

    return &mut Battle::new(grid);
}
