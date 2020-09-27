use serde::{ Serialize, Deserialize };

use crate::pathfinder::Moves;
use crate::evaluator::{ Evaluator, StandardEvaluator };
use minotetris::*;

pub struct Bot<E=StandardEvaluator> {
    pub data: BotData<E>,
    pub root: Node,
}

pub struct BotData<E> {
    pub queue: Vec<PieceType>,
    pub settings: BotSettings,
    evaluator: E,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct BotSettings {
    pub use_hold: bool,
    pub exploration_exploitation_constant: f32
}

impl Default for BotSettings {
    fn default() -> Self {
        BotSettings {
            use_hold: true,
            exploration_exploitation_constant: std::f32::consts::SQRT_2
        }
    }
}

impl<E: Evaluator> Bot<E> {
    pub fn new(board: Board, evaluator: E, settings: BotSettings) -> Self {
        Bot {
            data: BotData {
                queue: Vec::new(),
                evaluator,
                settings
            },
            root: Node::root(board),
        }
    }
    pub fn update_queue(&mut self, mino: PieceType) {
        self.data.queue.push(mino);
    }
    pub fn reset(&mut self, board: Board) {
        self.root = Node::root(board);
    }
    pub fn think(&mut self) -> bool {
        self.root.update(&mut self.data);
        self.root.finished
    }
    pub fn next_move(&mut self) -> Option<&Node> {
        self.data.queue.remove(0);
        let root = self.root.children
            .drain(..)
            .max_by_key(|c| c.total_value());
        if let Some(root) = root {
            self.root = root;
            self.root.advance();
            // for &row in self.root.board.rows().iter().rev().skip(20) {
            //     for x in 0..10 {
            //         print!("{}", if row.get(x) {
            //             "[]"
            //         } else {
            //             ".."
            //         });
            //     }
            //     println!()
            // }
            // println!("Hold: {:?}, Queue: {:?}", self.root.board.hold, self.data.queue);
            // println!();
            Some(&self.root)
        } else {
            None
        }
    }
}

#[derive(Debug)]
pub struct Node {
    pub board: Board,
    pub mv: Piece,
    pub move_dist: i32,
    pub lock: LockResult,
    pub uses_hold: bool,

    pub children: Vec<Node>,
    pub value: i32,
    pub reward: i32,
    pub max_child_reward: i32,
    pub visits: u32,
    pub finished: bool,
    pub depth: u32
}

impl Node {
    fn total_value(&self) -> i32 {
        self.value.saturating_add(self.reward).saturating_add(self.max_child_reward)
    }
    fn root(board: Board) -> Self {
        Self {
            board,
            children: Vec::new(),
            mv: Piece {
                kind: PieceType::O,
                x: 0,
                y: 0,
                r: 0,
                tspin: TspinType::None
            },
            move_dist: 0,
            lock: LockResult {
                lines_cleared: 0,
                block_out: false,
                combo: 0,
                b2b_bonus: false
            },
            value: std::i32::MIN,
            reward: 0,
            max_child_reward: 0,
            visits: 1,
            uses_hold: false,
            finished: false,
            depth: 0
        }
    }
    fn update<E: Evaluator>(&mut self, data: &BotData<E>) -> ((i32, i32), u32) {
        let mut child_index = None;
        let mut score = std::f32::NEG_INFINITY;
        for (i, c) in self.children.iter().enumerate() {
            if !c.finished {
                let child_score =
                    (i as f32) / (self.children.len() as f32) +
                    data.settings.exploration_exploitation_constant * 
                    ((self.visits as f32).ln() / (c.visits as f32)).sqrt();
                if child_score > score {
                    child_index = Some(i);
                    score = child_score;
                }
            }
        }
        if let Some(child_index) = child_index {
            let ((value, reward), visits) = self.children[child_index].update(data);
            let child = self.children.remove(child_index);
            let child_index = self.children
                .iter()
                .position(|c| c.total_value() > child.total_value())
                .unwrap_or(self.children.len());
            self.children.insert(child_index, child);
            if value.saturating_add(reward) > self.value.saturating_add(self.max_child_reward) {
                self.value = value;
                self.max_child_reward = reward;
            }
            self.visits += visits;
            ((value, self.reward.saturating_add(reward)), visits)
        } else if self.children.is_empty() {
            self.expand(data)
        } else {
            self.finished = true;
            ((std::i32::MIN, 0), 0)
        }
    }
    fn expand<E: Evaluator>(&mut self, data: &BotData<E>) -> ((i32, i32), u32) {
        if self.depth as usize >= data.queue.len() {
            self.finished = true;
            return ((std::i32::MIN, 0), 0);
        }
        let piece = Piece::spawn(&self.board, data.queue[self.depth as usize]);
        for mv in Moves::moves(&self.board, piece).moves {
            self.create_child(data, mv, false);
        }
        if data.settings.use_hold {
            let mut hold_board = self.board.clone();
            let piece_type = hold_board.hold
                .replace(data.queue[self.depth as usize])
                .or(data.queue.get((self.depth + 1) as usize).copied());
            if let Some(piece_type) = piece_type {
                let piece = Piece::spawn(&self.board, piece_type);
                for mv in Moves::moves(&hold_board, piece).moves {
                    self.create_child(data, mv, true);
                }
            }
        }
        if self.children.is_empty() {
            self.finished = true;
            ((std::i32::MIN, 0), 0)
        } else {
            self.children.sort_unstable_by_key(|c| c.total_value());
            let best = self.children.last().unwrap();
            let visits = self.children.len() as u32;
            self.value = best.value;
            self.max_child_reward = best.reward;
            self.visits += visits;
            ((best.value, self.reward.saturating_add(best.reward)), visits)
        }
    }
    fn create_child<E: Evaluator>(&mut self, data: &BotData<E>, (mv, move_dist): (Piece, i32), uses_hold: bool) {
        let mut board = self.board.clone();
        let mut child_depth = self.depth;
        if uses_hold {
            if board.hold.replace(data.queue[child_depth as usize]).is_none() {
                child_depth += 1;
            }
        }
        let lock = board.lock_piece(mv);
        child_depth += 1;
        let mut child = Node {
            board,
            mv,
            move_dist,
            lock,
            children: Vec::new(),
            depth: child_depth,
            value: 0,
            reward: 0,
            max_child_reward: 0,
            visits: 1,
            uses_hold,
            finished: child_depth as usize >= data.queue.len()
        };
        let (value, reward) = data.evaluator.evaluate(&child, &data.queue);
        child.value = value;
        child.reward = reward;
        self.children.push(child);
    }
    fn advance(&mut self){
        self.finished = false;
        self.depth -= 1;
        for c in self.children.iter_mut() {
            c.advance();
        }
    }
}
