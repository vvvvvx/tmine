pub mod display;
pub mod position;

use super::{cell::*, level::Level};
use crossterm::terminal::{disable_raw_mode, enable_raw_mode};
use rand::Rng;
use std::{
    io::{self, stdout, Stdout, Write},
    vec,
};

#[derive(PartialEq,Eq)]
pub enum GameResult {
    Success,
    Failed,
    NotOver,
}
#[derive(Clone, PartialEq,Eq)]
pub enum GameStatus {
    NotStart,
    Started,
    Paused,
    Finished,
}
//#[derive(Clone)]
pub struct Game {
    pub level: Level,               // Game difficult level
    pub mine_table: Vec<Vec<Cell>>, // mine table array
    //timer     : Timer,          // to calculate time consuming
    pub mines_left: i16,
    pub status: GameStatus,
    pub stdout: Stdout,
}

impl Game {
    fn new(level: u8) -> Game {
        //level==1 新手/ Beginner
        //level==2 初级/ Basic
        //level==3 中级/ Intermediate
        //level==4 高级/ Advanced
        let lv = Level::new(level);

        Game {
            level: lv.clone(),
            mine_table: (vec![vec![Cell::new(); lv.cols]; lv.rows]),
            //timer: Timer::new(),
            mines_left: (lv.mines),
            status: GameStatus::NotStart,
            stdout: io::stdout(),
        }
    }
    pub fn new_game(level: u8) -> Game {
        Game::clear_screen();
        disable_raw_mode().expect("Failed to enable raw mode");

        let mut game = Game::new(level);

        // 调整窗口大小，以符合该游戏级别的窗口尺寸要求
        // Adjust the terminal windows size to fit the game level
        print!("\x1B[8;{};{}t", game.level.height, game.level.width);
        stdout().flush().unwrap();

        // Initialize the game
        game.laying_mine();
        game.calc_surrnd_mines_all();
        game.draw_ui();
        enable_raw_mode().expect("Failed to enable raw mode");
        game
    }
    // 计算（x,y）单元格周围的雷数
    // Calculte surrounding mines of the cell (x,y)
    fn calc_mines_1cell(&self, x: &usize, y: &usize) -> i8 {
        let mine_arr = &self.mine_table;

        let max_y = mine_arr.len();
        let max_x = if max_y > 0 { mine_arr[0].len() } else { 0 };

        let min_x: usize = if *x == 0 { 0 } else { x - 1 };
        let min_y: usize = if *y == 0 { 0 } else { y - 1 };

        let mut sum: i8 = 0;
        // for i in min_y..y + 2 {
        //     for j in min_x..x + 2 {
        //         if !(i == *y && j == *x) && i < max_y && j < max_x && mine_arr[i][j].is_mine {
        //             sum += 1;
        //         }
        //     }
        // }
        for (i, _) in mine_arr.iter().enumerate().take(y + 2).skip(min_y) {
            for (j, cell) in mine_arr[i].iter().enumerate().take(x + 2).skip(min_x) {
                if !(i == *y && j == *x) && i < max_y && j < max_x && cell.is_mine {
                    sum += 1;
                }
            }
        }
       
        sum
    }
    //计算周围雷数 / calculate the surrounding mines of cur cell
    fn calc_surrnd_mines_all(&mut self) {
        for i in 0..self.mine_table.len() {
            for j in 0..self.mine_table[i].len() {
                if !self.mine_table[i][j].is_mine {
                    self.mine_table[i][j].surrnd_mines = self.calc_mines_1cell(&j, &i);
                }
            }
        }
    }
    // 翻开单元格 / Dig cell function
    // x、y为mine_arr数组index坐标
    // x y is the mine array's index
    pub fn dig_cell(&mut self, x: &usize, y: &usize, cmd: &char) {
        //let  m: &mut Cell=&mut (self.mine_table[*y][*x]);
        if *x >= self.level.cols || *y >= self.level.rows {
            return;
        }
        match self.mine_table[*y][*x].status {
            //根据单元格状态 / depend on cell's status
            Status::Unexplored => {
                //未开状态
                match cmd {
                    // 标记命令 / Flag cmd
                    'F' => {
                        self.mine_table[*y][*x].status = Status::Flaged;
                        self.mines_left -= 1; //余雷减1
                        self.refresh_cell(x, y);
                        self.update_mine_left_disp(); //更新余雷数量显示
                    }
                    //存疑标记 / Pending cmd
                    'P' => {
                        self.mine_table[*y][*x].status = Status::Pending;
                        self.refresh_cell(x, y);
                    }
                    //挖开命令 / Dig cmd
                    'D' | ' ' | '\n' => {
                        //D、Space、Enter
                        self.mine_table[*y][*x].status = Status::Opened; //修改会未开
                                                                         //修改单元格界面
                        self.refresh_cell(x, y);
                        if self.mine_table[*y][*x].is_mine {
                            //self.failed(); //触发爆炸，程序结束 /if digged mine ,game over
                            return;
                        }
                        if self.mine_table[*y][*x].surrnd_mines == 0 {
                            //递归翻开周围单元格 / if surrnd_mines=0 recursive dig its surrounding cells
                            //限定数组边界 / limit the index range
                            let tx = *x as i16 - 1;
                            let ty = *y as i16 - 1;
                            let min_x = if tx < 0 { &0 } else { &tx };
                            let min_y = if ty < 0 { &0 } else { &ty };

                            let mx = *x + 2;
                            let my = *y + 2;

                            let tmax_x = &self.mine_table[0].len();
                            let tmax_y = &self.mine_table.len();
                            let max_x = if mx > *tmax_x { tmax_x } else { &mx };
                            let max_y = if my > *tmax_y { tmax_y } else { &my };
                            for yy in *min_y as usize..*max_y {
                                for xx in *min_x as usize..*max_x {
                                    if self.mine_table[yy][xx].status == Status::Unexplored
                                        && !(xx == *x && yy == *y)
                                    {
                                        self.dig_cell(&(xx), &(yy), &'D');
                                    }
                                }
                            }
                        }
                    }
                    // 其他不做处理，T-Test命令此处无效 / Test cmd is no use in this condition
                    _ => {} //Test Openator 'T' can't use in this case
                }
            } //Status Unexplored
            Status::Opened => {
                match cmd {
                    // 测试命令 /Test cmd
                    'T' | '\n' | ' ' => {
                        //Test:如果测试周围Flag数量==srrnd_mines,则把周围未开的的全开
                        let tx = *x as i16 - 1;
                        let ty = *y as i16 - 1;
                        let min_x = if tx < 0 { &0 } else { &tx };
                        let min_y = if ty < 0 { &0 } else { &ty };

                        let mx = *x + 2;
                        let my = *y + 2;

                        // let tmax_x = &(self.level.cols as usize);
                        // let tmax_y = &(self.level.rows as usize);
                        let tmax_y = &self.mine_table.len();
                        let tmax_x = &self.mine_table[0].len();

                        let max_x = if mx > *tmax_x { tmax_x } else { &mx };
                        let max_y = if my > *tmax_y { tmax_y } else { &my };
                        let mut sum_mines = 0;
                        //计算周围标记的雷个数 / calculate surrounding mines
                        for yy in *min_y as usize..*max_y {
                            for xx in *min_x as usize..*max_x {
                                if self.mine_table[yy][xx].status == Status::Flaged {
                                    sum_mines += 1;
                                }
                            }
                        }
                        // 如果标记的数量等于单元格总雷数，则把除标记之外的单元格都打开
                        // if cell's surrounding mines== cell.surrnd_mine, dig open the left unopened cells surrounding.
                        if self.mine_table[*y][*x].surrnd_mines == sum_mines {
                            for yy in *min_y as usize..*max_y {
                                for xx in *min_x as usize..*max_x {
                                    if self.mine_table[yy][xx].status != Status::Flaged
                                        && !(xx == *x && yy == *y)
                                    {
                                        if self.mine_table[yy][xx].surrnd_mines > 0 {
                                            self.mine_table[yy][xx].status = Status::Opened;
                                            self.refresh_cell(&xx, &yy);
                                        } else {
                                            // 如果打开的单元格雷数为0,则递归展开
                                            // if cell.surrnd_mines==0,recursive dig the cell
                                            self.dig_cell(&xx, &yy, &'D');
                                        }
                                    }
                                }
                            }
                        }
                    }
                    // Other cmd,do nothing
                    _ => {}
                }
            }
            Status::Flaged => {
                match cmd {
                    // 挖开命令 / Dig cmd
                    'D' => {
                        // 强制撬开 / Although it has been marked as a mine,  still forcibly open
                        self.mine_table[*y][*x].status = Status::Unexplored;
                        self.dig_cell(x, y, &'D');
                    }
                    // Flag cmd
                    'F' => {
                        // 取消标记 / been flaged,now unflag it.
                        self.mine_table[*y][*x].status = Status::Unexplored;
                        self.mines_left += 1;
                        self.refresh_cell(x, y);
                        self.update_mine_left_disp(); //更新余雷数量显示
                    }
                    // Pending cmd
                    'P' => {
                        // 重新标记为Pending  / been flaged ,now re-mark it as pending
                        self.mine_table[*y][*x].status = Status::Pending;
                        self.refresh_cell(x, y);
                    }

                    // Other cmd,do nothing
                    _ => {}
                }
            }
            Status::Pending => {
                match cmd {
                    // Dig cmd
                    'D' | '\n' | ' ' => {
                        //打开
                        self.mine_table[*y][*x].status = Status::Unexplored;
                        self.dig_cell(x, y, &'D');
                    }
                    'F' => {
                        //标记 / Flag it
                        self.mine_table[*y][*x].status = Status::Flaged;
                        self.refresh_cell(x, y);
                    }
                    'P' => {
                        self.mine_table[*y][*x].status = Status::Unexplored;
                        self.refresh_cell(x, y);
                    }
                    // Other cmd,do nothing
                    _ => {}
                }
            }
        } // match Status ended
          //self.success_check();
    }

    // 产生随机数
    fn get_rand(range: usize) -> usize {
        let mut rng = rand::thread_rng();
        rng.gen_range(0, range)
    }
    //随机初始化雷阵 / random init the mine array
    fn laying_mine(&mut self) {
        //各单元格复位 / reset the array
        for i in 0..self.mine_table.len() {
            for j in 0..self.mine_table[i].len() {
                self.mine_table[i][j].reset();
            }
        }
        let mut i = self.level.mines;
        while i > 0 {
            //生成地雷随机坐标 / Generating random coordinates for mines.
            let x = Game::get_rand(self.level.cols);
            let y = Game::get_rand(self.level.rows);
            //布雷
            if !self.mine_table[y][x].is_mine {
                self.mine_table[y][x].is_mine = true;
                i -= 1;
            }
        }
    }

    // Pause game
    pub fn pause(&mut self) {
        for i in 0..self.mine_table.len() {
            for j in 0..self.mine_table[i].len() {
                let (x, y) = Game::pos_from_index(j as u16, i as u16);
                self.move_to(x - 1, y);
                print!("   ");
                self.stdout.flush().unwrap();
            }
        }
        self.status = GameStatus::Paused;
    }

    // Restore the paused game
    pub fn resume(&mut self) {
        self.display_refresh();
        self.status = GameStatus::Started;
    }

    // 检查是否胜利结束 / Check if the game has been successfully ended
    pub fn success_check(&mut self) -> GameResult {
        let mut opened: usize = 0;
        let mut unexplored: usize = 0;
        let mut err_flaged: usize = 0;
        let cells_sum = self.level.rows * self.level.cols;

        // count data
        for i in 0..self.mine_table.len() {
            for j in 0..self.mine_table[i].len() {
                if self.mine_table[i][j].status == Status::Opened {
                    opened += 1;
                }

                if self.mine_table[i][j].status == Status::Unexplored
                    || self.mine_table[i][j].status == Status::Pending
                {
                    unexplored += 1;
                }

                if self.mine_table[i][j].is_mine && self.mine_table[i][j].status == Status::Opened {
                    err_flaged += 1;
                }
            }
        }
        // Failed
        if err_flaged > 0 {
            self.display_failed();
            return GameResult::Failed;
        }

        // Not Over
        // 如果不满足："已翻开总数+总雷数==格子总数"，则未胜利，返回。
        // if not match:“opened cells + sum of mines == sum of all cells”,game is not over,return
        if self.level.mines as usize + opened != cells_sum {
            return GameResult::NotOver;
        }

        // Success
        // 如果未标记数量不等于0,则自动标记所有余下单元格。
        // if succeed and unexplored>0,auto "Flag" all the cells left.
        if unexplored > 0 {
            for i in 0..self.mine_table.len() {
                for j in 0..self.mine_table[i].len() {
                    if self.mine_table[i][j].status == Status::Unexplored
                        || self.mine_table[i][j].status == Status::Pending
                    {
                        self.dig_cell(&j, &i, &'F');
                    }
                }
            }
        }

        //打印胜利信息 / print success info
        self.display_success();

        GameResult::Success
    }
}
