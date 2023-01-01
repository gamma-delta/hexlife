use std::fmt::Display;

use ahash::AHashMap;
use hex2d::Angle;
use math::{Aliveness, EdgePos, EdgesState, HexCoord, RestrictedHexDir};

pub mod math;

#[derive(Clone)]
pub struct Board {
    cells: AHashMap<HexCoord, u8>,
}

impl Board {
    pub fn new() -> Self {
        Self {
            cells: AHashMap::new(),
        }
    }

    pub fn get_liveness(&self, pos: EdgePos) -> Aliveness {
        match self.cells.get(&pos.coord()) {
            None => Aliveness::Dead,
            Some(edges) => {
                let state = EdgesState::unpack(*edges);
                state.get(pos.edge())
            }
        }
    }

    /// Set the edge to be alive or not.
    pub fn set_alive(&mut self, pos: EdgePos, alive: Aliveness) {
        match alive {
            Aliveness::Barren | Aliveness::Alive => {
                let here = self.cells.entry(pos.coord()).or_default();
                let mut state = EdgesState::unpack(*here);
                state.set(pos.edge(), alive);
                *here = state.pack();
            }
            Aliveness::Dead => {
                // Don't bother creating and then immediately removing
                if let Some(here) = self.cells.get_mut(&pos.coord()) {
                    let mut state = EdgesState::unpack(*here);
                    state.set(pos.edge(), alive);
                    let packed = state.pack();
                    if packed == 0 {
                        self.cells.remove(&pos.coord());
                    } else {
                        *here = packed;
                    }
                } // else we're trying to kill an already live cell
            }
        }
    }

    /// Go dead or barren to alive, alive to dead
    pub fn twiddle_alive(&mut self, pos: EdgePos) {
        let alive_here = self.get_liveness(pos);
        self.set_alive(
            pos,
            match alive_here {
                Aliveness::Dead | Aliveness::Barren => Aliveness::Alive,
                _ => Aliveness::Dead,
            },
        );
    }

    /// Get the three edges at the given position
    pub fn get_edges(&self, pos: HexCoord) -> Option<EdgesState> {
        self.cells.get(&pos).copied().map(EdgesState::unpack)
    }

    pub fn apply_rule(&mut self, rule: Rule) {
        // Maps edge positions to the number of neighbors there.
        enum Update {
            NormalNeighborCount(u8),
            Barren,
        }
        let mut updates = AHashMap::<EdgePos, Update>::new();

        for (&coord, &packed) in self.cells.iter() {
            for edge in [
                RestrictedHexDir::XY,
                RestrictedHexDir::ZY,
                RestrictedHexDir::ZX,
            ] {
                let state = EdgesState::unpack(packed);
                let here = EdgePos::new(coord, edge.to_hex2d());
                let liveness = state.get(here.edge());
                match liveness {
                    Aliveness::Barren => {
                        updates.insert(here, Update::Barren);
                    }
                    Aliveness::Dead => {
                        if !updates.contains_key(&here) {
                            updates.insert(here, Update::NormalNeighborCount(0));
                        }
                    }
                    Aliveness::Alive => {
                        if !updates.contains_key(&here) {
                            updates.insert(here, Update::NormalNeighborCount(0));
                        }

                        for neighbor in rule.neighbors.neighbors(here) {
                            let neighbor_state = self.get_liveness(neighbor);
                            if neighbor_state != Aliveness::Barren {
                                match updates.get_mut(&neighbor) {
                                    None => {
                                        updates.insert(neighbor, Update::NormalNeighborCount(1));
                                    }
                                    Some(Update::NormalNeighborCount(ref mut count)) => {
                                        *count += 1;
                                    }
                                    Some(Update::Barren) => {
                                        // This branch shouldn't be taken but I don't think it's bad to
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        for (edge_pos, update) in updates.into_iter() {
            match update {
                Update::NormalNeighborCount(count) => {
                    let alive = match self.get_liveness(edge_pos) {
                        Aliveness::Alive => {
                            if (rule.survive_mask & (1 << count)) != 0 {
                                Aliveness::Alive
                            } else {
                                Aliveness::Barren
                            }
                        }
                        Aliveness::Dead => {
                            if (rule.birth_mask & (1 << count)) != 0 {
                                Aliveness::Alive
                            } else {
                                Aliveness::Dead
                            }
                        }
                        Aliveness::Barren => {
                            panic!("should never be trying to update a barren cell normally")
                        }
                    };
                    // println!(
                    //     "{:?} has {} live neighbors, setting {}",
                    //     edge_pos, count, should_be_alive
                    // );
                    self.set_alive(edge_pos, alive);
                }
                Update::Barren => {
                    self.set_alive(edge_pos, Aliveness::Dead);
                }
            }
        }
    }

    pub fn clear(&mut self) {
        self.cells.clear();
    }
}

/// Instructions on how to update the board.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Rule {
    birth_mask: u32,
    survive_mask: u32,
    neighbors: NeighborRegion,
}

impl Rule {
    pub fn new_raw(birth_mask: u32, survive_mask: u32, neighbors: NeighborRegion) -> Self {
        assert!(
            birth_mask <= (1 << (neighbors.count() + 1)),
            "cannot have birth bits set above the neighbor count"
        );
        assert!(
            survive_mask <= (1 << (neighbors.count() + 1)),
            "cannot have survival bits set above the neighbor count"
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
        for i in 0..=self.neighbors.count() {
            if self.birth_mask & (1 << i) != 0 {
                write!(f, "{:x}", i)?;
            }
        }
        write!(f, "/S")?;
        for i in 0..=self.neighbors.count() {
            if self.survive_mask & (1 << i) != 0 {
                write!(f, "{:x}", i)?;
            }
        }
        write!(f, "/@{}", self.neighbors)?;
        Ok(())
    }
}

/// What is considered to be a neighbor?
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NeighborRegion {
    Four,
    Six,
    EightCross,
    EightParallel,
    Ten,
}

impl NeighborRegion {
    fn count(&self) -> u32 {
        match self {
            NeighborRegion::Four => 4,
            NeighborRegion::Six => 6,
            NeighborRegion::EightCross => 8,
            NeighborRegion::EightParallel => 8,
            NeighborRegion::Ten => 10,
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
            NeighborRegion::EightCross => vec![
                EdgePos::new(coord, real_dir + Angle::Left),
                EdgePos::new(coord, real_dir + Angle::Right),
                EdgePos::new(coord, real_dir + Angle::LeftBack),
                EdgePos::new(coord, real_dir + Angle::RightBack),
                EdgePos::new(neighbor_pos, real_dir + Angle::Left),
                EdgePos::new(neighbor_pos, real_dir + Angle::Right),
                EdgePos::new(neighbor_pos, real_dir + Angle::LeftBack),
                EdgePos::new(neighbor_pos, real_dir + Angle::RightBack),
            ],
            NeighborRegion::EightParallel => {
                let ccw_neighbor = coord + (real_dir + Angle::Left);
                let cw_neighbor = coord + (real_dir + Angle::Right);
                vec![
                    EdgePos::new(coord, real_dir + Angle::Left),
                    EdgePos::new(coord, real_dir + Angle::Right),
                    EdgePos::new(neighbor_pos, real_dir + Angle::LeftBack),
                    EdgePos::new(neighbor_pos, real_dir + Angle::RightBack),
                    EdgePos::new(ccw_neighbor, real_dir),
                    EdgePos::new(ccw_neighbor, real_dir + Angle::Back),
                    EdgePos::new(cw_neighbor, real_dir),
                    EdgePos::new(cw_neighbor, real_dir + Angle::Back),
                ]
            }
            NeighborRegion::Ten => vec![
                EdgePos::new(coord, real_dir + Angle::Left),
                EdgePos::new(coord, real_dir + Angle::Right),
                EdgePos::new(coord, real_dir + Angle::LeftBack),
                EdgePos::new(coord, real_dir + Angle::RightBack),
                EdgePos::new(coord, real_dir + Angle::Back),
                EdgePos::new(neighbor_pos, real_dir + Angle::Left),
                EdgePos::new(neighbor_pos, real_dir + Angle::Right),
                EdgePos::new(neighbor_pos, real_dir + Angle::LeftBack),
                EdgePos::new(neighbor_pos, real_dir + Angle::RightBack),
                EdgePos::new(neighbor_pos, real_dir),
            ],
        }
    }
}

impl Display for NeighborRegion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            NeighborRegion::Four => write!(f, "4"),
            NeighborRegion::Six => write!(f, "6"),
            NeighborRegion::EightCross => write!(f, "8*"),
            NeighborRegion::EightParallel => write!(f, "8="),
            NeighborRegion::Ten => write!(f, "10"),
        }
    }
}
