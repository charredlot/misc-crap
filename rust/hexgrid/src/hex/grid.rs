use wasm_bindgen::prelude::*;

use std::cmp::{Ord, Ordering, PartialOrd};
use std::collections::{BTreeSet, HashMap, HashSet};
use std::iter::FromIterator;

use super::coord::AxialCoord;

#[wasm_bindgen]
pub struct HexGrid {
    adjacent: HashMap<AxialCoord, HashMap<AxialCoord, EdgeCosts>>,
    tiles: HashSet<AxialCoord>,
}

/*
 * only used internally for pathfinding stuff
 * float would be nicer, but they're a PITA for this use case since
 * they don't implement cmp
 */
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Hash)]
struct CostCoord {
    coord: AxialCoord,
    costx10: u32, /* use fixed point so we can tweak the scores */
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

    fn insert_edge_adjacency(
        adjacent: &mut HashMap<AxialCoord, HashMap<AxialCoord, EdgeCosts>>,
        tiles: &HashSet<AxialCoord>,
        src: AxialCoord,
        dst: AxialCoord,
        costs: EdgeCosts,
    ) -> bool {
        if !tiles.contains(&src) || !tiles.contains(&dst) {
            return false;
        }

        let dsts = adjacent.entry(src).or_insert_with(HashMap::new);
        dsts.entry(dst)
            .and_modify(|curr_costs| {
                curr_costs.merge(costs);
            })
            .or_insert(costs);
        return true;
    }

    pub fn insert_edge(&mut self,
                       src: AxialCoord,
                       dst: AxialCoord,
                       costs: EdgeCosts) -> bool {
        return HexGrid::insert_edge_adjacency(&mut self.adjacent,
                                              &self.tiles,
                                              src, dst, costs);
    }

    pub fn insert_edge_for_all_neighbors(&mut self, default_costs: EdgeCosts) {
        let tiles = &self.tiles;
        let adjacent = &mut self.adjacent;
        for &coord in tiles {
            for &neighbor in &coord.neighbors() {
                HexGrid::insert_edge_adjacency(
                    adjacent, tiles, coord, neighbor, default_costs.clone(),
                );
                HexGrid::insert_edge_adjacency(
                    adjacent, tiles, neighbor, coord, default_costs.clone(),
                );
            }
        }
    }

    pub fn get_edge_costs(&self,
                          src: AxialCoord,
                          dst: AxialCoord) -> Option<&EdgeCosts> {
        let dsts = self.adjacent.get(&src)?;
        let costs = dsts.get(&dst)?;
        Some(&costs)
    }

    pub fn get_path(&self,
                    src: &AxialCoord,
                    dst: &AxialCoord) -> Option<Vec<AxialCoord>> {
        /* a star as implemented in wiki */

        /* predecessor map to rebuild the path */
        let mut best_prev: HashMap<AxialCoord, AxialCoord> = HashMap::new();

        /* g(x) is best edge cost to get to the given coord x from src */
        let mut g_scorex10: HashMap<AxialCoord, u32> = HashMap::new();

        /* the value is f(x) = g(x) + h(x), where h is the heuristic */
        let mut open_set: BTreeSet<CostCoord> = BTreeSet::new();
        let mut visited: HashSet<AxialCoord> = HashSet::new();

        g_scorex10.insert(*src, 0);
        open_set.insert(CostCoord{coord: *src, costx10: 0});
        while !open_set.is_empty() {
            /* pop the min val (but there's no pop or drain method Q_Q) */
            let curr = open_set.iter().next().unwrap().clone();
            open_set.remove(&curr);

            /* found the path */
            if curr.coord == *dst {
                let mut path = Vec::new();
                let mut curr = *dst;
                loop {
                    path.push(curr);
                    if let Some(&prev) = best_prev.get(&curr) {
                        curr = prev;
                    }
                    else {
                        break;
                    }
                }
                path.reverse();
                return Some(path);
            }

            visited.insert(curr.coord);

            let neighbors = self.adjacent.get(&curr.coord);
            if neighbors.is_none() {
                continue;
            }

            let prefer_dir = match best_prev.get(&curr.coord) {
                Some(prev_coord) => Some(prev_coord.dir(&curr.coord)),
                None => None,
            };

            /* XXX: get best_dir and add a bonus for direction */
            let neighbors = neighbors.unwrap();
            for (&neighbor_coord, &costs) in neighbors {
                if visited.get(&neighbor_coord).is_some() {
                    continue;
                }

                /*
                 * curr.coord is either the start coord, or it was put into
                 * open_set after going through this loop. so it's always
                 * in g_scorex10
                 */
                let tentative_scorex10 = g_scorex10.get(&curr.coord).unwrap() +
                                        (costs.cost * 10);
                let update = match g_scorex10.get(&neighbor_coord) {
                    Some(&scorex10) => (tentative_scorex10 < scorex10),
                    None => true,
                };

                if !update {
                    continue;
                }

                best_prev.insert(neighbor_coord, curr.coord);
                g_scorex10.insert(neighbor_coord, tentative_scorex10);

                /*
                 * this is the heuristic, it should never overestimate the cost
                 * to be admissible
                 *
                 * add 0.5 cost if it causes a dir change. this is so that the
                 * paths prefer maintaining a straight line to look more
                 * natural
                 */
                let neigh_dir = neighbor_coord.dir(&curr.coord);
                let h_scorex10 = neighbor_coord.hex_distance(&dst) * 10 +
                    match prefer_dir {
                        Some(dir) => {
                            if neigh_dir != dir {
                                5
                            }
                            else {
                                0
                            }
                        },
                        None => 0,
                    };
                /* this is f(x) = g(x) + h(x) */
                open_set.insert(CostCoord{coord: neighbor_coord,
                                          costx10: h_scorex10});
            }
        }

        return None
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

impl Ord for CostCoord {
    fn cmp(&self, other: &Self) -> Ordering {
        self.costx10.cmp(&other.costx10)
    }
}

impl PartialOrd for CostCoord {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
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

    #[test]
    pub fn test_get_path() {
        let center = AxialCoord{q: 0, r: 0};
        let tiles = center.circle_coords(3);
        let mut grid = HexGrid::from(&tiles as &[AxialCoord]);
        grid.insert_edge_for_all_neighbors(EdgeCosts{cost: 1});

        assert_eq!(grid.get_path(&center, &AxialCoord{q: 5, r: 5}), None);

        /*
         * there can be multiple valid paths and the algo will randomly pick
         * one depending on map order. so we need multiple test possibilities
         */
        let tests = [
            ((center, AxialCoord{q: -3, r: 3}),
              vec![
                vec![AxialCoord{q: 0, r: 0},
                     AxialCoord{q: -1, r: 1},
                     AxialCoord{q: -2, r: 2},
                     AxialCoord{q: -3, r: 3}],
              ]),
            ((center, AxialCoord{q: 2, r: 1}),
              vec![
                vec![AxialCoord{q: 0, r: 0},
                     AxialCoord{q: 0, r: 1},
                     AxialCoord{q: 1, r: 1},
                     AxialCoord{q: 2, r: 1}],
                vec![AxialCoord{q: 0, r: 0},
                     AxialCoord{q: 1, r: 0},
                     AxialCoord{q: 2, r: 0},
                     AxialCoord{q: 2, r: 1}],
              ]),
        ];
        for ((src, dst), possibilities) in &tests {
            let path = grid.get_path(src, dst).unwrap();
            let mut matched = false;

            for possible_path in possibilities {
                if path == *possible_path {
                    matched = true;
                    break;
                }
            }

            assert!(matched,
                    "Path {:?} did not match any of {:?}",
                    path, possibilities);
        }
    }
}
