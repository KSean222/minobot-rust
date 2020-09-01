use serde::{ Serialize, Deserialize };

use crate::bot::Node;
use minotetris::*;

pub trait Evaluator: Send {
    fn evaluate(&self, node: &Node, queue: &[PieceType]) -> (f64, f64);
}

#[derive(Serialize, Deserialize)]
pub struct StandardEvaluator {
    holes: f64,
    holes_sq: f64,
    hole_depths: f64,
    hole_depths_sq: f64,
    move_height: f64,
    move_height_sq: f64,
    max_height: f64,
    max_height_sq: f64,
    bumpiness: f64,
    bumpiness_sq: f64,
    row_transitions: f64,
    row_transitions_sq: f64,
    line_clear: [f64; 5],
    tspin_clear: [f64; 4],
    wasted_t: f64,
    tslot: f64
}

impl Default for  StandardEvaluator {
    fn default() -> Self {
        StandardEvaluator {
            holes: -200.0,
            holes_sq: -7.0,
            hole_depths: -25.0,
            hole_depths_sq: -2.0,
            move_height: -20.0,
            move_height_sq: 0.0,
            max_height: -10.0,
            max_height_sq: 0.0,
            bumpiness: -20.0,
            bumpiness_sq: -5.0,
            row_transitions: -15.0,
            row_transitions_sq: 0.0,
            line_clear: [
                0.0,
                -200.0,
                -175.0,
                -150.0,
                500.0
            ],
            tspin_clear: [
                0.0,
                100.0,
                750.0,
                1000.0,
            ],
            wasted_t: -500.0,
            tslot: 300.0
        }
    }
}

impl Evaluator for StandardEvaluator {
    fn evaluate(&self, node: &Node, queue: &[PieceType]) -> (f64, f64) {
        let mut value = 0.0;
        let mut reward = 0.0;

        // if node.lock.block_out {
        //     return (std::f64::NEG_INFINITY, std::f64::NEG_INFINITY);
        // }

        let mut holes = 0;
        let mut hole_depths = 0;
        let mut hole_depths_sq = 0;
        let mut tslots = 0;
        for x in 0..10 {
            let column_height = node.board.column_heights[x as usize] as i32;
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
        let max_height = node.board.column_heights.iter().copied().max().unwrap();
        value += holes as f64 * self.holes;
        value += (holes * holes) as f64 * self.holes_sq;
        value += hole_depths as f64 * self.hole_depths;
        value += hole_depths_sq as f64 * self.hole_depths_sq;
        value += max_height as f64 * self.max_height;
        value += (max_height * max_height) as f64 * self.max_height_sq;

        let mut bumpiness = 0.0;
        let mut bumpiness_sq = 0.0;
        for (&h1, &h2) in node.board.column_heights.iter().zip(node.board.column_heights.iter().skip(1)) {
            let diff = (h1 - h2).abs() as f64;
            bumpiness += diff;
            bumpiness_sq += diff * diff;
        }
        value += bumpiness * self.bumpiness;
        value += bumpiness_sq * self.bumpiness_sq;

        let mut t_pieces  = queue
            .iter()
            .skip(node.depth as usize)
            .filter(|&&t| t == PieceType::T)
            .count() as u32;
        if node.board.hold == Some(PieceType::T) {
            t_pieces += 1;
        }
        value += t_pieces.min(tslots).max(1) as f64 * self.tslot;

        let mut row_transitions = 0;
        for y in 20..40 {
            for x in 0..11 {
                if node.board.occupied(x - 1, y) != node.board.occupied(x, y) {
                    row_transitions += 1;
                }
            }
        }
        value += row_transitions as f64 * self.row_transitions;
        value += (row_transitions * row_transitions) as f64 * self.row_transitions_sq;

        let move_height = 39 - node.mv.y;
        value += move_height as f64 * self.move_height;
        value += (move_height * move_height) as f64 * self.move_height_sq;

        if node.mv.kind == PieceType::T && (node.mv.tspin == TspinType::None || node.lock.lines_cleared == 0) {
            reward += self.wasted_t;
        }
        reward += match node.mv.tspin {
            TspinType::None => &self.line_clear[..],
            TspinType::Mini | TspinType::Full => {
                // for board in &[&parent.board, &node.board] {
                //     for y in 20..40 {
                //         for x in 0..10 {
                //             print!("{}", if board.get_cell(x, y) == CellType::Empty {
                //                 ".."
                //             } else {
                //                 "[]"
                //             });
                //         }
                //         println!();
                //     }
                //     println!("Hold: {:?}", board.hold);
                // }
                // println!("Detected as {:?} {}", node.lock.tspin, node.lock.lines_cleared);
                // panic!();
                &self.tspin_clear[..]
            }
        }[node.lock.lines_cleared as usize];

        (value, reward)
    }
}
