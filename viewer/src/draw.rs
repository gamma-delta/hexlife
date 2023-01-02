mod mnq_bs;
pub use mnq_bs::FiddlyMiniquadBullshit;

use std::f32::consts::TAU;

use crate::{px_to_coord, GameState, SQRT_3};

use hex2d::Coordinate;
use hexlife::math::{Aliveness, EdgePos, HexCoord, RestrictedHexDir};
use macroquad::miniquad as mnq;
use macroquad::prelude::*;

impl GameState {
    pub fn draw(&self) {
        clear_background(Color::from_rgba(0x05, 0x07, 0x10, 0xff));

        self.draw_background();

        let corner_hexpos = self.screen_to_hex(Vec2::ZERO);
        for offset_y in -1..=(screen_height() / self.zoom) as i64 + 1 {
            for offset_x in -1..=(screen_width() / self.zoom) as i64 + 1 {
                let coord_offset = px_to_coord(vec2(offset_x as f32, offset_y as f32), 1.0);
                let coord = corner_hexpos + coord_offset;
                self.draw_edges(coord);
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

    fn draw_background(&self) {
        let mut ctx = unsafe { macroquad::prelude::get_internal_gl() };

        let colors = vec![
            Color::from_vec(Vec4::new(0.01, 0.02, 0.05, 1.0)),
            Color::from_vec(Vec4::new(0.01, 0.02, 0.05, 1.0) * 1.2),
            Color::from_vec(Vec4::new(0.01, 0.02, 0.05, 1.0) * 1.35),
            Color::from_vec(Vec4::new(0.01, 0.02, 0.05, 1.0) * 1.5),
        ];

        let corner_hexpos = self.screen_to_hex(Vec2::ZERO);
        for offset_y in -1..=(screen_height() / self.zoom) as i64 + 1 {
            let mut verts = Vec::new();
            let mut idxes = Vec::new();
            for (idx, offset_x) in (-1..=(screen_width() / self.zoom) as i64 + 1).enumerate() {
                // for (idx, offset_x) in (-1..=0).enumerate() {
                let coord_offset = px_to_coord(vec2(offset_x as f32, offset_y as f32), 1.0);
                let coord = corner_hexpos + coord_offset;

                let color = colors[(coord.x + coord.y * 3).rem_euclid(4) as usize];

                let vert = |px: Vec2| Vertex::new(px.x, px.y, 1.0, 0.0, 0.0, color);
                let anglevert = |twelfths: u8| {
                    let px = self.angled_pos_of(coord, twelfths);
                    vert(px)
                };

                // create the seven points
                verts.push(vert(self.hex_to_screen(coord)));
                verts.push(anglevert(1)); // bottom
                verts.push(anglevert(3)); // bottom
                verts.push(anglevert(5)); // bottom
                verts.push(anglevert(7)); // bottom
                verts.push(anglevert(9)); // bottom
                verts.push(anglevert(11)); // bottom
                let n0 = (7 * idx) as u16;
                #[rustfmt::skip]
                {
                    idxes.push(n0); idxes.push(n0 + 1); idxes.push(n0 + 2);
                    idxes.push(n0); idxes.push(n0 + 2); idxes.push(n0 + 3);
                    idxes.push(n0); idxes.push(n0 + 3); idxes.push(n0 + 4);
                    idxes.push(n0); idxes.push(n0 + 4); idxes.push(n0 + 5);
                    idxes.push(n0); idxes.push(n0 + 5); idxes.push(n0 + 6);
                    idxes.push(n0); idxes.push(n0 + 6); idxes.push(n0 + 1);
                };
            }

            // println!("{:?}\n{:?}", &verts, idxes.chunks(3).collect::<Vec<_>>());
            // panic!();
            ctx.quad_gl.texture(None);
            ctx.quad_gl.draw_mode(DrawMode::Triangles);
            ctx.quad_gl.geometry(&verts, &idxes);
        }

        ctx.flush();
    }

    fn draw_edges(&self, coord: Coordinate<i64>) {
        let px = self.hex_to_screen(coord);
        let edges = self.board.get_edges(coord).unwrap_or_default();
        let mouse_edge = self.mouse_edge();

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
                let color = match edges.get(edge) {
                    _ if mouse_edge == EdgePos::new_raw(coord, edge) => {
                        Some(Color::new(0.2, 0.2, 0.6, 1.0))
                    }
                    Aliveness::Dead => Some(Color::new(0.3, 0.4, 0.6, 1.0)),
                    Aliveness::Barren => Some(Color::new(0.3, 0.0, 0.0, 1.0)),
                    Aliveness::Alive => None,
                };

                if let Some(color) = color {
                    let sx = angle[0].cos() * self.zoom;
                    let sy = -angle[0].sin() * self.zoom;
                    let ex = angle[1].cos() * self.zoom;
                    let ey = -angle[1].sin() * self.zoom;
                    draw_line(sx + px.x, sy + px.y, ex + px.x, ey + px.y, 1.5, color);
                }
            }
        }
        if self.draw_mode.do_connectors() {
            for (edge, angle) in [
                (RestrictedHexDir::XY, TAU * 1.0 / 6.0),
                (RestrictedHexDir::ZY, TAU * 2.0 / 6.0),
                (RestrictedHexDir::ZX, TAU * 3.0 / 6.0),
            ] {
                let pos = EdgePos::new_raw(coord, edge);
                let mouse_here = mouse_edge == pos;

                let color = match (edges.get(edge), mouse_here) {
                    (Aliveness::Dead | Aliveness::Barren, true) => {
                        Some(Color::new(0.4, 0.3, 0.2, 1.0))
                    }
                    (Aliveness::Alive, true) => Some(Color::new(0.7, 0.6, 0.7, 1.0)),
                    (Aliveness::Dead, false) => None,
                    (Aliveness::Barren, false) => Some(Color::new(0.5, 0.2, 0.1, 1.0)),
                    (Aliveness::Alive, false) => Some(Color::new(0.7, 0.6, 0.5, 1.0)),
                };

                if let Some(color) = color {
                    let ex = angle.cos() * self.zoom * SQRT_3;
                    let ey = -angle.sin() * self.zoom * SQRT_3;
                    draw_line(px.x, px.y, ex + px.x, ey + px.y, 2.0, color);
                }
            }
        }
    }

    fn angled_pos_of(&self, pos: HexCoord, twelfths: u8) -> Vec2 {
        let angle = TAU * twelfths as f32 / 12.0;
        let (y, x) = angle.sin_cos();
        let center = self.hex_to_screen(pos);
        center + (vec2(x, -y) * self.zoom)
    }
}
