use rand::prelude::*;
use enumset::EnumSet;

mod game;
pub use game::*;

pub struct TetrisBattle {
    p1: TetrisGame,
    p2: TetrisGame
}

impl TetrisBattle {
    pub fn new(
        p1_config: TetrisGameConfig,
        p1_rng: &mut (impl Rng + ?Sized),
        p2_config: TetrisGameConfig,
        p2_rng: &mut (impl Rng + ?Sized)
    ) -> Self {
        Self {
            p1: TetrisGame::new(p1_config, p1_rng),
            p2: TetrisGame::new(p2_config, p2_rng)
        }
    }

    pub fn update(
        &mut self,
        p1_inputs: EnumSet<TetrisInput>,
        p1_rng: &mut (impl Rng + ?Sized),
        p2_inputs: EnumSet<TetrisInput>,
        p2_rng: &mut (impl Rng + ?Sized),
        garbage_rng: &mut (impl Rng + ?Sized)
    ) -> (Vec<TetrisGameEvent>, Vec<TetrisGameEvent>) {
        let p1_events = self.p1.update(p1_inputs, p1_rng, garbage_rng);
        let p2_events = self.p2.update(p2_inputs, p2_rng, garbage_rng);
        for event in &p1_events {
            if let &TetrisGameEvent::GarbageSent(garbage) = event {
                self.p2.add_garbage(garbage);
            }
        }
        for event in &p2_events {
            if let &TetrisGameEvent::GarbageSent(garbage) = event {
                self.p1.add_garbage(garbage);
            }
        }
        (p1_events, p2_events)
    }
}
