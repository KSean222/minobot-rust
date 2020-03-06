mod tetrimino;
use tetrimino::{ CellType, Tetrimino };

pub struct HardDropResult {
    pub block_out: bool,
    pub lines_cleared: i32
}

#[derive(Copy, Clone)]
pub struct PieceState {
    pub x: i32,
    pub y: i32,
    pub r: u8
}

pub struct Board {
    pub grid: [[CellType; 40]; 10],
    pub current: Tetrimino,
    pub hold: Option<Tetrimino>,
    pub state: PieceState,
    pub held: bool
}

impl Board {
    pub fn new(start_piece: Tetrimino) -> Board {
        let mut board = Board {
            grid: [[CellType::EMPTY; 40]; 10],
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
    pub fn clone(source: &Board) -> Board {
        Board {
            grid: source.grid,
            current: source.current,
            hold: source.hold.clone(),
            state: source.state,
            held: source.held
        }
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
            CellType::SOLID
        } else {
            self.grid[x as usize][y as usize]
        }
    }
    pub fn set_cell(&mut self, x: i32, y: i32, cell: CellType) {
        if !Self::is_out_of_bounds(x, y) {
            self.grid[x as usize][y as usize] = cell;
        }
    }
    pub fn hard_drop(&mut self, next: Tetrimino) -> HardDropResult {
        while self.try_move(self.state.x, self.state.y + 1, self.state.r) { }
        for (cell_x, cell_y) in &self.current.cells(self.state.r) {
            self.set_cell(self.state.x + cell_x, self.state.y + cell_y, self.current.cell());
        }
        let mut new_board = [[CellType::EMPTY; 40]; 10];
        let mut lines_cleared: i32 = 0;
        for y in (0..40).rev() {
            let mut row_filled = true;
            for x in 0..10 {
                if self.grid[x][y] == CellType::EMPTY {
                    row_filled = false;
                    break;
                }
            }
            if row_filled {
                lines_cleared += 1;
            } else {
                let new_y = y + (lines_cleared as usize);
                for x in 0..10 {
                    new_board[x][new_y] = self.grid[x][y];
                }
            }
        }
        self.grid = new_board;
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
            if self.get_cell(x + cell_x, y + cell_y) != CellType::EMPTY {
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
