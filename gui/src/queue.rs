use std::collections::VecDeque;

use minotetris::*;
use enumset::EnumSet;
use arrayvec::ArrayVec;
use rand::prelude::*;

pub struct PieceQueue {
    queue: VecDeque<PieceType>,
    bag: ArrayVec<[PieceType; 7]>,
}

fn random_bag(rng: &mut (impl Rng + ?Sized)) -> ArrayVec<[PieceType; 7]> {
    let mut bag: ArrayVec<[PieceType; 7]> = EnumSet::all().into_iter().collect();
    bag.shuffle(rng);
    bag
}

impl PieceQueue {
    pub fn new(len: usize, rng: &mut (impl Rng + ?Sized)) -> Self {
        let mut queue = Self {
            queue: VecDeque::with_capacity(len + 1),
            bag: random_bag(rng)
        };
        for _ in 0..len {
            queue.queue_from_bag(rng);
        }
        queue
    }

    pub fn next(&mut self, rng: &mut (impl Rng + ?Sized)) -> PieceType {
        self.queue_from_bag(rng);
        self.queue.pop_front().unwrap()
    }

    fn queue_from_bag(&mut self, rng: &mut (impl Rng + ?Sized)) {
        if self.bag.is_empty() {
            self.bag = random_bag(rng);
        }
        self.queue.push_back(self.bag.pop().unwrap());
    }

    pub fn get_queue(&self) -> &VecDeque<PieceType> {
        &self.queue
    }
}