use ggez::{ Context, input::keyboard, event::KeyCode };
use enumset::EnumSet;
use crate::tetris::{ Tetris, TetrisEvent };
use crate::input::*;

pub struct HumanController {
    pub inputs: EnumSet<TetrisInput>,
    pub hold: KeyCode,
    pub left: KeyCode,
    pub right: KeyCode,
    pub rot_left: KeyCode,
    pub rot_right: KeyCode,
    pub hard_drop: KeyCode,
    pub soft_drop: KeyCode
}

impl Default for HumanController {
    fn default() -> Self {
        HumanController {
            inputs: EnumSet::empty(),
            hold: KeyCode::C,
            left: KeyCode::Left,
            right: KeyCode::Right,
            rot_left: KeyCode::Z,
            rot_right: KeyCode::Up,
            hard_drop: KeyCode::Space,
            soft_drop: KeyCode::Down
        }
    }
}

impl TetrisController for HumanController {
    fn update(&mut self, ctx: &Context, _tetris: &mut Tetris, _events: &Vec<TetrisEvent>) {
        self.inputs.clear();
        if keyboard::is_key_pressed(ctx, self.hold) {
            self.inputs.insert(TetrisInput::Hold);
        }
        if keyboard::is_key_pressed(ctx, self.left) {
            self.inputs.insert(TetrisInput::Left);
        }
        if keyboard::is_key_pressed(ctx, self.right) {
            self.inputs.insert(TetrisInput::Right);
        }
        if keyboard::is_key_pressed(ctx, self.rot_left) {
            self.inputs.insert(TetrisInput::RotLeft);
        }
        if keyboard::is_key_pressed(ctx, self.rot_right) {
            self.inputs.insert(TetrisInput::RotRight);
        }
        if keyboard::is_key_pressed(ctx, self.hard_drop) {
            self.inputs.insert(TetrisInput::HardDrop);
        }
        if keyboard::is_key_pressed(ctx, self.soft_drop) {
            self.inputs.insert(TetrisInput::SoftDrop);
        }
    }
    fn inputs(&mut self) -> EnumSet<TetrisInput> {
        self.inputs
    }
}