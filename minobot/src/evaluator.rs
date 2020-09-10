use serde::{ Serialize, Deserialize };

use crate::bot::Node;
use minotetris::*;

pub trait Evaluator: Send {
    fn evaluate(&self, node: &Node, queue: &[PieceType]) -> (i32, i32);
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StandardEvaluator {
    pub holes: i32,
    pub holes_sq: i32,
    pub hole_depths: i32,
    pub hole_depths_sq: i32,
    pub move_height: i32,
    pub move_height_sq: i32,
    pub move_dist: i32,
    pub max_height: i32,
    pub max_height_sq: i32,
    pub bumpiness: i32,
    pub bumpiness_sq: i32,
    pub row_transitions: i32,
    pub row_transitions_sq: i32,
    pub line_clear: [i32; 5],
    pub mini_clear: [i32; 3],
    pub tspin_clear: [i32; 4],
    pub wasted_t: i32,
    pub tslot: i32
}

impl Default for  StandardEvaluator {
    fn default() -> Self {
        StandardEvaluator {
            holes: -200,
            holes_sq: -7,
            hole_depths: -25,
            hole_depths_sq: -2,
            move_height: -20,
            move_height_sq: 0,
            move_dist: -1,
            max_height: -10,
            max_height_sq: 0,
            bumpiness: -20,
            bumpiness_sq: -5,
            row_transitions: -15,
            row_transitions_sq: 0,
            line_clear: [
                0,
                -300,
                -290,
                -280,
                500
            ],
            mini_clear: [
                0,
                -200,
                100
            ],
            tspin_clear: [
                0,
                100,
                500,
                1000,
            ],
            wasted_t: -250,
            tslot: 300
        }
    }
}

impl Evaluator for StandardEvaluator {
    fn evaluate(&self, node: &Node, queue: &[PieceType]) -> (i32, i32) {
        if node.lock.block_out {
            return (std::i32::MIN, std::i32::MIN);
        }
        
        let mut value = 0;
        let mut reward = 0;

        let mut holes = 0;
        let mut hole_depths = 0;
        let mut hole_depths_sq = 0;
        let mut tslots = 0;
        for x in 0..10 {
            let column_height = node.board.column_heights()[x as usize] as i32;
            for y in (40 - column_height as i32)..40 {
                let height = 40 - y;
                if !node.board.occupied(x, y) {
                    let depth = column_height - height;
                    holes += 1;
                    hole_depths += depth;
                    hole_depths_sq += depth * depth;
                    if  !node.board.occupied(x - 1, y) &&
                        !node.board.occupied(x + 1, y) &&
                        !node.board.occupied(x, y + 1) &&
                        !node.board.occupied(x, y - 1) &&
                        node.board.occupied(x - 1, y + 1) &&
                        node.board.occupied(x + 1, y + 1) &&
                        (node.board.occupied(x - 1, y - 1) ||
                        node.board.occupied(x + 1, y - 1)) {
                        tslots += 1;
                    }
                }
            }
        }
        let max_height = node.board.column_heights().iter().copied().max().unwrap();
        value += holes * self.holes;
        value += holes * holes * self.holes_sq;
        value += hole_depths * self.hole_depths;
        value += hole_depths_sq * self.hole_depths_sq;
        value += max_height * self.max_height;
        value += max_height * max_height * self.max_height_sq;

        let mut bumpiness = 0;
        let mut bumpiness_sq = 0;
        let bumpiness_iter = node.board
            .column_heights()
            .iter()
            .zip(node.board.column_heights().iter().skip(1));
        for (&h1, &h2) in bumpiness_iter {
            let diff = (h1 - h2).abs();
            bumpiness += diff;
            bumpiness_sq += diff * diff;
        }
        value += bumpiness * self.bumpiness;
        value += bumpiness_sq * self.bumpiness_sq;

        let mut t_pieces  = queue
            .iter()
            .skip(node.depth as usize)
            .filter(|&&t| t == PieceType::T)
            .count() as i32;
        if node.board.hold == Some(PieceType::T) {
            t_pieces += 1;
        }
        value += t_pieces.min(tslots).max(1) * self.tslot;

        let mut row_transitions = 0;
        for y in 20..40 {
            for x in 0..11 {
                if node.board.occupied(x - 1, y) != node.board.occupied(x, y) {
                    row_transitions += 1;
                }
            }
        }
        value += row_transitions * self.row_transitions;
        value += row_transitions * row_transitions * self.row_transitions_sq;

        let move_height = 39 - node.mv.y;
        value += move_height * self.move_height;
        value += move_height * move_height * self.move_height_sq;

        if node.mv.kind == PieceType::T && (node.mv.tspin == TspinType::None || node.lock.lines_cleared == 0) {
            reward += self.wasted_t;
        }
        reward += match node.mv.tspin {
            TspinType::None => &self.line_clear[..],
            TspinType::Mini => &self.mini_clear[..],
            TspinType::Full => &self.tspin_clear[..],
        }[node.lock.lines_cleared as usize];
        reward += node.move_dist * self.move_dist;

        (value, reward)
    }
}
