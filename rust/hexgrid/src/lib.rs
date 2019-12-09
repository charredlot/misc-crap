use wasm_bindgen::prelude::*;
use game::battle::Battle;

mod game;
mod hex;
mod misc;

use hex::coord::AxialCoord;
use hex::grid::{EdgeCosts, HexGrid};

#[wasm_bindgen]
pub struct JSBattle {
    /* XXX: need a ptr to avoid unnecessary copies? investigate more */
    ptr: Box<Battle>,
}

#[wasm_bindgen]
impl JSBattle {
    pub fn get_grid_coords_json(&self) -> String {
        self.ptr.grid.get_coords_json()
    }
}


/* XXX: replace this later this is just for testing */
#[wasm_bindgen]
pub fn initial_battle() -> JSBattle {
    let center = AxialCoord{q: 0, r: 0};
    let tiles = center.circle_coords(3);
    let mut grid = HexGrid::from(&tiles as &[AxialCoord]);
    grid.insert_edge_for_all_neighbors(EdgeCosts{cost: 1});

    return JSBattle{ptr: Box::new(Battle::new(grid))};
}
