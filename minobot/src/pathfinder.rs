use minotetris::*;
use std::collections::{ VecDeque, HashMap, hash_map::Entry };

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

pub struct Pathfinder {
    field: [[[Option<MoveNode>; 4]; 40]; 10]
}

impl Pathfinder {
    pub fn new() -> Self {
        Pathfinder {
            field: [[[None; 4]; 40]; 10]
        }
    }
    pub fn get_moves(&mut self, board: &mut Board) -> Vec<PieceState> {
        let start_state = board.state;
        self.field = [[[None; 4]; 40]; 10];
        self.field[start_state.x as usize][start_state.y as usize][start_state.r as usize] = Some(MoveNode {
            parent: None,
            mv: PathfinderMove::SonicDrop,
            tspin: TspinType::None,
            total_dist: 0,
            dist: 0
        });
        let mut locks = HashMap::<[(i32, i32); 4], PieceState>::with_capacity(1024);
        let mut queue = VecDeque::with_capacity(1024);
        queue.push_back(start_state);
        while let Some(parent_state) = queue.pop_front() {
            let parent = self.field[parent_state.x as usize][parent_state.y as usize][parent_state.r as usize].unwrap();
            for mv in &MOVES {
                let mv = *mv;
                board.state = parent_state;
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
                        (y - parent_state.y).abs()
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
                        queue.push_back(board.state);
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
                            let prev = self.field[lock.x as usize][lock.y as usize][lock.r as usize].unwrap();
                            let new = self.field[board.state.x as usize][board.state.y as usize][board.state.r as usize].unwrap();
                            if new.total_dist < prev.total_dist {
                                *lock = board.state;
                            }
                        },
                        Entry::Vacant(entry) => {
                            entry.insert(board.state);
                        }
                    }
                }
            }
        }
        board.state = start_state;
        locks.values().map(|x| *x).collect()
    }
    pub fn path_to(&self, x: i32, y: i32, r: u8) -> VecDeque<PathfinderMove> {
        let mut node = self.field[x as usize][y as usize][r as usize].unwrap();
        let mut moves = VecDeque::with_capacity(16);
        let mut skipping = true;
        while let Some(parent) = node.parent {
            if node.mv != PathfinderMove::SonicDrop {
                skipping = false;
            }
            if !skipping {
                moves.push_front(node.mv);
            }
            node = self.field[parent.x as usize][parent.y as usize][parent.r as usize].unwrap();
        }
        moves
    }
}

#[derive(Copy, Clone)]
pub struct MoveNode {
    pub parent: Option<PieceState>,
    pub mv: PathfinderMove,
    pub tspin: TspinType,
    pub total_dist: i32,
    pub dist: i32
}