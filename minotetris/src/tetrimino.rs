use enumset::EnumSetType;
use serde::{ Serialize, Deserialize };

#[derive(PartialEq, Copy, Clone, Serialize, Deserialize)]
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

#[derive(EnumSetType, Debug, Serialize, Deserialize)]
pub enum Tetrimino {
    J,
    L,
    S,
    T,
    Z,
    I,
    O
}

impl Tetrimino {
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
    [(-1, -1), (-1, 0), (0, 0), (1, 0)],
    [(0, -1), (0, 0), (0, 1), (1, -1)],
    [(-1, 0), (0, 0), (1, 0), (1, 1)],
    [(-1, 1), (0, -1), (0, 0), (0, 1)],
];

const L_STATES: [[(i32, i32); 4]; 4] = [
    [(-1, 0), (0, 0), (1, -1), (1, 0)],
    [(0, -1), (0, 0), (0, 1), (1, 1)],
    [(-1, 0), (-1, 1), (0, 0), (1, 0)],
    [(-1, -1), (0, -1), (0, 0), (0, 1)],
];

const S_STATES: [[(i32, i32); 4]; 4] = [
    [(-1, 0), (0, -1), (0, 0), (1, -1)],
    [(0, -1), (0, 0), (1, 0), (1, 1)],
    [(-1, 1), (0, 0), (0, 1), (1, 0)],
    [(-1, -1), (-1, 0), (0, 0), (0, 1)],
];

const T_STATES: [[(i32, i32); 4]; 4] = [
    [(-1, 0), (0, -1), (0, 0), (1, 0)],
    [(0, -1), (0, 0), (0, 1), (1, 0)],
    [(-1, 0), (0, 0), (0, 1), (1, 0)],
    [(-1, 0), (0, -1), (0, 0), (0, 1)],
];

const Z_STATES: [[(i32, i32); 4]; 4] = [
    [(-1, -1), (0, -1), (0, 0), (1, 0)],
    [(0, 0), (0, 1), (1, -1), (1, 0)],
    [(-1, 0), (0, 0), (0, 1), (1, 1)],
    [(-1, 0), (-1, 1), (0, -1), (0, 0)],
];

const I_STATES: [[(i32, i32); 4]; 4] = [
    [(-1, 0), (0, 0), (1, 0), (2, 0)],
    [(0, -1), (0, 0), (0, 1), (0, 2)],
    [(-2, 0), (-1, 0), (0, 0), (1, 0)],
    [(0, -2), (0, -1), (0, 0), (0, 1)],
];

const O_STATES: [[(i32, i32); 4]; 4] = [
    [(0, -1), (0, 0), (1, -1), (1, 0)],
    [(0, 0), (0, 1), (1, 0), (1, 1)],
    [(-1, 0), (-1, 1), (0, 0), (0, 1)],
    [(-1, -1), (-1, 0), (0, -1), (0, 0)],
];
