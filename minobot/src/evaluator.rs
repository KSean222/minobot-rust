use crate::bot::Node;
use std::collections::HashSet;
use minotetris::*;

pub trait Evaluator {
    fn evaluate(&self, node: &Node, parent: &Node) -> (f64, f64);
}

pub struct StandardEvaluator {
    holes: f64,
    holes_sq: f64,
    hole_depths: f64,
    hole_depths_sq: f64,
    move_height: f64,
    move_height_sq: f64,
    piece_fit: f64,
    piece_fit_sq: f64,
    max_height: f64,
    max_height_sq: f64,
    wells: f64,
    wells_sq: f64,
    spikes: f64,
    spikes_sq: f64
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
            piece_fit: 0.0,
            piece_fit_sq: 0.0,
            max_height: -1.0,
            max_height_sq: 0.0,
            wells: 0.0,
            wells_sq: -1.0,
            spikes: 0.0,
            spikes_sq: -1.0
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
                } else if heights[x as usize] == 0 {
                    heights[x as usize] = height;
                    max_height = max_height.max(height);
                    if node.board.get_cell(x - 1, y) == CellType::Empty && node.board.get_cell(x + 1, y) == CellType::Empty {
                        spike_streak += 1;
                        if spike_streak == 2 {
                            spikes += 1;
                        }
                    }
                }
            }
        }
        let mut to_check = HashSet::new();
        for &(x, y) in &parent.board.current.cells(node.mv.r) {
            to_check.insert((node.mv.x + x, node.mv.y + y));
        }
        let mut filled_edge_tiles = 0;
        let total_edge_tiles = to_check.len();
        for (x, y) in to_check {
            if parent.board.get_cell(x, y) != CellType::Empty {
                filled_edge_tiles += 1;
            }
        }
        if node.mv.y <= 35 {
            let move_height = 39 - node.mv.y;
            accumulated += (move_height as f64) * self.move_height;
            accumulated += ((move_height * move_height) as f64) * self.move_height_sq;
        }
        accumulated += (holes as f64) * self.holes;
        accumulated += ((holes * holes) as f64) * self.holes_sq;
        accumulated += (hole_depths as f64) * self.hole_depths;
        accumulated += (hole_depths_sq as f64) * self.hole_depths_sq;
        let fit = (filled_edge_tiles as f64) / (total_edge_tiles as f64);
        accumulated += fit * self.piece_fit;
        accumulated += fit * fit * self.piece_fit_sq;
        accumulated += (max_height as f64) * self.max_height;
        accumulated += ((max_height * max_height) as f64) * self.max_height_sq;
        accumulated += (wells as f64) * self.wells;
        accumulated += ((wells * wells) as f64) * self.wells_sq;
        accumulated += (spikes as f64) * self.spikes;
        accumulated += ((spikes * spikes) as f64) * self.spikes_sq;
        (accumulated, transient)
    }
}
