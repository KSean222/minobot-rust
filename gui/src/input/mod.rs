use ggez::Context;
use enumset::{ EnumSet, EnumSetType };
use crate::tetris::{ Tetris, TetrisEvent };

mod bot_input;
pub use bot_input::*;
mod human_input;
pub use human_input::*;

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
    fn update(&mut self, ctx: &Context, tetris: &mut Tetris, events: &Vec<TetrisEvent>);
    fn inputs(&mut self) -> EnumSet<TetrisInput>;
}
