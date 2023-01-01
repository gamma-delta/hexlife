//! Math and coordinate primitives.

use enumflags2::{bitflags, BitFlags};
use hex2d::{Coordinate, Direction};

/// Because (say) the bottom edge of a hexagon is the same as the top edge of
/// the hex below it, internally we only store the coordinate and whether it has
/// a connection in the `XY`, `ZY`, and `ZX` dirs.
pub type HexEdges = BitFlags<RestrictedHexDir>;

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
#[bitflags]
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
