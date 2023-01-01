//! Math and coordinate primitives.

use hex2d::{Coordinate, Direction};

pub type HexCoord = Coordinate<i64>;

/// Location of an edge on the grid, for convenience.
///
/// This is never actually used internally but it is handy to handle.
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub struct EdgePos {
    coord: HexCoord,
    edge: RestrictedHexDir,
}

impl EdgePos {
    /// Turn a coordinate and a "raw" direction into an `EdgePos`.
    ///
    /// This will canonicalize the edge direction (so a direction that isn't tracked
    /// will instead shift the coordinate and store the opposite direction, which is.)
    pub fn new(coord: HexCoord, dir: Direction) -> EdgePos {
        let (coord, edge) = canonicalize(coord, dir);
        EdgePos { coord, edge }
    }

    pub fn new_raw(coord: HexCoord, edge: RestrictedHexDir) -> EdgePos {
        EdgePos { coord, edge }
    }

    pub fn coord(&self) -> HexCoord {
        self.coord
    }

    pub fn dir(&self) -> Direction {
        self.edge.to_hex2d()
    }

    pub fn edge(&self) -> RestrictedHexDir {
        self.edge
    }
}

/// Hex direction but only for the 3 directions we track on the coord
#[repr(u8)]
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub enum RestrictedHexDir {
    /// Right
    XY,
    /// Down-right
    ZY,
    /// Down-left
    ZX,
}

impl RestrictedHexDir {
    pub fn to_hex2d(&self) -> Direction {
        match self {
            RestrictedHexDir::XY => Direction::XY,
            RestrictedHexDir::ZY => Direction::ZY,
            RestrictedHexDir::ZX => Direction::ZX,
        }
    }
}

/// Turn an unrestricted direction into the restricted direction on the coordinate
pub(crate) fn canonicalize(coord: HexCoord, dir: Direction) -> (HexCoord, RestrictedHexDir) {
    match dir {
        Direction::XY => (coord, RestrictedHexDir::XY),
        Direction::ZY => (coord, RestrictedHexDir::ZY),
        Direction::ZX => (coord, RestrictedHexDir::ZX),
        // These three we need to offset
        // Offset by the direction and flip
        Direction::YX => (coord + Direction::YX, RestrictedHexDir::XY),
        Direction::YZ => (coord + Direction::YZ, RestrictedHexDir::ZY),
        Direction::XZ => (coord + Direction::XZ, RestrictedHexDir::ZX),
    }
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, Default)]
#[repr(u8)]
pub enum Aliveness {
    #[default]
    Dead = 0,
    Barren = 1,
    Alive = 2,
}

impl Aliveness {
    fn unconvert(x: u8) -> Self {
        match x {
            0 => Aliveness::Dead,
            1 => Aliveness::Barren,
            2 => Aliveness::Alive,
            _ => unreachable!(),
        }
    }
}

/// Thing we pretend to use internally tracking the liveness of the three edges.
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, Default)]
pub struct EdgesState {
    xy: Aliveness,
    zy: Aliveness,
    zx: Aliveness,
}

impl EdgesState {
    pub fn new(xy: Aliveness, zy: Aliveness, zx: Aliveness) -> Self {
        Self { xy, zy, zx }
    }

    pub fn get(&self, edge: RestrictedHexDir) -> Aliveness {
        match edge {
            RestrictedHexDir::XY => self.xy,
            RestrictedHexDir::ZY => self.zy,
            RestrictedHexDir::ZX => self.zx,
        }
    }

    pub fn set(&mut self, edge: RestrictedHexDir, alive: Aliveness) {
        let slot = match edge {
            RestrictedHexDir::XY => &mut self.xy,
            RestrictedHexDir::ZY => &mut self.zy,
            RestrictedHexDir::ZX => &mut self.zx,
        };
        *slot = alive;
    }

    /// Pack into a number from 0 to 26.
    ///
    /// The least significant trit is XY, then ZY, then ZX.
    pub(crate) fn pack(&self) -> u8 {
        self.xy as u8 + self.zy as u8 * 3 + self.zx as u8 * 9
    }

    pub(crate) fn unpack(packed: u8) -> Self {
        let xy = (packed / 1) % 3;
        let zy = (packed / 3) % 3;
        let zx = (packed / 9) % 3;
        Self {
            xy: Aliveness::unconvert(xy),
            zy: Aliveness::unconvert(zy),
            zx: Aliveness::unconvert(zx),
        }
    }
}
