use crate::*;
use serde::{ Serialize, Deserialize };
use serde::de::DeserializeOwned;

#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
pub struct HardDropResult {
    pub mino: Tetrimino,
    pub block_out: bool,
    pub lines_cleared: i32,
    pub tspin: TspinType
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct PieceState {
    pub x: i32,
    pub y: i32,
    pub r: u8,
    pub tspin: TspinType
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub enum TspinType {
    None,
    Mini,
    Full
}

pub trait Row: Copy + Default + Serialize + DeserializeOwned {
    fn set(&mut self, x: i32, cell: CellType);
    fn get(&self, x: i32) -> CellType;
    fn filled(&self) -> bool;
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
}

#[derive(Copy, Clone, Serialize, Deserialize)]
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
    fn set(&mut self, x: i32, cell: CellType) {
        self.row[x as usize] = cell;
    }
    fn get(&self, x: i32) -> CellType {
        self.row[x as usize]
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

big_array! { BigArray; }

#[derive(Clone, Serialize, Deserialize)]
pub struct Board<T=u16> where T: Row {
    #[serde(with = "BigArray")]
    pub rows: [T; 40],
    pub current: Tetrimino,
    pub hold: Option<Tetrimino>,
    pub state: PieceState,
    pub held: bool
}

impl std::fmt::Debug for Board {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Board {{  }}")
    }
}

impl<T: Row> Board<T> {
    pub fn new(start_piece: Tetrimino) -> Self {
        let mut board = Board {
            rows: [T::default(); 40],
            current: start_piece,
            hold: None,
            state: PieceState {
                x: 0,
                y: 0,
                r: 0,
                tspin: TspinType::None
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
            let current = self.current;
            self.set_piece(if let Some(hold) = self.hold {
                hold
            } else {
                next
            });
            self.hold.replace(current);
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
        for &(cell_x, cell_y) in &self.current.cells(self.state.r) {
            self.set_cell(self.state.x + cell_x, self.state.y + cell_y, self.current.cell());
        }
        let mut new_board = [T::default(); 40];
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
        let mino = self.current;
        let tspin = self.state.tspin;
        self.set_piece(next);
        self.held = false;
        HardDropResult {
            mino,
            block_out: !self.piece_fits(self.state.x, self.state.y, self.state.r),
            lines_cleared,
            tspin
        }
    }
    pub fn set_piece(&mut self, piece: Tetrimino){
        self.current = piece;
        self.state = PieceState {
            x: 4,
            y: 19,
            r: 0,
            tspin: TspinType::None
        };
        self.try_move(self.state.x, self.state.y + 1, self.state.r);
    }
    pub fn piece_fits(&self, x: i32, y: i32, r: u8) -> bool {
        for &(cell_x, cell_y) in &self.current.cells(r) {
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
        let from_table = self.current.offset_table(self.state.r);
        let to_table = self.current.offset_table(r);
        for (i, (from, to)) in from_table.iter().zip(to_table.iter()).enumerate() {
            let x = self.state.x + from.0 - to.0;
            let y = self.state.y - (from.1 - to.1);
            if self.try_move(x, y, r) {
                if self.current == Tetrimino::T {
                    const CORNER_CELLS: [(i32, i32); 4] = [
                        (-1, -1),
                        (-1, 1),
                        (1, 1),
                        (1, -1)
                    ];
                    let mut corners = 0;
                    for &(corner_x, corner_y) in &CORNER_CELLS {
                        if self.get_cell(x + corner_x, y + corner_y) != CellType::Empty {
                            corners += 1;
                        }
                    }
                    if corners > 2 {
                        let front_corner_cells = match r {
                            0 => [(-1, -1), (1, -1)],
                            1 => [(1, -1), (1, 1)],
                            2 => [(-1, 1), (1, 1)],
                            3 => [(-1, 1), (-1, -1)],
                            _ => unreachable!()
                        };
                        let mut front_corners = 0;
                        for &(corner_x, corner_y) in &front_corner_cells {
                            if self.get_cell(x + corner_x, y + corner_y) != CellType::Empty {
                                front_corners += 1;
                            }
                        }
                        self.state.tspin = if front_corners >= 2 || i == 4 {
                            TspinType::Full
                        } else {
                            TspinType::Mini
                        };
                    }
                }
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
                r,
                tspin: TspinType::None
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
        for (y, row) in self.rows.iter().enumerate() {
            rows[y] = row.compress();
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