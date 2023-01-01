use std::fmt::Display;

use ahash::AHashMap;
use hex2d::Angle;
use math::{EdgePos, HexCoord, HexEdges};

pub mod math;

#[derive(Clone)]
pub struct Board {
    cells: AHashMap<HexCoord, HexEdges>,
}

impl Board {
    pub fn new() -> Self {
        Self {
            cells: AHashMap::new(),
        }
    }

    /// Return whether that edge is alive.
    pub fn is_alive(&self, pos: EdgePos) -> bool {
        match self.cells.get(&pos.coord()) {
            None => false,
            Some(edges) => edges.contains(pos.edge()),
        }
    }

    /// Set the edge to be alive or not.
    pub fn set_alive(&mut self, pos: EdgePos, alive: bool) {
        if alive {
            let edges = self.cells.entry(pos.coord()).or_default();
            edges.insert(pos.edge());
        } else if let Some(here) = self.cells.get_mut(&pos.coord()) {
            // Don't bother creating and then immediately removing
            here.remove(pos.edge());
        }
    }

    pub fn toggle_alive(&mut self, pos: EdgePos) {
        let alive_here = self.is_alive(pos);
        self.set_alive(pos, !alive_here);
    }

    /// Get the three edges at the given position
    pub fn get_edges(&self, pos: HexCoord) -> Option<HexEdges> {
        self.cells.get(&pos).copied()
    }

    pub fn apply_rule(&mut self, rule: Rule) {
        // Maps edge positions to the number of neighbors there.
        let mut neighbor_count = AHashMap::<EdgePos, u8>::new();

        for (&coord, &edges) in self.cells.iter() {
            for edge in edges {
                let here = EdgePos::new(coord, edge.to_hex2d());
                if !neighbor_count.contains_key(&here) {
                    neighbor_count.insert(here, 0);
                }

                for neighbor in rule.neighbors.neighbors(here) {
                    let slot = neighbor_count.entry(neighbor).or_default();
                    *slot += 1;
                }
            }
        }

        for (edge_pos, count) in neighbor_count.into_iter() {
            let should_be_alive = match self.is_alive(edge_pos) {
                true => (rule.survive_mask & (1 << count)) != 0,
                false => (rule.birth_mask & (1 << count)) != 0,
            };
            // println!(
            //     "{:?} has {} live neighbors, setting {}",
            //     edge_pos, count, should_be_alive
            // );
            self.set_alive(edge_pos, should_be_alive);
        }
    }

    pub fn clear(&mut self) {
        self.cells.clear();
    }
}

/// Instructions on how to update the board.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Rule {
    birth_mask: u8,
    survive_mask: u8,
    neighbors: NeighborRegion,
}

impl Rule {
    pub fn new_raw(birth_mask: u8, survive_mask: u8, neighbors: NeighborRegion) -> Self {
        assert!(
            birth_mask < (1 << neighbors.count()),
            "cannot have bits set above the neighbor count"
        );
        assert!(
            survive_mask < (1 << neighbors.count()),
            "cannot have bits set above the neighbor count"
        );
        Self {
            birth_mask,
            survive_mask,
            neighbors,
        }
    }
}

impl Display for Rule {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "B")?;
        for i in 0..5 {
            if self.birth_mask & (1 << i) != 0 {
                write!(f, "{}", i)?;
            }
        }
        write!(f, "/S")?;
        for i in 0..5 {
            if self.survive_mask & (1 << i) != 0 {
                write!(f, "{}", i)?;
            }
        }
        write!(f, "/@{}", self.neighbors.count())?;
        Ok(())
    }
}

/// What is considered to be a neighbor?
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NeighborRegion {
    Four,
    Six,
}

impl NeighborRegion {
    fn count(&self) -> u8 {
        match self {
            NeighborRegion::Four => 4,
            NeighborRegion::Six => 6,
        }
    }
    fn neighbors(&self, pos: EdgePos) -> Vec<EdgePos> {
        let coord = pos.coord();
        let real_dir = pos.edge().to_hex2d();
        let neighbor_pos = pos.coord() + real_dir;
        match self {
            NeighborRegion::Four => vec![
                EdgePos::new(coord, real_dir + Angle::Left),
                EdgePos::new(coord, real_dir + Angle::Right),
                EdgePos::new(neighbor_pos, real_dir + Angle::LeftBack),
                EdgePos::new(neighbor_pos, real_dir + Angle::RightBack),
            ],
            NeighborRegion::Six => vec![
                EdgePos::new(coord, real_dir + Angle::Left),
                EdgePos::new(coord, real_dir + Angle::Right),
                EdgePos::new(coord, real_dir + Angle::Back),
                EdgePos::new(neighbor_pos, real_dir + Angle::LeftBack),
                EdgePos::new(neighbor_pos, real_dir + Angle::RightBack),
                EdgePos::new(neighbor_pos, real_dir),
            ],
        }
    }
}
