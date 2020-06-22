use serde::{ Serialize, Deserialize };

use crate::pathfinder::Pathfinder;
use crate::evaluator::{ Evaluator, StandardEvaluator };
use minotetris::*;

pub struct Bot<T=StandardEvaluator> {
    pub queue: Vec<Tetrimino>,
    pub root: Option<Node>,
    pathfinder: Pathfinder,
    evaluator: T,
    pub settings: BotSettings
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

impl<T: Evaluator> Bot<T> {
    pub fn new(evaluator: T, settings: BotSettings) -> Self {
        Bot {
            queue: Vec::new(),
            root: None,
            pathfinder: Pathfinder::new(),
            evaluator,
            settings
        }
    }
    pub fn update_queue(&mut self, mino: Tetrimino) {
        self.queue.push(mino);
    }
    pub fn reset(&mut self, board: Board, queue: Vec<Tetrimino>) {
        self.root = Some(Node {
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
            score: 0.0,
            sims: 1,
            uses_hold: false,
            finished: false,
            depth: 0
        });
        self.queue = queue;
    }
    pub fn think(&mut self) -> bool {
        let mut root = self.root.take().unwrap();
        self.update_child(&mut root);
        let finished = root.finished;
        self.root.replace(root);
        finished
    }
    fn update_child(&mut self, node: &mut Node) -> (f64, u32) {
        let mut child = None;
        let mut score = std::f64::NEG_INFINITY;
        for c in node.children.iter_mut() {
            if !c.finished {
                use std::f64::consts::SQRT_2;
                let child_score = c.score / (c.sims as f64) + 1.0 * SQRT_2
                    * ((node.sims as f64).ln() / (c.sims as f64)).sqrt();
                if child_score > score {
                    child = Some(c);
                    score = child_score;
                }
            }
        }
        if let Some(child) = child {
            let (score, sims) = self.update_child(child);
            node.score += score;
            node.sims += sims;
            (score, sims)
        } else if node.children.is_empty() {
            self.expand_node(node)
        } else {
            node.finished = true;
            (0.0, 0)
        }
    }
    fn expand_node(&mut self, node: &mut Node) -> (f64, u32) {
        fn create_child<T: Evaluator>(bot: &Bot<T>, mv: PieceState, uses_hold: bool, child_depth: u32, parent: &mut Node) -> f64 {
            let mut board = parent.board.clone();
            let mut child_depth = child_depth;
            if uses_hold {
                let used = board.hold.is_none();
                board.hold_piece(bot.queue[child_depth as usize]);
                if used {
                    child_depth += 1;
                }
            }
            board.state = mv;
            let lock = board.hard_drop(bot.queue[child_depth as usize]);
            child_depth += 1;
            let mut child = Node {
                board,
                mv,
                lock,
                children: Vec::new(),
                depth: child_depth,
                score: 0.0,
                sims: 1,
                uses_hold,
                finished: child_depth as usize >= bot.queue.len()
            };
            let eval = bot.evaluator.evaluate(&child, parent);
            child.score = eval.0 + eval.1;
            parent.children.push(child);
            eval.0
        }
        let child_depth = node.depth;
        let mut score = 0.0;
        for mv in self.pathfinder.get_moves(&mut node.board) {
            score += create_child(&self, mv, false, child_depth, node);
        }
        if self.settings.use_hold {
            let mut hold_board = node.board.clone();
            let used = if hold_board.hold.is_none() { 1 } else { 0 };
            hold_board.hold_piece(self.queue[child_depth as usize]);
            if ((child_depth + used) as usize) < self.queue.len() {
                for mv in self.pathfinder.get_moves(&mut hold_board) {
                    score += create_child(&self, mv, true, child_depth, node);
                }
            }
        }
        let sims = node.children.len() as u32;
        if node.children.is_empty() {
            node.finished = true;
        } else {
            node.score += score;
            node.sims += sims;
        }
        (score, sims)
    }
    pub fn next_move(&mut self) -> Option<&Node> {
        self.queue.remove(0);
        let root = self.root.take().unwrap();
        //println!("{}", serde_json::to_string(&root).unwrap());
        self.root = root.children.into_iter().max_by_key(|c| c.sims);
        if self.root.is_some() {
            Self::update_tree(self.root.as_mut().unwrap());
        }
        // let board = &self.root.as_ref().unwrap().board;
        // for y in 20..40 {
        //     for x in 0..10 {
        //         print!("{}", if board.get_cell(x, y) == CellType::Empty {
        //             ".."
        //         } else {
        //             "[]"
        //         });
        //     }
        //     println!("{}", "");
        // }
        // println!("Hold: {:?}", board.hold);
        self.root.as_ref()
    }
    fn update_tree(node: &mut Node){
        node.finished = false;
        node.depth -= 1;
        for c in node.children.iter_mut() {
            Self::update_tree(c);
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
    pub score: f64,
    pub sims: u32,
    pub finished: bool,
    pub depth: u32
}