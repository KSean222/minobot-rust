use crate::*;
use arrayvec::ArrayVec;

#[derive(Copy, Clone, Debug)]
pub struct LockResult {
    pub lines_cleared: i32,
    pub block_out: bool
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
    pub hold: Option<PieceType>
}

impl<R: Row> Board<R> {
    pub fn new() -> Self {
        Self {
            rows: [R::default(); 40].into(),
            column_heights: [0; 10],
            hold: None
        }
    }
    pub fn occupied(&self, x: i32, y: i32) -> bool {
        x < 0 || x >= 10 || y < 0 || y >= 40 || self.rows[y as usize].get(x as usize)
    }
    pub fn lock_piece(&mut self, piece: Piece) -> LockResult {
        let mut block_out = true;
        for &(x, y) in &piece.cells() {
            self.rows[y as usize].set(x as usize, piece.kind.cell());
            self.column_heights[x as usize] = self.column_heights[x as usize].max(40 - y);
            if y >= 20 {
                block_out = false;
            }
        }
        
        let lines_cleared = self.rows.iter().filter(|row| row.filled()).count() as i32;
        let new = (0..lines_cleared)
            .map(|_| R::default())
            .chain(self.rows.iter().copied().filter(|row| !row.filled()))
            .collect();
        self.rows = new;

        for x in 0..10 {
            self.column_heights[x] -= lines_cleared;
            while self.column_heights[x] > 0 &&
                !self.occupied(x as i32, 40 - self.column_heights[x] as i32) {
                self.column_heights[x] -= 1;
            }
        }
        
        LockResult {
            lines_cleared,
            block_out
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
    pub fn set_field(&mut self, rows: impl Into<ArrayVec<[R; 40]>>) {
        self.rows = rows.into();
        for x in 0..10 {
            for y in 0..40 {
                if self.occupied(x, y) {
                    self.column_heights[x as usize] = 40 - y;
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
            hold: self.hold
        }
    }
}