use std::f32::consts::TAU;

use hex2d::Direction;
use hexlife::{
    math::{EdgePos, HexCoord, RestrictedHexDir},
    Board, Rule,
};
use macroquad::prelude::*;

const ZOOM_SPEED: f32 = 1.1;
const MOVE_SPEED: f32 = 8.0;

const SQRT_3: f32 = 1.7320508;

struct GameState {
    board: Board,
    rule: Rule,
    running: RunState,

    // Transform is: worldspace - pos * zoom = screenspace
    // (no operator precedence)
    // Campos is in screen space
    campos: Vec2,
    zoom: f32,
}

impl GameState {
    fn update(&mut self) {
        let wheel_y = mouse_wheel().1;
        if wheel_y < 0.0 {
            self.zoom /= ZOOM_SPEED;
        } else if wheel_y > 0.0 {
            self.zoom *= ZOOM_SPEED;
        }

        let mut delta_view = (0.0, 0.0);
        if is_key_down(KeyCode::W) {
            delta_view.1 -= 1.0;
        }
        if is_key_down(KeyCode::S) {
            delta_view.1 += 1.0;
        }
        if is_key_down(KeyCode::A) {
            delta_view.0 -= 1.0;
        }
        if is_key_down(KeyCode::D) {
            delta_view.0 += 1.0;
        }
        self.campos += Vec2::from(delta_view) * MOVE_SPEED;

        if is_key_pressed(KeyCode::Space) {
            self.running = if self.running == RunState::Run {
                RunState::Stopped
            } else {
                RunState::Run
            };
        } else if is_key_pressed(KeyCode::Enter) {
            self.running = RunState::OneStep;
        }

        let mouse_hexpos = px_to_coord(Vec2::from(mouse_position()) + self.campos, self.zoom);
        if is_mouse_button_pressed(MouseButton::Left) {
            let ideal_mousepos = coord_to_px(mouse_hexpos, self.zoom);
            let delta = Vec2::from(mouse_position()) + self.campos - ideal_mousepos;
            let angle = delta.y.atan2(delta.x);
            let clean_angle = ((angle / TAU) * 6.0).round() as i32;
            let dir = Direction::from_int(1 - clean_angle);
            let edgepos = EdgePos::new(mouse_hexpos, dir);
            self.board.toggle_alive(edgepos);
        }

        match self.running {
            RunState::Stopped => {}
            RunState::OneStep => {
                self.board.apply_rule(self.rule);
                self.running = RunState::Stopped;
            }
            RunState::Run => {
                self.board.apply_rule(self.rule);
            }
        }
    }

    fn draw(&self) {
        clear_background(Color::from_rgba(0x05, 0x07, 0x10, 0xff));

        let mouse_hexpos = px_to_coord(Vec2::from(mouse_position()) + self.campos, self.zoom);

        enum DrawStage {
            Background,
            Edges,
        }

        let scrw_half = screen_width() / 2.0;
        let scrh_half = screen_height() / 2.0;
        let corner_dist = (scrw_half * scrw_half + scrh_half * scrh_half).sqrt();
        let corner_hex_dist = corner_dist / self.zoom;
        let center_hexpos = px_to_coord(self.campos + vec2(scrw_half, scrh_half), self.zoom);

        for stage in [DrawStage::Background, DrawStage::Edges] {
            // it appears range_iter is bugged and only produces coords around 0
            for coord_offset in HexCoord::new(0, 0).range_iter(corner_hex_dist.round() as i64 + 1) {
                let coord = center_hexpos + coord_offset;
                let px = coord_to_px(coord, self.zoom)
                    - vec2(self.campos.x.trunc(), self.campos.y.trunc());

                match stage {
                    DrawStage::Background => {
                        let color = if coord == mouse_hexpos {
                            Color::new(0.15, 0.4, 0.3, 1.0)
                        } else {
                            Color::from_vec(
                                Vec4::new(0.01, 0.02, 0.05, 1.0)
                                    * [1.0, 1.25, 1.5, 1.75]
                                        [(coord.x + coord.y * 3).rem_euclid(4) as usize],
                            )
                        };
                        draw_poly(px.x, px.y, 6, self.zoom, 360.0 / 12.0, color);

                        if is_key_down(KeyCode::LeftShift) {
                            draw_text(
                                &format!("{},{}", coord.x, coord.y),
                                px.x - self.zoom / 2.0,
                                px.y - self.zoom / 4.0,
                                self.zoom / 2.0,
                                WHITE,
                            );
                        }
                    }
                    DrawStage::Edges => {
                        let edges = self.board.get_edges(coord).unwrap_or_default();
                        for (angle, edge) in [
                            TAU * 1.0 / 12.0,
                            TAU * 3.0 / 12.0,
                            TAU * 5.0 / 12.0,
                            TAU * 7.0 / 12.0,
                        ]
                        .windows(2)
                        .zip([
                            RestrictedHexDir::XY,
                            RestrictedHexDir::ZY,
                            RestrictedHexDir::ZX,
                        ]) {
                            if !edges.contains(edge) {
                                let sx = angle[0].cos() * self.zoom;
                                let sy = -angle[0].sin() * self.zoom;
                                let ex = angle[1].cos() * self.zoom;
                                let ey = -angle[1].sin() * self.zoom;
                                draw_line(
                                    sx + px.x,
                                    sy + px.y,
                                    ex + px.x,
                                    ey + px.y,
                                    1.5,
                                    Color::new(0.3, 0.4, 0.6, 1.0),
                                );
                            }
                        }
                        for (dir, angle) in [
                            (RestrictedHexDir::XY, TAU * 1.0 / 6.0),
                            (RestrictedHexDir::ZY, TAU * 2.0 / 6.0),
                            (RestrictedHexDir::ZX, TAU * 3.0 / 6.0),
                        ] {
                            if edges.contains(dir) {
                                let ex = angle.cos() * self.zoom * SQRT_3;
                                let ey = -angle.sin() * self.zoom * SQRT_3;
                                draw_line(
                                    px.x,
                                    px.y,
                                    ex + px.x,
                                    ey + px.y,
                                    1.5,
                                    Color::new(0.7, 0.6, 0.5, 1.0),
                                );
                            }
                        }
                    }
                }
            }
        }

        draw_text(
            &format!("{}; {:?}", self.rule, self.running),
            12.0,
            12.0,
            16.0,
            WHITE,
        );
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
    let mut state = GameState {
        board: Board::new(),
        rule: Rule::new_raw(0b0001000, 0b0001100),
        running: RunState::Stopped,

        campos: Vec2::ZERO,
        zoom: 32.0,
    };

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
