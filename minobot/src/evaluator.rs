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
    pub well_depth: i32,
    pub max_well_depth: i32,
    pub line_clear: [i32; 5],
    pub mini_clear: [i32; 3],
    pub tspin_clear: [i32; 4],
    pub perfect_clear: i32,
    pub combo_garbage: i32,
    pub wasted_t: i32,
    pub tslot: i32
}


const COMBO_TABLE: [i32; 13] = [0, 0, 0, 1, 1, 2, 2, 3, 3, 4, 4, 4, 5];
impl Default for  StandardEvaluator {
    fn default() -> Self {
        StandardEvaluator {
            holes: -203,
            holes_sq: -8,
            hole_depths: -18,
            hole_depths_sq: -1,
            move_height: -18,
            move_height_sq: -4,
            move_dist: -5,
            max_height: -8,
            max_height_sq: 0,
            bumpiness: -15,
            bumpiness_sq: -9,
            row_transitions: -20,
            row_transitions_sq: 0,
            well_depth: 55,
            max_well_depth: 10,
            line_clear: [
                7,
                -363,
                -293,
                -280,
                554
            ],
            mini_clear: [
                1,
                -194,
                101
            ],
            tspin_clear: [
                -6,
                108,
                629,
                1244
            ],
            perfect_clear: 5000,
            combo_garbage: 305,
            wasted_t: -268,
            tslot: 301
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
            let column_height = node.board.column_heights()[x as usize];
            for y in 0..column_height {
                if !node.board.occupied(x, y) {
                    let depth = column_height - y - 1;
                    holes += 1;
                    hole_depths += depth;
                    hole_depths_sq += depth * depth;
                    if  !node.board.occupied(x - 1, y) &&
                        !node.board.occupied(x + 1, y) &&
                        !node.board.occupied(x, y - 1) &&
                        !node.board.occupied(x, y + 1) &&
                        node.board.occupied(x - 1, y - 1) &&
                        node.board.occupied(x + 1, y - 1) &&
                        (node.board.occupied(x - 1, y + 1) ||
                        node.board.occupied(x + 1, y + 1)) {
                        tslots += 1;
                    }
                }
            }
        }
        value += holes * self.holes;
        value += holes * holes * self.holes_sq;
        value += hole_depths * self.hole_depths;
        value += hole_depths_sq * self.hole_depths_sq;

        let max_height = node.board.column_heights().iter().copied().max().unwrap();
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
        for y in 0..20 {
            for x in 0..11 {
                if node.board.occupied(x - 1, y) != node.board.occupied(x, y) {
                    row_transitions += 1;
                }
            }
        }
        value += row_transitions * self.row_transitions;
        value += row_transitions * row_transitions * self.row_transitions_sq;

        let (well_column, min_height) = node.board
            .column_heights()
            .iter()
            .copied()
            .enumerate()
            .min_by_key(|&(_, h)| h)
            .unwrap();
        let mut well_row = u16::default();
        for x in 0..10 {
            if x != well_column {
                well_row.set(x, CellType::Garbage);
            }
        }
        let well_depth = node.board
            .rows()
            .iter()
            .skip(min_height as usize)
            .take_while(|&&r| r == well_row)
            .count()
            as i32;
        value += well_depth.min(self.max_well_depth) * self.well_depth;
        
        value += node.mv.y * self.move_height;
        value += node.mv.y * node.mv.y * self.move_height_sq;

        if node.mv.kind == PieceType::T && (node.mv.tspin == TspinType::None || node.lock.lines_cleared == 0) {
            reward += self.wasted_t;
        }
        reward += match node.mv.tspin {
            TspinType::None => &self.line_clear[..],
            TspinType::Mini => &self.mini_clear[..],
            TspinType::Full => &self.tspin_clear[..],
        }[node.lock.lines_cleared as usize];
        reward += COMBO_TABLE[(node.lock.combo as usize).min(COMBO_TABLE.len() - 1)] * self.combo_garbage;
        reward += node.move_dist * self.move_dist;
        if node.board.column_heights().iter().all(|&h| h == 0) {
            reward += self.perfect_clear;
        }

        (value, reward)
    }
}
