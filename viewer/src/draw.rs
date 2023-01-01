use std::f32::consts::TAU;

use crate::{px_to_coord, GameState, SQRT_3};

use hexlife::math::RestrictedHexDir;
use macroquad::prelude::*;

impl GameState {
    pub fn draw(&self) {
        clear_background(Color::from_rgba(0x05, 0x07, 0x10, 0xff));

        let mouse_hexpos = self.screen_to_hex(mouse_position().into());

        enum DrawStage {
            Background,
            Edges,
        }

        let corner_hexpos = self.screen_to_hex(Vec2::ZERO);

        for stage in [DrawStage::Background, DrawStage::Edges] {
            // it appears range_iter is bugged and only produces coords around 0
            for offset_x in -1..=(screen_width() / self.zoom) as i64 + 1 {
                for offset_y in -1..=(screen_height() / self.zoom) as i64 + 1 {
                    let coord_offset = px_to_coord(vec2(offset_x as f32, offset_y as f32), 1.0);

                    let coord = corner_hexpos + coord_offset;
                    let px = self.hex_to_screen(coord);

                    match stage {
                        DrawStage::Background => self.draw_background(coord, mouse_hexpos, px),
                        DrawStage::Edges => self.draw_edges(coord, px),
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
        draw_text(
            &format!("{} FPS", get_fps()),
            12.0,
            12.0 + 18.0,
            16.0,
            WHITE,
        );
    }

    fn draw_edges(&self, coord: hex2d::Coordinate<i64>, px: Vec2) {
        let edges = self.board.get_edges(coord).unwrap_or_default();

        if self.draw_mode.do_edges() {
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
        }
        if self.draw_mode.do_connectors() {
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

    fn draw_background(
        &self,
        coord: hex2d::Coordinate<i64>,
        mouse_hexpos: hex2d::Coordinate<i64>,
        px: Vec2,
    ) {
        let color = if coord == mouse_hexpos {
            Color::new(0.15, 0.4, 0.3, 1.0)
        } else {
            Color::from_vec(
                Vec4::new(0.01, 0.02, 0.05, 1.0)
                    * [1.0, 1.25, 1.5, 1.75][(coord.x + coord.y * 3).rem_euclid(4) as usize],
            )
        };
        draw_poly(px.x, px.y, 6, self.zoom, 360.0 / 12.0, color);
        if is_key_down(KeyCode::LeftControl) {
            draw_text(
                &format!("{},{}", coord.x, coord.y),
                px.x - self.zoom / 2.0,
                px.y - self.zoom / 4.0,
                self.zoom / 2.0,
                WHITE,
            );
        }
    }
}
