use wasm_bindgen::prelude::*;

mod game;
mod hex;
mod misc;

use hex::coord::AxialCoord;
use hex::grid::{EdgeCosts, HexGrid};

#[wasm_bindgen]
pub fn initial_grid() -> HexGrid {
    let center = AxialCoord{q: 0, r: 0};
    let tiles = center.circle_coords(3);
    let mut grid = HexGrid::from(&tiles as &[AxialCoord]);
    grid.insert_edge_for_all_neighbors(EdgeCosts{cost: 1});

    return grid;
}
