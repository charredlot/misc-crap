use wasm_bindgen::prelude::*;

use std::cmp::{max, min};

use crate::misc;
use misc::gcd32;

#[wasm_bindgen]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Hash)]
pub struct AxialCoord {
    pub q: i32,
    pub r: i32,
}

#[wasm_bindgen]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Hash)]
pub struct AxialDir {
    pub dq: i32,
    pub dr: i32,
}

#[wasm_bindgen]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Hash)]
pub struct CubeCoord {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

/* axial coordinates per https://www.redblobgames.com/grids/hexagons/ */
impl AxialCoord {
    pub fn neighbors(&self) -> Vec<AxialCoord> {
        vec![
            AxialCoord{q: self.q - 1, r: self.r},
            AxialCoord{q: self.q - 1, r: self.r + 1},
            AxialCoord{q: self.q, r: self.r - 1},
            AxialCoord{q: self.q, r: self.r + 1},
            AxialCoord{q: self.q + 1, r: self.r - 1},
            AxialCoord{q: self.q + 1, r: self.r},
        ]
    }

    pub fn to_cartesian(&self, radius: f64) -> (f64, f64) {
        let s3 = 3_f64.sqrt();
        let qf = self.q as f64;
        let rf = self.r as f64;
        (radius * s3 * (qf + (rf / 2_f64)),
         radius * ((rf * 3_f64) / 2_f64))
    }

    pub fn hex_distance(&self, other: &AxialCoord) -> u32 {
        (((self.q - other.q).abs() +
          (self.q + self.r - other.q - other.r).abs() +
          (self.r - other.r).abs()) / 2) as u32
    }

    pub fn circle_coords(&self, radius: usize) -> Vec<AxialCoord> {
        /*
         * a circle centered at self
         * see https://www.redblobgames.com/grids/hexagons/
         */
        let mut coords = Vec::new();
        let r = radius as i32;
        let center = CubeCoord::from(*self);
        for x in -r..(r+1) {
            let min_y: i32 = max(-r, -x - r);
            let max_y: i32 = min(r, -x + r)  + 1;
            for y in min_y..max_y {
                /* x + y + z = 0 for cube coords */
                coords.push(AxialCoord::from(
                    CubeCoord{x: center.x + x, y: y, z: center.z + 0 - x - y}
                ));
            }
        }
        coords
    }

    pub fn dir(&self, dst: AxialCoord) -> AxialDir {
        let mut dq = dst.q - self.q;
        let mut dr = dst.r - self.r;
        let divisor = gcd32(dq, dr);
        if divisor != 0 {
            dq = dq / divisor;
            dr = dr / divisor;
        }
        AxialDir{dq: dq, dr: dr}
    }
}

impl From<CubeCoord> for AxialCoord {
    fn from(coord: CubeCoord) -> AxialCoord {
        AxialCoord{q: coord.x, r: coord.z}
    }

}

impl From<AxialCoord> for CubeCoord {
    fn from(coord: AxialCoord) -> CubeCoord {
        CubeCoord{x: coord.q, y: -coord.q - coord.r, z: coord.r}
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn test_neighbors() {
        let expected: HashSet<AxialCoord> =
            [
                AxialCoord{q: 0, r: 0},
                AxialCoord{q: 0, r: 1},
                AxialCoord{q: 1, r: -1},
                AxialCoord{q: 1, r: 1},
                AxialCoord{q: 2, r: -1},
                AxialCoord{q: 2, r: 0},
            ].iter().copied().collect();
        let coord = AxialCoord{q: 1, r: 0};
        let neighbors: HashSet<AxialCoord> =
            coord.neighbors().iter().copied().collect();
        assert_eq!(neighbors, expected);
    }

    #[test]
    fn test_cartesian() {
        let coord = AxialCoord{q: 1, r: -1};
        let (x, y) = coord.to_cartesian(3.0);
        assert_eq!((x * 1000.0).round() as i32, 2598);
        assert_eq!((y * 1000.0).round() as i32, -4500);
    }

    #[test]
    fn test_distance() {
        let origin = AxialCoord{q: 1, r: -1};
        assert_eq!(origin.hex_distance(&AxialCoord{q: 2, r: -2}), 1);
        assert_eq!(origin.hex_distance(&AxialCoord{q: 2, r: -1}), 1);
        assert_eq!(origin.hex_distance(&AxialCoord{q: 1, r: 0}), 1);
        assert_eq!(origin.hex_distance(&AxialCoord{q: 0, r: 0}), 1);
        assert_eq!(origin.hex_distance(&AxialCoord{q: 2, r: 0}), 2);
        assert_eq!(origin.hex_distance(&AxialCoord{q: 1, r: 1}), 2);
    }

    #[test]
    fn test_cube_axial_conversions() {
        let axial = AxialCoord{q: 3, r: -2};
        let cube: CubeCoord = CubeCoord::from(axial);
        assert_eq!(cube, CubeCoord{x: 3, y: -1, z: -2});
        assert_eq!(AxialCoord::from(cube), axial);
    }

    #[test]
    fn test_dir() {
        assert_eq!(AxialCoord{q: 2, r: 3}.dir(AxialCoord{q: 4, r: 5}),
                   AxialDir{dq: 1, dr: 1});
        assert_eq!(AxialCoord{q: 5, r: 1}.dir(AxialCoord{q: 3, r: 6}),
                   AxialDir{dq: -2, dr: 5});
        assert_eq!(AxialCoord{q: -2, r: 3}.dir(AxialCoord{q: -8, r: -1}),
                   AxialDir{dq: -3, dr: -2});
    }

    #[test]
    fn test_circle() {
        let center = AxialCoord{q: 1, r: 0};
        let expected: HashSet<AxialCoord> =
            [
                center,
                AxialCoord{q: -1, r: 0},
                AxialCoord{q: -1, r: 1},
                AxialCoord{q: -1, r: 2},
                AxialCoord{q: 0, r: -1},
                AxialCoord{q: 0, r: 0},
                AxialCoord{q: 0, r: 1},
                AxialCoord{q: 0, r: 2},
                AxialCoord{q: 1, r: -2},
                AxialCoord{q: 1, r: -1},
                AxialCoord{q: 1, r: 1},
                AxialCoord{q: 1, r: 2},
                AxialCoord{q: 2, r: -2},
                AxialCoord{q: 2, r: -1},
                AxialCoord{q: 2, r: 0},
                AxialCoord{q: 2, r: 1},
                AxialCoord{q: 3, r: -2},
                AxialCoord{q: 3, r: -1},
                AxialCoord{q: 3, r: 0},
            ].iter().copied().collect();
        let circle: HashSet<AxialCoord> =
            center.circle_coords(2).iter().copied().collect();
        assert_eq!(circle, expected);
    }

    #[test]
    fn test_hex_distance() {
        let coord = AxialCoord{q: -1, r: 1};
        assert_eq!(coord.hex_distance(&coord), 0);
        assert_eq!(coord.hex_distance(&AxialCoord{q: 3, r: -1}), 4);
        assert_eq!(coord.hex_distance(&AxialCoord{q: 0, r: 0}), 1);
        assert_eq!(coord.hex_distance(&AxialCoord{q: -3, r: 1}), 2);
        assert_eq!(coord.hex_distance(&AxialCoord{q: -2, r: 4}), 3);
    }
}
