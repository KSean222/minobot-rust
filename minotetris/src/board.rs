use crate::*;
use arrayvec::ArrayVec;

#[derive(Copy, Clone, Debug)]
pub struct LockResult {
    pub lines_cleared: i32,
    pub block_out: bool,
    pub combo: u32,
    pub b2b_bonus: bool
}

pub trait Row: Copy + Default {
    fn set(&mut self, x: usize, cell: CellType);
    fn get(&self, x: usize) -> bool;
    fn cell_type(&self, x: usize) -> CellType;
    fn filled(&self) -> bool;
}

impl Row for u16 {
    fn set(&mut self, x: usize, cell: CellType) {
        if cell == CellType::Empty {
            *self &= !(1 << x);
        } else {
            *self |= 1 << x;
        }
    }
    fn get(&self, x: usize) -> bool {
        *self & (1 << x) != 0
    }
    fn cell_type(&self, x: usize) -> CellType {
        if self.get(x) {
            CellType::Garbage
        } else {
            CellType::Empty
        }
    }
    fn filled(&self) -> bool {
        *self == 0b0000001111111111
    }
}

#[derive(Copy, Clone)]
pub struct ColoredRow {
    row: [CellType; 10]
}

impl ColoredRow {
    pub fn compress(&self) -> u16 {
        let mut row = u16::default();
        for x in 0..10 {
            row.set(x, self.row[x as usize]);
        }
        row
    }
}

impl Row for ColoredRow {
    fn set(&mut self, x: usize, cell: CellType) {
        self.row[x] = cell;
    }
    fn get(&self, x: usize) -> bool {
        self.row[x] != CellType::Empty
    }
    fn cell_type(&self, x: usize) -> CellType {
        self.row[x]
    }
    fn filled(&self) -> bool {
        self.row
            .iter()
            .all(|&c| c != CellType::Empty)
    }
}

impl Default for ColoredRow {
    fn default() -> Self {
        ColoredRow {
            row: [CellType::Empty; 10]
        }
    }
}

#[derive(Debug, Clone)]
pub struct Board<R=u16> {
    rows: ArrayVec<[R; 40]>,
    column_heights: [i32; 10],
    pub hold: Option<PieceType>,
    pub combo: u32,
    pub b2b: bool
}

impl<R: Row> Board<R> {
    pub fn new() -> Self {
        Self {
            rows: [R::default(); 40].into(),
            column_heights: [0; 10],
            hold: None,
            combo: 0,
            b2b: false
        }
    }
    pub fn occupied(&self, x: i32, y: i32) -> bool {
        x < 0 || x >= 10 || y < 0 || y >= 40 || self.rows[y as usize].get(x as usize)
    }
    pub fn lock_piece(&mut self, piece: Piece) -> LockResult {
        let mut block_out = true;
        for &(x, y) in &piece.cells() {
            self.rows[y as usize].set(x as usize, piece.kind.cell());
            self.column_heights[x as usize] = self.column_heights[x as usize].max(y + 1);
            if y < 20 {
                block_out = false;
            }
        }
        
        self.rows.retain(|r| !r.filled());
        let lines_cleared = 40 - self.rows.len() as i32;
        self.rows.extend((0..lines_cleared).map(|_| R::default()));

        for x in 0..10 {
            self.column_heights[x] -= lines_cleared;
            while self.column_heights[x] > 0 &&
                !self.occupied(x as i32, self.column_heights[x] as i32 - 1) {
                self.column_heights[x] -= 1;
            }
        }
        
        let mut b2b_bonus = false;
        if lines_cleared > 0 {
            self.combo += 1;
            let b2b = self.b2b;
            self.b2b = lines_cleared == 4 || piece.tspin != TspinType::None;
            b2b_bonus = b2b && self.b2b;
        } else {
            self.combo = 0;
        }

        LockResult {
            lines_cleared,
            block_out,
            combo: self.combo,
            b2b_bonus
        }
    }
    pub fn piece_fits(&self, piece: Piece) -> bool {
        piece
            .cells()
            .iter()
            .all(|&(x, y)| !self.occupied(x, y))
    }
    pub fn column_heights(&self) -> &[i32] {
        &self.column_heights
    }
    pub fn rows(&self) -> &[R] {
        &self.rows
    }
    pub fn add_garbage(&mut self, holes: &[i32]) -> bool {
        for &hole in holes {
            for (x, height) in self.column_heights.iter_mut().enumerate() {
                if !(*height == 0 && x as i32 == hole) {
                    *height += 1;
                }
            }
        }
        let garbage_rows = holes
            .iter()
            .map(|&hole| {
                let mut row = R::default();
                for x in 0..10 {
                    if x != hole {
                        row.set(x as usize, CellType::Garbage);
                    }
                }
                row
            });
        let rows = self.rows.len();
        let block_out = self.rows.iter().skip(rows - holes.len()).any(|r| r.filled());
        self.rows = garbage_rows.rev().chain(self.rows.iter().take(rows - holes.len()).copied()).collect();
        block_out
    }
    pub fn set_field(&mut self, rows: impl Into<ArrayVec<[R; 40]>>) {
        self.rows = rows.into();
        for x in 0..10 {
            for y in (0..41).rev() {
                if self.occupied(x, y - 1) {
                    self.column_heights[x as usize] = y;
                    break;
                }
            }
        }
    }
}

impl Board<ColoredRow> {
    pub fn compress(&self) -> Board {
        Board {
            rows: self.rows.iter().map(|row| row.compress()).collect(),
            column_heights: self.column_heights.clone(),
            hold: self.hold,
            combo: self.combo,
            b2b: self.b2b
        }
    }
}