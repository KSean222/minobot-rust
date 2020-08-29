use minotetris::*;
use std::collections::{ VecDeque, HashMap, hash_map::Entry };
use serde::{Serialize, Deserialize};

#[derive(PartialEq, Copy, Clone, Debug)]
pub enum PathfinderMove {
    Left,
    Right,
    RotLeft,
    RotRight,
    SonicDrop
}

const MOVES: [PathfinderMove; 5] = [
    PathfinderMove::RotLeft,
    PathfinderMove::RotRight,
    PathfinderMove::Left,
    PathfinderMove::Right,
    PathfinderMove::SonicDrop
];

const O_MOVES: [PathfinderMove; 3] = [
    PathfinderMove::Left,
    PathfinderMove::Right,
    PathfinderMove::SonicDrop
];

pub struct Pathfinder {
    field: [[[Option<MoveNode>; 4]; 40]; 10]
}
#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
pub struct MoveState {
    pub piece: PieceState,
    pub tspin: TspinType
}

impl Default for Pathfinder {
    fn default() -> Self {
        Pathfinder::new()
    }
}

impl Pathfinder {
    pub fn new() -> Self {
        Pathfinder {
            field: [[[None; 4]; 40]; 10]
        }
    }
    pub fn get_moves(&mut self, board: &mut Board) -> Vec<MoveState> {
        let start_state = MoveState {
            piece: board.state,
            tspin: TspinType::None
        };
        self.field = [[[None; 4]; 40]; 10];
        self.field[start_state.piece.x as usize][start_state.piece.y as usize][start_state.piece.r as usize] = Some(MoveNode {
            parent: None,
            mv: PathfinderMove::SonicDrop,
            tspin: TspinType::None,
            total_dist: 0,
            dist: 0
        });
        let mut locks = HashMap::<[(i32, i32); 4], MoveState>::with_capacity(1024);
        let mut queue = VecDeque::with_capacity(1024);
        queue.push_back(start_state);
        while let Some(parent_state) = queue.pop_front() {
            let parent = self.field[parent_state.piece.x as usize][parent_state.piece.y as usize][parent_state.piece.r as usize].unwrap();
            let moves = if board.current == Tetrimino::O {
                O_MOVES.iter()
            } else {
                MOVES.iter()
            };
            for &mv in moves {
                board.state = parent_state.piece;
                
                let success = match mv {
                    PathfinderMove::Left => board.move_left(),
                    PathfinderMove::Right => board.move_right(),
                    PathfinderMove::RotLeft => board.turn_left(),
                    PathfinderMove::RotRight => board.turn_right(),
                    PathfinderMove::SonicDrop => {
                        let mut success = false;
                        while board.soft_drop() {
                            success = true;
                        }
                        success
                    }
                };
                if success {
                    let x = board.state.x;
                    let y = board.state.y;
                    let r = board.state.r;
                    let dist = if mv == PathfinderMove::SonicDrop {
                        (y - parent_state.piece.y).abs()
                    } else {
                        1
                    };
                    let is_more_important = if let Some(prev) = &self.field[x as usize][y as usize][r as usize] {
                        let mut prev_dist = prev.total_dist;
                        if prev.mv == PathfinderMove::SonicDrop {
                            prev_dist -= prev.dist;
                        }
                        let mut new_dist = parent.total_dist;
                        if mv != PathfinderMove::SonicDrop {
                            new_dist += dist;
                        }
                        new_dist < prev_dist
                    } else {
                        true
                    };
                    if is_more_important {
                        self.field[x as usize][y as usize][r as usize] = Some(MoveNode {
                            parent: Some(parent_state),
                            mv,
                            tspin: TspinType::None,//TODO
                            total_dist: parent.total_dist + dist,
                            dist
                        });
                        queue.push_back(MoveState {
                            piece: board.state,
                            tspin: board.tspin
                        });
                    }
                }
                if mv == PathfinderMove::SonicDrop {
                    let mut key = [(0, 0); 4];
                    for (i, (cell_x, cell_y)) in board.current.cells(board.state.r).iter().enumerate() {
                        key[i] = (cell_x + board.state.x, cell_y + board.state.y);
                    }
                    match locks.entry(key) {
                        Entry::Occupied(mut entry) => {
                            let lock = entry.get_mut();
                            let prev = self.field[lock.piece.x as usize][lock.piece.y as usize][lock.piece.r as usize].unwrap();
                            let new = self.field[board.state.x as usize][board.state.y as usize][board.state.r as usize].unwrap();
                            if new.total_dist < prev.total_dist {
                                *lock = MoveState {
                                    piece: board.state,
                                    tspin: board.tspin
                                };
                            }
                        },
                        Entry::Vacant(entry) => {
                            entry.insert(MoveState {
                                piece: board.state,
                                tspin: board.tspin
                            });
                        }
                    }
                }
            }
        }
        board.state = start_state.piece;
        board.tspin = start_state.tspin;
        locks.values().copied().collect()
    }
    pub fn path_to(&self, x: i32, y: i32, r: u8) -> Option<VecDeque<PathfinderMove>> {
        if let Some(mut node) = self.field[x as usize][y as usize][r as usize] {
            let mut moves = VecDeque::with_capacity(16);
            let mut skipping = true;
            while let Some(parent) = node.parent {
                if node.mv != PathfinderMove::SonicDrop {
                    skipping = false;
                }
                if !skipping {
                    moves.push_front(node.mv);
                }
                node = self.field[parent.piece.x as usize][parent.piece.y as usize][parent.piece.r as usize].unwrap();
            }
            Some(moves)
        } else {
            None
        }
    }
}

#[derive(Copy, Clone)]
pub struct MoveNode {
    pub parent: Option<MoveState>,
    pub mv: PathfinderMove,
    pub tspin: TspinType,
    pub total_dist: i32,
    pub dist: i32
}