use minotetris::*;
use std::collections::{ VecDeque, HashMap };

#[derive(PartialEq, Copy, Clone, Debug)]
pub enum PathfinderMove {
    Left,
    Right,
    RotLeft,
    RotRight,
    SonicDrop
}

const MOVES: &[PathfinderMove] = &[
    PathfinderMove::RotLeft,
    PathfinderMove::RotRight,
    PathfinderMove::Left,
    PathfinderMove::Right,
    PathfinderMove::SonicDrop
];

const O_MOVES: &[PathfinderMove] = &[
    PathfinderMove::Left,
    PathfinderMove::Right,
    PathfinderMove::SonicDrop
];

pub struct Moves {
    field: [[[[Option<MoveNode>; 3]; 4]; 40]; 10],
    pub moves: Vec<(Piece, i32)>
}

impl Moves {
    pub fn moves(board: &Board, piece: Piece) -> Self {
        let mut this = Self {
            field: [[[[None; 3]; 4]; 40]; 10],
            moves: Vec::new(),
        };
        let mut locks = HashMap::with_capacity(1024);
        let mut queue = VecDeque::with_capacity(1024);
        *this.get_mut(piece) = Some(MoveNode {
            parent: None,
            mv: PathfinderMove::SonicDrop,
            total_dist: 0,
            dist: 0
        });
        queue.push_back(piece);
        while let Some(parent) = queue.pop_front() {
            let moves = if piece.kind == PieceType::O {
                O_MOVES
            } else {
                MOVES
            };
            for &mv in moves {
                let mut child = parent;
                
                let success = match mv {
                    PathfinderMove::Left => child.move_left(board),
                    PathfinderMove::Right => child.move_right(board),
                    PathfinderMove::RotLeft => child.turn_left(board),
                    PathfinderMove::RotRight => child.turn_right(board),
                    PathfinderMove::SonicDrop => {
                        let mut success = false;
                        while child.soft_drop(board) {
                            success = true;
                        }
                        success
                    }
                };
                if success {
                    let dist = if mv == PathfinderMove::SonicDrop {
                        child.y - parent.y
                    } else {
                        1
                    };
                    let parent_dist = this.get(parent).unwrap().total_dist;
                    let node = MoveNode {
                        parent: Some(parent),
                        mv,
                        dist,
                        total_dist: parent_dist + dist
                    };
                    let entry = this.get_mut(child);
                    if entry.is_none() || node.true_dist() < entry.as_ref().unwrap().true_dist() {
                        *entry = Some(node);
                        queue.push_back(child);
                    }
                }
                if mv == PathfinderMove::SonicDrop {
                    let mut key = [(0, 0); 4];
                    for (dest, &src) in key.iter_mut().zip(child.cells().iter()) {
                        *dest = src;
                    }
                    locks.entry(key)
                        .and_modify(|prev| {
                            let prev_dist = this.get(*prev).unwrap().true_dist();
                            let new_dist = this.get(child).unwrap().true_dist();
                            if new_dist < prev_dist {
                                *prev = child;
                            }
                        })
                        .or_insert(child);
                }
            }
        }
        this.moves = locks
            .values()
            .map(|&mv| (mv, this.get(mv).unwrap().true_dist()))
            .collect();
        this
    }
    fn get(&self, state: Piece) -> &Option<MoveNode> {
        &self.field[state.x as usize][state.y as usize][state.r as usize][state.tspin as usize]
    }
    fn get_mut(&mut self, state: Piece) -> &mut Option<MoveNode> {
        &mut self.field[state.x as usize][state.y as usize][state.r as usize][state.tspin as usize]
    }
    pub fn path(&self, state: Piece) -> VecDeque<PathfinderMove> {
        let mut path = VecDeque::new();
        let mut parent = state;
        let mut skipping = true;
        loop {
            let node = self.get(parent).as_ref().unwrap();
            if let Some(state) = node.parent {
                parent = state;
            } else {
                break;
            }
            if node.mv != PathfinderMove::SonicDrop {
                skipping = false;
            }
            if !skipping {
                path.push_front(node.mv);
            }
        }
        path
    }
}

#[derive(Copy, Clone)]
pub struct MoveNode {
    pub parent: Option<Piece>,
    pub mv: PathfinderMove,
    pub total_dist: i32,
    pub dist: i32
}

impl MoveNode {
    pub fn true_dist(&self) -> i32 {
        let mut dist = self.total_dist;
        if self.mv == PathfinderMove::SonicDrop {
            dist -= self.dist;
        }
        dist
    }
}