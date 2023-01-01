use crate::{GameState, RunState, MOVE_SPEED, SUPER_MOVE_SPEED, SUPER_ZOOM_SPEED, ZOOM_SPEED};

use macroquad::prelude::*;

impl GameState {
    pub fn update(&mut self) {
        self.controls();
        self.tick_board();
    }

    fn tick_board(&mut self) {
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

    fn controls(&mut self) {
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
        self.campos += Vec2::from(delta_view)
            * if is_key_down(KeyCode::LeftShift) {
                SUPER_MOVE_SPEED
            } else {
                MOVE_SPEED
            }
            / self.zoom;

        let mouse_prezoom = self.screen_to_world(mouse_position().into());
        let wheel_y = mouse_wheel().1;
        let zoom_speed = if is_key_down(KeyCode::LeftShift) {
            SUPER_ZOOM_SPEED
        } else {
            ZOOM_SPEED
        };
        if wheel_y < 0.0 {
            self.zoom /= zoom_speed;
        } else if wheel_y > 0.0 {
            self.zoom *= zoom_speed;
        }
        let mouse_postzoom = self.screen_to_world(mouse_position().into());
        // javidx9 is the coolest person alive
        self.campos += mouse_prezoom - mouse_postzoom;

        if is_mouse_button_down(MouseButton::Left) {
            let mouse_edge = self.mouse_edge();
            let state = self
                .drag_state
                .unwrap_or_else(|| self.board.get_liveness(mouse_edge).flip());
            self.board.set_alive(mouse_edge, state);
            self.drag_state = Some(state);
        } else {
            self.drag_state = None;
        }

        if is_key_pressed(KeyCode::R) && is_key_down(KeyCode::LeftShift) {
            self.board.clear();
        }

        if is_key_pressed(KeyCode::Enter) {
            self.draw_mode = self.draw_mode.next();
        }

        if is_key_pressed(KeyCode::Space) {
            self.running = if self.running == RunState::Run {
                RunState::Stopped
            } else {
                RunState::Run
            };
        } else if is_key_pressed(KeyCode::Tab) {
            self.running = RunState::OneStep;
        }
    }
}
