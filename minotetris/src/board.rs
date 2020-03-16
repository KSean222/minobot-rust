use crate::*;
use std::hash::Hash;

 #[derive(Copy, Clone, Debug)]
pub struct HardDropResult {
    pub block_out: bool,
    pub lines_cleared: i32
}

#[derive(Copy, Clone, Hash, Debug)]
pub struct PieceState {
    pub x: i32,
    pub y: i32,
    pub r: u8
}

impl PartialEq for PieceState {
    fn eq(&self, other: &Self) -> bool {
        self.x == other.x &&
        self.y == other.y &&
        self.r == other.r
    }
}

impl Eq for PieceState { }

#[derive(PartialEq, Copy, Clone)]
pub enum TspinType {
    None,
    Mini,
    Full
}

pub trait Row: Copy {
    fn set(&mut self, x: i32, cell: CellType);
    fn get(&self, x: i32) -> CellType;
    fn filled(&self) -> bool;
    const EMPTY: Self;
}

impl Row for u16 {
    fn set(&mut self, x: i32, cell: CellType) {
        if cell == CellType::Empty {
            *self &= !(1 << x);
        } else {
            *self |= 1 << x;
        }
    }
    fn get(&self, x: i32) -> CellType {
        if *self & (1 << x) == 0 {
            CellType::Empty
        } else {
            CellType::Garbage
        }
    }
    fn filled(&self) -> bool {
        *self == 0b0000001111111111
    }
    const EMPTY: u16 = 0;
}

#[derive(Copy, Clone)]
pub struct ColoredRow {
    row: [CellType; 10]
}

impl ColoredRow {
    pub fn compress(&self) -> u16 {
        let mut row = u16::EMPTY;
        for x in 0..10 {
            row.set(x, self.row[x as usize]);
        }
        row
    }
}

impl Row for ColoredRow {
    fn set(&mut self, x: i32, cell: CellType) {
        self.row[x as usize] = cell;
    }
    fn get(&self, x: i32) -> CellType {
        self.row[x as usize]
    }
    fn filled(&self) -> bool {
        self.row
            .iter()
            .all(|c| *c != CellType::Empty)
    }
    const EMPTY: ColoredRow = ColoredRow {
        row: [CellType::Empty; 10]
    };
}

#[derive(Clone)]
pub struct Board<T=u16> {
    pub rows: [T; 40],
    pub current: Tetrimino,
    pub hold: Option<Tetrimino>,
    pub state: PieceState,
    pub held: bool
}

impl std::fmt::Debug for Board {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", "<Board>")
    }
}

impl<T: Row> Board<T> {
    pub fn new(start_piece: Tetrimino) -> Self {
        let mut board = Board {
            rows: [T::EMPTY; 40],
            current: start_piece,
            hold: None,
            state: PieceState {
                x: 0,
                y: 0,
                r: 0
            },
            held: false
        };
        board.set_piece(start_piece);
        board
    }
    pub fn is_out_of_bounds(x: i32, y: i32) -> bool {
        x < 0 || x >= 10 || y < 0 || y >= 40
    }
    pub fn hold_piece(&mut self, next: Tetrimino) -> bool {
        if self.held {
            false
        } else {
            self.held = true;
            let temp = self.current;
            self.set_piece(match self.hold {
                Some(hold) => hold,
                None => next
            });
            self.hold.replace(temp);
            true
        }
    }
    pub fn get_cell(&self, x: i32, y: i32) -> CellType {
        if Self::is_out_of_bounds(x, y) {
            CellType::Solid
        } else {
            self.rows[y as usize].get(x)
        }
    }
    pub fn set_cell(&mut self, x: i32, y: i32, cell: CellType) {
        if !Self::is_out_of_bounds(x, y) {
            self.rows[y as usize].set(x, cell);
        }
    }
    pub fn hard_drop(&mut self, next: Tetrimino) -> HardDropResult {
        while self.try_move(self.state.x, self.state.y + 1, self.state.r) { }
        for (cell_x, cell_y) in &self.current.cells(self.state.r) {
            self.set_cell(self.state.x + cell_x, self.state.y + cell_y, self.current.cell());
        }
        let mut new_board = [T::EMPTY; 40];
        let mut lines_cleared: i32 = 0;
        for y in (0..40).rev() {
            if self.rows[y].filled() {
                lines_cleared += 1;
            } else {
                let new_y = y + (lines_cleared as usize);
                new_board[new_y] = self.rows[y];
            }
        }
        self.rows = new_board;
        self.set_piece(next);
        self.held = false;
        HardDropResult {
            block_out: !self.piece_fits(self.state.x, self.state.y, self.state.r),
            lines_cleared: lines_cleared
        }
    }
    pub fn set_piece(&mut self, piece: Tetrimino){
        self.current = piece;
        self.state.x = 4;
        self.state.y = 19;
        self.state.r = 0;
        self.try_move(self.state.x, self.state.y + 1, self.state.r);
    }
    pub fn piece_fits(&self, x: i32, y: i32, rot: u8) -> bool {
        for (cell_x, cell_y) in &self.current.cells(rot) {
            if self.get_cell(x + cell_x, y + cell_y) != CellType::Empty {
                return false;
            }
        }
        true
    }
    pub fn move_left(&mut self) -> bool {
        self.try_move(self.state.x - 1, self.state.y, self.state.r)
    }
    pub fn move_right(&mut self) -> bool {
        self.try_move(self.state.x + 1, self.state.y, self.state.r)
    }
    pub fn soft_drop(&mut self) -> bool {
        self.try_move(self.state.x, self.state.y + 1, self.state.r)
    }
    pub fn turn_left(&mut self) -> bool {
        self.rotate(if self.state.r > 0 { self.state.r - 1 } else { 3 })
    }
    pub fn turn_right(&mut self) -> bool {
        self.rotate(if self.state.r < 3 { self.state.r + 1 } else { 0 })
    }
    fn rotate(&mut self, r: u8) -> bool {
        let current = self.current;
        let from_table = current.offset_table(self.state.r);
        let to_table = current.offset_table(r);
        for i in 0..from_table.len() {
            let (from_x, from_y) = from_table[i];
            let (to_x, to_y) = to_table[i];
            let x = self.state.x + from_x - to_x;
            let y = self.state.y - (from_y - to_y);
            if self.try_move(x, y, r) {
                return true;
            }
        }
        false
    }
    fn try_move(&mut self, x: i32, y: i32, r: u8) -> bool {
        if self.piece_fits(x, y, r) {
            self.state = PieceState {
                x,
                y,
                r
            };
            true
        } else {
            false
        }
    }
}

impl Board<ColoredRow> {
    pub fn compress(&self) -> Board {
        let mut rows = [0; 40];
        for y in 0..40 {
            rows[y] = self.rows[y].compress();
        }
        Board {
            rows,
            current: self.current,
            hold: self.hold,
            state: self.state,
            held: self.held
        }
    }
}