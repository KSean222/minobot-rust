use crate::*;
use rand::SeedableRng;
use rand::Rng;
use rand_pcg::Pcg32;
use std::collections::vec_deque::VecDeque;
use enumset::EnumSet;
use arrayvec::ArrayVec;

pub trait PieceQueue {
    fn take(&mut self) -> PieceType;
    fn get(&self, index: i32) -> PieceType;
    fn max_previews(&self) -> i32;
}

pub struct RandomPieceQueue {
    rng: Pcg32,
    bag: EnumSet<PieceType>,
    queue: VecDeque<PieceType>,
    previews: i32
}

impl RandomPieceQueue {
    pub fn new(seed: [u8; 16], previews: i32) -> Self {
        let mut queue = RandomPieceQueue {
            rng: Pcg32::from_seed(seed),
            bag: EnumSet::all(),
            queue: VecDeque::with_capacity(previews as usize),
            previews
        };
        for _ in 0..previews {
            queue.add_piece();
        }
        queue
    }
    fn add_piece(&mut self) {
        if self.bag.is_empty() {
            self.bag = EnumSet::all();
        }
        let bag: ArrayVec<[PieceType; 7]> = self.bag.iter().collect();
        let mino = bag[self.rng.gen_range(0, bag.len())];
        self.bag.remove(mino);
        self.queue.push_back(mino);
    }
}

impl PieceQueue for RandomPieceQueue {
    fn take(&mut self) -> PieceType {
        let piece = self.queue.pop_front().unwrap();
        self.add_piece();
        piece
    }
    fn get(&self, index: i32) -> PieceType {
        *self.queue.get(index as usize).unwrap()
    }
    fn max_previews(&self) -> i32 {
        self.previews
    }
}
