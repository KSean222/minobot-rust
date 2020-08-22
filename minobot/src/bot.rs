use serde::{ Serialize, Deserialize };

use crate::pathfinder::Pathfinder;
use crate::evaluator::{ Evaluator, StandardEvaluator };
use minotetris::*;

pub struct Bot<E=StandardEvaluator> {
    pub data: BotData<E>,
    pub root: Node,
}

pub struct BotData<E> {
    pub queue: Vec<Tetrimino>,
    pub settings: BotSettings,
    pathfinder: Pathfinder,
    evaluator: E,
}

#[derive(Serialize, Deserialize)]
pub struct BotSettings {
    pub use_hold: bool
}

impl Default for BotSettings {
    fn default() -> Self {
        BotSettings {
            use_hold: true
        }
    }
}

impl<E: Evaluator> Bot<E> {
    pub fn new(board: Board, evaluator: E, settings: BotSettings) -> Self {
        Bot {
            data: BotData {
                queue: Vec::new(),
                pathfinder: Pathfinder::new(),
                evaluator,
                settings
            },
            root: Node::root(board),
        }
    }
    pub fn update_queue(&mut self, mino: Tetrimino) {
        self.data.queue.push(mino);
    }
    pub fn reset(&mut self, board: Board, queue: Vec<Tetrimino>) {
        self.root = Node::root(board);
        self.data.queue = queue;
    }
    pub fn think(&mut self) -> bool {
        self.root.update(&mut self.data);
        self.root.finished
    }
    pub fn next_move(&mut self) -> Option<&Node> {
        self.data.queue.remove(0);
        let root = self.root.children
            .drain(..)
            .max_by_key(|c| c.visits);
        if let Some(root) = root {
            self.root = root;
            self.root.advance();
            Some(&self.root)
        } else {
            None
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Node {
    pub board: Board,
    pub mv: PieceState,
    pub lock: HardDropResult,
    pub uses_hold: bool,

    pub children: Vec<Node>,
    pub value: f64,
    pub reward: f64,
    pub max_child_reward: f64,
    pub visits: u32,
    pub finished: bool,
    pub depth: u32
}

impl Node {
    fn root(board: Board) -> Self {
        Self {
            board,
            children: Vec::new(),
            mv: PieceState {
                x: 0,
                y: 0,
                r: 0
            },
            lock: HardDropResult {
                lines_cleared: 0,
                block_out: false
            },
            value: 0.0,
            reward: 0.0,
            max_child_reward: 0.0,
            visits: 1,
            uses_hold: false,
            finished: false,
            depth: 0
        }
    }
    fn update<E: Evaluator>(&mut self, data: &mut BotData<E>) -> ((f64, f64), u32) {
        let mut child_index = None;
        let mut score = std::f64::NEG_INFINITY;
        for (i, c) in self.children.iter().enumerate() {
            if !c.finished {
                use std::f64::consts::SQRT_2;
                // let child_score = c.score / (c.sims as f64) + 1.0 * SQRT_2
                //     * ((node.sims as f64).ln() / (c.sims as f64)).sqrt();
                let child_score = (i as f64) / (self.children.len() as f64) +
                    SQRT_2 * ((self.visits as f64).ln() / (c.visits as f64)).sqrt();
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
                .position(|c| c.value + c.reward > child.value + child.reward)
                .unwrap_or(self.children.len());
            self.children.insert(child_index, child);
            if value + reward > self.value + self.max_child_reward {
                self.value = value;
                self.max_child_reward = reward;
            }
            self.visits += visits;
            ((value, self.reward + reward), visits)
        } else if self.children.is_empty() {
            self.expand(data)
        } else {
            self.finished = true;
            ((0.0, 0.0), 0)
        }
    }
    fn expand<E: Evaluator>(&mut self, data: &mut BotData<E>) -> ((f64, f64), u32) {
        for mv in data.pathfinder.get_moves(&mut self.board) {
            self.create_child(data, mv, false);
        }
        if data.settings.use_hold {
            let mut hold_board = self.board.clone();
            let used = if hold_board.hold.is_none() { 1 } else { 0 };
            hold_board.hold_piece(data.queue[self.depth as usize]);
            if ((self.depth + used) as usize) < data.queue.len() {
                for mv in data.pathfinder.get_moves(&mut hold_board) {
                    self.create_child(data, mv, true);
                }
            }
        }
        if self.children.is_empty() {
            self.finished = true;
            ((std::f64::NEG_INFINITY, std::f64::NEG_INFINITY), 0)
        } else {
            self.children.sort_unstable_by(|a, b| {
                (a.value + a.reward).partial_cmp(&(b.value + b.reward)).unwrap()
            });
            let best = self.children.last().unwrap();
            let visits = self.children.len() as u32;
            self.value = best.value;
            self.max_child_reward = best.reward;
            self.visits += visits;
            ((best.value, self.reward + best.reward), visits)
        }
    }
    fn create_child<E: Evaluator>(&mut self, data: &mut BotData<E>, mv: PieceState, uses_hold: bool) {
        let mut board = self.board.clone();
        let mut child_depth = self.depth;
        if uses_hold {
            let used = board.hold.is_none();
            board.hold_piece(data.queue[child_depth as usize]);
            if used {
                child_depth += 1;
            }
        }
        board.state = mv;
        let lock = board.hard_drop(data.queue[child_depth as usize]);
        child_depth += 1;
        let mut child = Node {
            board,
            mv,
            lock,
            children: Vec::new(),
            depth: child_depth,
            value: 0.0,
            reward: 0.0,
            max_child_reward: 0.0,
            visits: 1,
            uses_hold,
            finished: child_depth as usize >= data.queue.len()
        };
        let (value, reward) = data.evaluator.evaluate(&child, self);
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
