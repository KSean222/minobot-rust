use ggez::input::keyboard;
use ggez::event::KeyCode;
use ggez::Context;
use enumset::{ EnumSet, EnumSetType };

#[derive(EnumSetType)]
pub enum TetrisInput {
    Hold,
    Left,
    Right,
    RotLeft,
    RotRight,
    HardDrop,
    SoftDrop
}

pub trait TetrisController {
    fn update(&mut self, ctx: &Context);
    fn inputs(&self) -> EnumSet<TetrisInput>;
}

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

impl TetrisController for HumanController {
    fn update(&mut self, ctx: &Context) {
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
    fn inputs(&self) -> EnumSet<TetrisInput> {
        self.inputs
    }
}
