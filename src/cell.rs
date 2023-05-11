#[derive(Clone, PartialEq)]
pub enum Status {
    Opened,     // 已开     /have opened
    Flaged,     // 已标记雷  /Flaged mine
    Unexplored, // 未探明    /Have not explored
    Pending,    // 未决     / Pending
}

#[derive(Clone)]
pub struct Cell {
    // Mine cell
    pub is_mine: bool,    // 是否有雷   / Is it self a mine
    pub surrnd_mines: i8, // 周围雷数   / Sum of surrounding mines
    pub status: Status,   // 单元格状态 / Cell status
}

impl Cell {
    pub fn new() -> Cell {
        Cell {
            is_mine: false,
            surrnd_mines: 0,
            status: Status::Unexplored,
        }
    }
    /*
    pub fn set_mine(&mut self, b: bool) {
        self.is_mine = b;
    }
    pub fn set_surrnd(&mut self, n: i8) {
        self.surrnd_mines = n;
    }
    pub fn set_status(&mut self, s: Status) {
        self.status = s;
    }
    */
    // Reset current cell
    pub fn reset(&mut self) {
        self.is_mine = false;
        self.surrnd_mines = 0;
        self.status = Status::Unexplored;
    }
}
