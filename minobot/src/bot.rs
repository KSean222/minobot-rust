use crate::pathfinder::Pathfinder;
use crate::evaluator::{ Evaluator, StandardEvaluator };
use minotetris::*;

pub struct Bot<T=StandardEvaluator> {
    queue: Vec<Tetrimino>,
    pub root: Option<Node>,
    pathfinder: Pathfinder,
    evaluator: T
}

impl<T: Evaluator> Bot<T> {
    pub fn new(evaluator: T) -> Self {
        Bot {
            queue: Vec::new(),
            root: None,
            pathfinder: Pathfinder::new(),
            evaluator
        }
    }
    pub fn update(&mut self, mino: Tetrimino) {
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
            score: 0.0,
            sims: 1,
            finished: false,
            depth: 0
        });
        self.queue = queue;
    }
    pub fn think(&mut self) -> bool {
        Self::update_child(self.root.as_mut().unwrap(), &mut self.pathfinder, &mut self.queue, &self.evaluator);
        self.root.as_ref().unwrap().finished
    }
    fn update_child(parent: &mut Node, pathfinder: &mut Pathfinder, queue: &Vec<Tetrimino>, evaluator: &T) -> (f64, u32) {
        let mut child = None;
        let mut score = std::f64::NEG_INFINITY;
        for c in parent.children.iter_mut() {
            if c.finished {
                continue;
            }
            use std::f64::consts::SQRT_2;
            let s = (if c.sims == 0 { 0.0 } else { c.score / (c.sims as f64) }) +
                1.0 * SQRT_2 * ((parent.sims as f64).ln() / (c.sims as f64)).sqrt();
            if score > s {
                child = Some(c);
                score = s;
            }
        }
        if let Some(child) = child {
            let eval = Self::update_child(child, pathfinder, queue, evaluator);
            parent.score += eval.0;
            parent.sims += eval.1;
            eval
        } else if parent.children.is_empty() {
            let mut score = 0.0;
            let mut sims = 0u32;
            let moves = pathfinder.get_moves(&mut parent.board);
            for mv in moves {
                let mut board = parent.board.clone();
                board.state = mv;
                board.hard_drop(queue[parent.depth as usize]);
                let mut child = Node {
                    board,
                    mv,
                    children: Vec::new(),
                    depth: parent.depth + 1,
                    score: 0.0,
                    sims: 0,
                    finished: (parent.depth + 1) as usize >= queue.len()
                };
                let (accumulated, transient) = evaluator.evaluate(&child, parent);
                child.score = accumulated + transient;
                child.sims = 1;
                score += accumulated;
                sims += 1;
                parent.children.push(child);
            }
            if sims == 0 {
                parent.finished = true;
            } else {
                parent.score += score;
                parent.sims += sims;
            }
            (score, sims)
        } else {
            parent.finished = true;
            (0.0, 0)
        }
    }
    pub fn next_move(&mut self) -> Option<&Node> {
        self.queue.remove(0);
        let root = self.root.take().unwrap();
        self.root = root.children.into_iter().max_by(|x, y| {
            x.score.partial_cmp(&y.score).unwrap()
        });
        if self.root.is_some() {
            Self::update_tree(self.root.as_mut().unwrap());
        }
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

pub struct Node {
    pub board: Board,
    pub mv: PieceState,

    pub children: Vec<Node>,
    pub score: f64,
    pub sims: u32,
    pub finished: bool,
    pub depth: u32
}