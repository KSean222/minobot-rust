use serde::{ Serialize, Deserialize };

use crate::bot::Node;
use minotetris::*;

pub trait Evaluator: Send {
    fn evaluate(&self, node: &Node, parent: &Node) -> (f64, f64);
}

#[derive(Serialize, Deserialize)]
pub struct StandardEvaluator {
    holes: f64,
    holes_sq: f64,
    hole_depths: f64,
    hole_depths_sq: f64,
    move_height: f64,
    move_height_sq: f64,
    filled_cells_x: f64,
    filled_cells_x_sq: f64,
    filled_cells_down: f64,
    filled_cells_down_sq: f64,
    parity: f64,
    parity_sq: f64,
    max_height: f64,
    max_height_sq: f64,
    wells: f64,
    wells_sq: f64,
    spikes: f64,
    spikes_sq: f64,
    line_clear: [f64; 5]
}

impl Default for  StandardEvaluator {
    fn default() -> Self {
        StandardEvaluator {
            holes: 0.0,
            holes_sq: -1.0,
            hole_depths: 0.0,
            hole_depths_sq: -0.5,
            move_height: 0.0,
            move_height_sq: -1.0,
            filled_cells_x: 0.0,
            filled_cells_x_sq: 10.0,
            filled_cells_down: 0.0,
            filled_cells_down_sq: 10.0,
            parity: 0.0,
            parity_sq: -2.0,
            max_height: -10.0,
            max_height_sq: 0.0,
            wells: 0.0,
            wells_sq: -1.0,
            spikes: 0.0,
            spikes_sq: -1.0,
            line_clear: [
                0.0,
                -50.0,
                -40.0,
                -25.0,
                500.0
            ]
        }
    }
}

impl Evaluator for StandardEvaluator {
    fn evaluate(&self, node: &Node, parent: &Node) -> (f64, f64) {
        let mut accumulated = 0.0;
        let mut transient = 0.0;
        let mut heights = [0; 10];
        let mut holes = 0;
        let mut hole_depths = 0;
        let mut hole_depths_sq = 0;
        let mut max_height = 0;
        let mut wells = 0;
        let mut spikes = 0;
        let mut even_cells = 0i32;
        let mut odd_cells = 0i32;
        for x in 0..10 {
            let mut well_streak = 0;
            let mut spike_streak = 0;
            for y in 20..40 {
                let height = 39 - y;
                if node.board.get_cell(x, y) == CellType::Empty {
                    if height < heights[x as usize] {
                        let depth = heights[x as usize] - height;
                        holes += 1;
                        hole_depths += depth;
                        hole_depths_sq += depth * depth;
                    }
                    if node.board.get_cell(x - 1, y) != CellType::Empty && node.board.get_cell(x + 1, y) != CellType::Empty {
                        well_streak += 1;
                        if well_streak == 2 {
                            wells += 1;
                        }
                    }
                } else {
                    if (x + y) & 1 == 0 {
                        even_cells += 1;
                    } else {
                        odd_cells += 1;
                    }
                    if heights[x as usize] == 0 {
                        heights[x as usize] = height;
                        max_height = max_height.max(height);
                    }
                    if node.board.get_cell(x - 1, y) == CellType::Empty && node.board.get_cell(x + 1, y) == CellType::Empty {
                        spike_streak += 1;
                        if spike_streak == 2 {
                            spikes += 1;
                        }
                    }
                }
            }
        }
        let mut filled_cells_x = 0;
        let mut filled_cells_down = 0;
        for &(x, y) in &parent.board.current.cells(node.mv.r) {
            let cell_x = node.mv.x + x;
            let cell_y = node.mv.y + y;
            if parent.board.get_cell(cell_x + 1, cell_y) != CellType::Empty {
                filled_cells_x += 1;
            }
            if parent.board.get_cell(cell_x - 1, cell_y) != CellType::Empty {
                filled_cells_x += 1;
            }
            if parent.board.get_cell(cell_x, cell_y + 1) != CellType::Empty {
                filled_cells_down += 1;
            }
        }
        let parity_diff = (even_cells - odd_cells).abs();
        accumulated += parity_diff as f64 * self.parity;
        accumulated += (parity_diff * parity_diff) as f64 * self.parity_sq;
        accumulated += filled_cells_x as f64 * self.filled_cells_x;
        accumulated += (filled_cells_x * filled_cells_x) as f64 * self.filled_cells_x_sq;
        accumulated += filled_cells_down as f64 * self.filled_cells_down;
        accumulated += (filled_cells_down * filled_cells_down) as f64 * self.filled_cells_down_sq;
        if node.mv.y <= 35 {
            let move_height = 39 - node.mv.y;
            accumulated += (move_height as f64) * self.move_height;
            accumulated += ((move_height * move_height) as f64) * self.move_height_sq;
        }
        accumulated += self.line_clear[node.lock.lines_cleared as usize];
        accumulated += (holes as f64) * self.holes;
        accumulated += ((holes * holes) as f64) * self.holes_sq;
        accumulated += (hole_depths as f64) * self.hole_depths;
        accumulated += (hole_depths_sq as f64) * self.hole_depths_sq;
        accumulated += (max_height as f64) * self.max_height;
        accumulated += ((max_height * max_height) as f64) * self.max_height_sq;
        accumulated += (wells as f64) * self.wells;
        accumulated += ((wells * wells) as f64) * self.wells_sq;
        accumulated += (spikes as f64) * self.spikes;
        accumulated += ((spikes * spikes) as f64) * self.spikes_sq;
        (accumulated, transient)
    }
}
