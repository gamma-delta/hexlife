mod draw;
mod update;

use std::f32::consts::TAU;

use hex2d::Direction;
use hexlife::{
    math::{EdgePos, HexCoord},
    Board, NeighborRegion, Rule,
};
use macroquad::prelude::*;

const ZOOM_SPEED: f32 = 1.1;
const SUPER_ZOOM_SPEED: f32 = 1.5;
// Pick weird numbers to prevent the hexes moving an integer number
const MOVE_SPEED: f32 = 17.0;
const SUPER_MOVE_SPEED: f32 = 43.0;

const SQRT_3: f32 = 1.7320508;

struct GameState {
    board: Board,
    rule: Rule,
    running: RunState,

    /// Allow click and drag for edges but prevent flickering
    prev_clicked_edge: Option<EdgePos>,

    campos: Vec2,
    zoom: f32,
    draw_mode: DrawMode,
}

impl GameState {
    fn new() -> Self {
        Self {
            board: Board::new(),
            rule: Rule::new_raw(0b0001000, 0b0011000, NeighborRegion::Ten),
            running: RunState::Stopped,
            prev_clicked_edge: None,

            campos: Vec2::ZERO,
            zoom: 48.0,
            draw_mode: DrawMode::Both,
        }
    }

    // https://www.youtube.com/watch?v=ZQ8qtAizis4
    fn world_to_screen(&self, px: Vec2) -> Vec2 {
        (px - self.campos) * self.zoom
    }

    fn screen_to_world(&self, px: Vec2) -> Vec2 {
        px / self.zoom + self.campos
    }

    fn screen_to_hex(&self, px: Vec2) -> HexCoord {
        px_to_coord(self.screen_to_world(px), 1.0)
    }

    fn hex_to_screen(&self, coord: HexCoord) -> Vec2 {
        self.world_to_screen(coord_to_px(coord, 1.0))
    }

    fn mouse_edge(&self) -> EdgePos {
        let mouse_world = self.screen_to_world(mouse_position().into());
        let mouse_hexpos = self.screen_to_hex(mouse_position().into());
        let ideal_mousepos = coord_to_px(mouse_hexpos, 1.0);
        let delta = mouse_world - ideal_mousepos;
        let angle = delta.y.atan2(delta.x);
        let clean_angle = ((angle / TAU) * 6.0).round() as i32;
        let dir = Direction::from_int(1 - clean_angle);
        EdgePos::new(mouse_hexpos, dir)
    }
}

fn config() -> Conf {
    Conf {
        window_title: "HexLife".to_string(),
        ..Default::default()
    }
}

#[macroquad::main(config)]
async fn main() {
    let mut state = GameState::new();

    loop {
        state.update();
        state.draw();
        next_frame().await
    }
}

pub fn px_to_coord(px: Vec2, zoom: f32) -> HexCoord {
    let qf = (SQRT_3 / 3.0 * px.x - px.y / 3.0) / zoom;
    let rf = 2.0 / 3.0 * px.y / zoom;
    let q = qf.round() as i64;
    let r = rf.round() as i64;
    let qf = qf - q as f32;
    let rf = rf - r as f32;
    if q.abs() > r.abs() {
        HexCoord::new(q + (qf + rf / 2.0).round() as i64, r)
    } else {
        HexCoord::new(q, r + (rf + qf / 2.0).round() as i64)
    }
}

pub fn coord_to_px(coord: HexCoord, zoom: f32) -> Vec2 {
    Vec2::new(
        SQRT_3 * coord.x as f32 + SQRT_3 / 2.0 * coord.y as f32,
        1.5 * coord.y as f32,
    ) * zoom
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum RunState {
    Stopped,
    OneStep,
    Run,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum DrawMode {
    Edges,
    Connectors,
    Both,
}

impl DrawMode {
    fn next(&self) -> DrawMode {
        match self {
            DrawMode::Edges => DrawMode::Connectors,
            DrawMode::Connectors => DrawMode::Both,
            DrawMode::Both => DrawMode::Edges,
        }
    }

    fn do_edges(&self) -> bool {
        match self {
            DrawMode::Edges | DrawMode::Both => true,
            DrawMode::Connectors => false,
        }
    }

    fn do_connectors(&self) -> bool {
        match self {
            DrawMode::Connectors | DrawMode::Both => true,
            DrawMode::Edges => false,
        }
    }
}
