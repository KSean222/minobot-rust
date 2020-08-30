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
    pub moves: Vec<PieceState>
}

impl Moves {
    pub fn moves(board: Board) -> Self {
        let mut moves = Self {
            field: [[[[None; 3]; 4]; 40]; 10],
            moves: Vec::new(),
        };
        moves.init_moves(board);
        moves
    }
    fn init_moves(&mut self, mut board: Board) {
        let mut locks = HashMap::new();
        let mut queue = VecDeque::new();
        *self.get_mut(board.state) = Some(MoveNode {
            parent: None,
            mv: PathfinderMove::SonicDrop,
            total_dist: 0,
            dist: 0
        });
        queue.push_back(board.state);
        while let Some(parent) = queue.pop_front() {
            let moves = if board.current == Tetrimino::O {
                O_MOVES
            } else {
                MOVES
            };
            for &mv in moves {
                board.state = parent;
                
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
                    let dist = if mv == PathfinderMove::SonicDrop {
                        board.state.y - parent.y
                    } else {
                        1
                    };
                    let parent_dist = self.get(parent).unwrap().total_dist;
                    let node = MoveNode {
                        parent: Some(parent),
                        mv,
                        dist,
                        total_dist: parent_dist + dist
                    };
                    let entry = self.get_mut(board.state);
                    if entry.is_none() || node.true_dist() < entry.as_ref().unwrap().true_dist() {
                        *entry = Some(node);
                        queue.push_back(board.state);
                    }
                }
                if mv == PathfinderMove::SonicDrop {
                    let mut key = [(0, 0); 4];
                    let cells = board.current.cells(board.state.r);
                    let cells = cells
                        .iter()
                        .map(|&(x, y)| (board.state.x + x, board.state.y + y));
                    for (dest, src) in key.iter_mut().zip(cells) {
                        *dest = src;
                    }
                    locks.entry(key)
                        .and_modify(|prev| {
                            let prev_dist = self.get(*prev).unwrap().true_dist();
                            let new_dist = self.get(board.state).unwrap().true_dist();
                            if new_dist < prev_dist {
                                *prev = board.state;
                            }
                        })
                        .or_insert(board.state);
                }
            }
        }
        self.moves = locks.values().copied().collect();
    }
    fn get(&self, state: PieceState) -> &Option<MoveNode> {
        &self.field[state.x as usize][state.y as usize][state.r as usize][state.tspin as usize]
    }
    fn get_mut(&mut self, state: PieceState) -> &mut Option<MoveNode> {
        &mut self.field[state.x as usize][state.y as usize][state.r as usize][state.tspin as usize]
    }
    pub fn path(&self, state: PieceState) -> Vec<PathfinderMove> {
        let mut path = Vec::new();
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
                path.push(node.mv);
            }
        }
        path.reverse();
        path
    }
}

#[derive(Copy, Clone)]
pub struct MoveNode {
    pub parent: Option<PieceState>,
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