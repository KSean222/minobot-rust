use crate::*;
use enumset::EnumSetType;

#[derive(PartialEq, Copy, Clone)]
pub enum CellType {
    Empty,
    Garbage,
    Solid,
    J,
    L,
    S,
    T,
    Z,
    I,
    O
}

#[derive(EnumSetType, Debug)]
pub enum PieceType {
    J,
    L,
    S,
    T,
    Z,
    I,
    O
}

impl PieceType {
    pub fn offset_table(self, r: u8) -> &'static [(i32, i32)] {
        let r = r as usize;
        match self {
            Self::O => &O_OFFSET_TABLE[r],
            Self::I => &I_OFFSET_TABLE[r],
            _ => &JLSTZ_OFFSET_TABLE[r]
        }
    }
    pub fn cells(self, r: u8) -> [(i32, i32); 4] {
        let r = r as usize;
        match self {
            Self::J => J_STATES[r],
            Self::L => L_STATES[r],
            Self::S => S_STATES[r],
            Self::T => T_STATES[r],
            Self::Z => Z_STATES[r],
            Self::I => I_STATES[r],
            Self::O => O_STATES[r]
        }
    }
    pub fn cell(self) -> CellType {
        match self {
            Self::J => CellType::J,
            Self::L => CellType::L,
            Self::S => CellType::S,
            Self::T => CellType::T,
            Self::Z => CellType::Z,
            Self::I => CellType::I,
            Self::O => CellType::O
        }
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum TspinType {
    None,
    Mini,
    Full
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct Piece {
    pub kind: PieceType,
    pub x: i32,
    pub y: i32,
    pub r: u8,
    pub tspin: TspinType
}

impl Piece {
    pub fn spawn(board: &Board<impl Row>, kind: PieceType) -> Self {
        let mut piece = Self {
            kind,
            x: 4,
            y: 20,
            r: 0,
            tspin: TspinType::None
        };
        piece.soft_drop(board);
        piece
    }
    pub fn cells(&self) -> [(i32, i32); 4] {
        let mut cells = self.kind.cells(self.r);
        for (x, y) in &mut cells {
            *x += self.x;
            *y += self.y;
        }
        cells
    }
    pub fn move_left(&mut self, board: &Board<impl Row>) -> bool {
        self.try_move(board, self.x - 1, self.y, self.r)
    }
    pub fn move_right(&mut self, board: &Board<impl Row>) -> bool {
        self.try_move(board, self.x + 1, self.y, self.r)
    }
    pub fn soft_drop(&mut self, board: &Board<impl Row>) -> bool {
        self.try_move(board, self.x, self.y - 1, self.r)
    }
    pub fn turn_left(&mut self, board: &Board<impl Row>) -> bool {
        self.rotate(board,if self.r > 0 { self.r - 1 } else { 3 })
    }
    pub fn turn_right(&mut self, board: &Board<impl Row>) -> bool {
        self.rotate(board,if self.r < 3 { self.r + 1 } else { 0 })
    }
    fn rotate(&mut self, board: &Board<impl Row>, r: u8) -> bool {
        let from_table = self.kind.offset_table(self.r);
        let to_table = self.kind.offset_table(r);
        for (i, (from, to)) in from_table.iter().zip(to_table.iter()).enumerate() {
            let x = self.x + from.0 - to.0;
            let y = self.y + from.1 - to.1;
            if self.try_move(board, x, y, r) {
                if self.kind == PieceType::T {
                    const CORNER_CELLS: [(i32, i32); 4] = [
                        (-1, -1),
                        (-1, 1),
                        (1, 1),
                        (1, -1)
                    ];
                    let mut corners = 0;
                    for &(corner_x, corner_y) in &CORNER_CELLS {
                        if board.occupied(x + corner_x, y + corner_y) {
                            corners += 1;
                        }
                    }
                    if corners > 2 {
                        let front_corner_cells = match r {
                            0 => [(-1, 1), (1, 1)],
                            1 => [(1, 1), (1, -1)],
                            2 => [(-1, -1), (1, -1)],
                            3 => [(-1, -1), (-1, 1)],
                            _ => unreachable!()
                        };
                        let mut front_corners = 0;
                        for &(corner_x, corner_y) in &front_corner_cells {
                            if board.occupied(x + corner_x, y + corner_y) {
                                front_corners += 1;
                            }
                        }
                        self.tspin = if front_corners >= 2 || i == 4 {
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
    fn try_move(&mut self, board: &Board<impl Row>, x: i32, y: i32, r: u8) -> bool {
        let new = Piece {
            kind: self.kind,
            x,
            y,
            r,
            tspin: TspinType::None
        };
        if board.piece_fits(new) {
            *self = new;
            true
        } else {
            false
        }
    }
}

const JLSTZ_OFFSET_TABLE: [[(i32, i32); 5]; 4] = [
    [
        (0, 0),
        (0, 0),
        (0, 0),
        (0, 0),
        (0, 0)
    ], [
        (0, 0),
        (1, 0),
        (1, -1),
        (0, 2),
        (1, 2)
    ], [
        (0, 0),
        (0, 0),
        (0, 0),
        (0, 0),
        (0, 0)
    ], [
        (0, 0),
        (-1, 0),
        (-1, -1),
        (0, 2),
        (-1, 2)
    ]
];

const O_OFFSET_TABLE: [[(i32, i32); 1]; 4] = [
    [
        (0, 0)
    ], [
        (0, -1)
    ], [
        (-1, -1)
    ], [
        (-1, 0)
    ]
];

const I_OFFSET_TABLE: [[(i32, i32); 5]; 4] = [
    [
        (0, 0),
        (-1, 0),
        (2, 0),
        (-1, 0),
        (2, 0)
    ], [
        (-1, 0),
        (0, 0),
        (0, 0),
        (0, 1),
        (0, -2)
    ], [
        (-1, 1),
        (1, 1),
        (-2, 1),
        (1, 0),
        (-2, 0)
    ], [
        (0, 1),
        (0, 1),
        (0, 1),
        (0, -1),
        (0, 2)
    ]
];

const J_STATES: [[(i32, i32); 4]; 4] = [
    [(-1, 1), (-1, 0), (0, 0), (1, 0)],
    [(0, 1), (0, 0), (0, -1), (1, 1)],
    [(-1, 0), (0, 0), (1, 0), (1, -1)],
    [(-1, -1), (0, 1), (0, 0), (0, -1)],
];

const L_STATES: [[(i32, i32); 4]; 4] = [
    [(-1, 0), (0, 0), (1, 1), (1, 0)],
    [(0, 1), (0, 0), (0, -1), (1, -1)],
    [(-1, 0), (-1, -1), (0, 0), (1, 0)],
    [(-1, 1), (0, 1), (0, 0), (0, -1)],
];

const S_STATES: [[(i32, i32); 4]; 4] = [
    [(-1, 0), (0, 1), (0, 0), (1, 1)],
    [(0, 1), (0, 0), (1, 0), (1, -1)],
    [(-1, -1), (0, 0), (0, -1), (1, 0)],
    [(-1, 1), (-1, 0), (0, 0), (0, -1)],
];
const T_STATES: [[(i32, i32); 4]; 4] = [
    [(-1, 0), (0, 1), (0, 0), (1, 0)],
    [(0, 1), (0, 0), (0, -1), (1, 0)],
    [(-1, 0), (0, 0), (0, -1), (1, 0)],
    [(-1, 0), (0, 1), (0, 0), (0, -1)],
];

const Z_STATES: [[(i32, i32); 4]; 4] = [
    [(-1, 1), (0, 1), (0, 0), (1, 0)],
    [(0, 0), (0, -1), (1, 1), (1, 0)],
    [(-1, 0), (0, 0), (0, -1), (1, -1)],
    [(-1, 0), (-1, -1), (0, 1), (0, 0)],
];

const I_STATES: [[(i32, i32); 4]; 4] = [
    [(-1, 0), (0, 0), (1, 0), (2, 0)],
    [(0, 1), (0, 0), (0, -1), (0, -2)],
    [(-2, 0), (-1, 0), (0, 0), (1, 0)],
    [(0, 2), (0, 1), (0, 0), (0, -1)],
];

const O_STATES: [[(i32, i32); 4]; 4] = [
    [(0, 1), (0, 0), (1, 1), (1, 0)],
    [(0, 0), (0, -1), (1, 0), (1, -1)],
    [(-1, 0), (-1, -1), (0, 0), (0, -1)],
    [(-1, 1), (-1, 0), (0, 1), (0, 0)],
];
