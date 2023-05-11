use super::{cell::*, level::Level};
use crossterm::{
    cursor,
    terminal::{self, disable_raw_mode, enable_raw_mode},
    QueueableCommand,
};
use rand::Rng;
use std::{
    io::{self, stdout, Stdout, Write},
    vec,
};

#[derive(PartialEq)]
pub enum GameResult {
    Success,
    Failed,
    NotOver,
}
#[derive(Clone, PartialEq)]
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
    pub fn new(level: u8) -> Game {
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

    // cancel reverse display the row
    pub fn cancel_rever_row(&mut self, row: u16) {
        let (x, y) = Game::pos_from_index(0, row);
        self.move_to(x - 1, y);
        for i in 0..(self.level.cols - 1) {
            self.refresh_cell(&i, &(row as usize));
            print!("│");
        }
        self.refresh_cell(&(self.level.cols - 1), &(row as usize));
        self.stdout.flush().unwrap();
    }
    // cancel reverse display the column
    pub fn cancel_rever_col(&mut self, col: u16) {
        for i in 0..(self.level.rows - 1) {
            let (x, y) = Game::pos_from_index(col, i as u16);
            self.move_to(x - 1, y);
            self.refresh_cell(&(col as usize), &(i as usize));
            self.move_to(x - 1, y + 1);
            print!("───");
        }
        self.refresh_cell(&(col as usize), &(self.level.rows - 1));
        self.stdout.flush().unwrap();
    }
    // 计算（x,y）单元格周围的雷数
    // Calculte surrounding mines of the cell (x,y)
    pub fn calc_mines_1cell(&self, x: &usize, y: &usize) -> i8 {
        let mine_arr = &self.mine_table;

        let max_y = mine_arr.len();
        let max_x = if max_y > 0 { mine_arr[0].len() } else { 0 };

        let min_x: usize = if *x == 0 { 0 } else { x - 1 };
        let min_y: usize = if *y == 0 { 0 } else { y - 1 };

        let mut sum: i8 = 0;
        for i in min_y..y + 2 {
            for j in min_x..x + 2 {
                if !(i == *y && j == *x) && i < max_y && j < max_x && mine_arr[i][j].is_mine == true
                {
                    sum += 1;
                }
            }
        }
        return sum;
    }
    //计算周围雷数 / calculate the surrounding mines of cur cell
    pub fn calc_surrnd_mines_all(&mut self) {
        for i in 0..self.mine_table.len() {
            for j in 0..self.mine_table[i].len() {
                if self.mine_table[i][j].is_mine == false {
                    self.mine_table[i][j].surrnd_mines = self.calc_mines_1cell(&j, &i);
                }
            }
        }
    }
    // Refresh the mine table of terminal UI
    pub fn display_refresh(&mut self) {
        for i in 0..self.mine_table.len() {
            for j in 0..self.mine_table[i].len() {
                self.refresh_cell(&j, &i)
            }
        }
    }

    // Display all mines in terminal UI
    pub fn display_mine(&mut self) {
        for i in 0..self.mine_table.len() {
            for j in 0..self.mine_table[i].len() {
                if self.mine_table[i][j].status == Status::Opened
                    || (self.mine_table[i][j].status == Status::Flaged
                        && self.mine_table[i][j].is_mine)
                {
                    //如果已经打开了，则不管。
                    continue;
                }
                if self.mine_table[i][j].is_mine == true {
                    let (x, y) = Game::pos_from_index(j as u16, i as u16);
                    self.move_to(x, y);
                    print!("\x1B[31mM\x1B[0m");
                    self.stdout.flush().unwrap();
                }
            }
        }
    }
    //Display error flaged cell
    pub fn display_err(&mut self) {
        for i in 0..self.mine_table.len() {
            for j in 0..self.mine_table[i].len() {
                if self.mine_table[i][j].status == Status::Flaged
                    && self.mine_table[i][j].is_mine == false
                {
                    let (x, y) = Game::pos_from_index(j as u16, i as u16);
                    self.move_to(x, y);
                    print!("\x1B[31m\x1B[43m F \x1B[0m");
                    self.stdout.flush().unwrap();
                }
            }
        }
    }

    //失败，画面爆炸 / display info of game failed
    pub fn display_failed(&mut self) {
        self.display_mine();
        let (x, y) = self.get_table_mid_pos();
        self.move_to(x - 4, y - 1);
        print!("\x1B[31m\x1B[5m\x1B[1m 您失败了！ \x1B[0m");
        self.move_to(x - 4, y);
        print!("\x1B[31m\x1B[5m\x1B[1mYou failed!\x1B[0m");

        self.stdout.flush().unwrap();
    }
    // Display info of success
    pub fn display_success(&mut self) {
        self.update_mine_left_disp();
        let (x, y) = self.get_table_mid_pos(); // get the center position of the table UI.
        self.move_to(x - 4, y - 1);
        print!("\x1B[32m\x1B[5m\x1B[1m您胜利了!\n\x1B[0m");
        self.move_to(x - 4, y);
        print!("\x1B[32m\x1B[5m\x1B[1mYou won !\n\x1B[0m");
        self.stdout.flush().unwrap();
    }
    // 翻开单元格 / Dig cell function
    // x、y为mine_arr数组index坐标
    // x y is the mine array's index
    pub fn dig_cell(&mut self, x: &usize, y: &usize, cmd: &char) {
        //let  m: &mut Cell=&mut (self.mine_table[*y][*x]);
        if *x >= self.level.cols as usize || *y >= self.level.rows as usize {
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

                        let tmax_x = &(self.level.cols as usize);
                        let tmax_y = &(self.level.rows as usize);
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

    //绘制界面 / Draw mine table on terminal UI
    pub fn draw_ui(&mut self) {
        let row = self.level.rows;
        let col = self.level.cols;

        // 清屏 / clear screen
        self.stdout
            .queue(terminal::Clear(terminal::ClearType::All))
            .unwrap();
        self.move_to(0, 0);
        //打印列序号 / Print column number on top line, ABCDEFG...
        print!("   {}", (65) as char);
        for i in 0..col - 1 {
            let c = 66 + i;
            print!("   {}", (c as u8) as char);
        }
        print!("\n");
        //打印表格第一行 / Print the firest line of table
        let mut first_line: String = String::from(" ┌");
        for _i in 0..col - 1 {
            first_line += "───┬";
        }
        first_line += "───┐\n";
        self.stdout.write_all(first_line.as_bytes()).unwrap();
        //打印表格中间行 / Print the middle lines of the table
        //   拼接中间行字符串 / Concatenate row string of middle line
        let mut mid_line: String = String::from(" ├");
        for _i in 0..col - 1 {
            mid_line += "───┼";
        }
        mid_line += "───┤\n";

        //   拼接衔接列
        let mut jj_line: String = String::from("│");
        for _i in 0..col + 0 {
            jj_line += "   │"
        }
        //   打印中间行 / Print middle lines
        for i in 0..row - 1 {
            let c = 65 + i;
            print!("{}", (c as u8) as char); //打印左侧行序号 / Print row numbers on the left ,like ABCDEFG...
            self.stdout.write_all(jj_line.as_bytes()).unwrap();
            print!("{}\n", (c as u8) as char); //打印右侧行序号 / Print row numbers on the right ,like ABCDEF....
            self.stdout.write_all(mid_line.as_bytes()).unwrap();
        }
        //打印底行 / Print bottom line
        let mut bot_line: String = String::from(" └");
        for _i in 0..col - 1 {
            bot_line += "───┴";
        }
        bot_line += "───┘\n";
        let c = 65 + row - 1;
        print!("{}", (c as u8) as char); //打印最后一行左侧行序号 / Print the last line's row number on the left
        self.stdout.write_all(jj_line.as_bytes()).unwrap();
        print!("{}\n", (c as u8) as char); //打印最后一行右侧行序号 / Print the last line's row number on the right
        self.stdout.write_all(bot_line.as_bytes()).unwrap();
        //打印底边列号 / Print the column nubmer on bottom.
        print!("   {}", (65) as char);
        for i in 0..col - 1 {
            let c = 66 + i;
            print!("   {}", (c as u8) as char);
        }
        print!("\n");

        //打印提示信息 / Print statistics info
        let (x, y) = self.get_stat_mine_pos();
        self.move_to(x, y);
        print!("余雷Mine: {}", self.level.mines); //Mines left
        let (x1, y1) = self.get_stat_time_pos();
        self.move_to(x1, y1);
        print!("耗时Time:"); //Time used
        let (x2, y2) = self.get_cmd_pos();
        self.move_to(x2, y2);
        print!("Input:");
        // 打印帮助信息
        let (x3, y3) = self.get_help_pos();
        self.move_to(x3, y3 - 1);
        print!("───────HELP───────"); //──Help──
        self.move_to(x3, y3);
        print!("操作:行+列+命令"); // print!("CMD:Row+Col+Cmd")
        self.move_to(x3, y3 + 1);
        print!("Oper:Row+Col+CMD"); // print!("CMD:Row+Col+Cmd")

        let mut y3_ = y3 + 3;
        if self.level.level <= 2 {
            y3_ = y3 + 2;
        }
        self.move_to(x3, y3_);
        print!("─────命令|CMD─────");
        self.move_to(x3, y3_ + 1);
        print!("F-Flag        标记");
        self.move_to(x3, y3_ + 2);
        print!("D-Dig         翻开");
        self.move_to(x3, y3_ + 3);
        print!("T-Test        测试");
        self.move_to(x3, y3_ + 4);
        print!("P-Pending     疑问");

        if self.level.level > 1 {
            if self.level.level > 2 {
                y3_ = y3_ + 1;
            }
            self.move_to(x3, y3_ + 5);
            print!("Left Click  = D/T");
            self.move_to(x3, y3_ + 6);
            print!("Right Click = F");
            y3_ = y3_ + 2;
        }

        self.move_to(x3, y3_ + 6);
        print!("!Q-Quit       退出");
        self.move_to(x3, y3_ + 7);
        print!("!P-Pause      暂停");
        self.move_to(x3, y3_ + 8);
        print!("!R-Resume     恢复");
        self.move_to(x3, y3_ + 9);
        print!("!N-New        重玩");
        self.move_to(x3, y3_ + 10);
        print!("!D-Difficulty换难度");
        self.stdout.flush().unwrap();
    }
    // display the input char at Input area.
    pub fn echo_cmd(&mut self, cmd: &String) {
        let (x, y) = self.get_cmd_pos();
        self.move_to(x + 7, y);
        // clear history cmd.
        print!("   ");
        self.move_to(x + 7, y);
        print!("{}", *cmd);
        self.stdout.flush().unwrap();
    }
    // 产生随机数
    pub fn get_rand(range: usize) -> usize {
        let mut rng = rand::thread_rng();
        return rng.gen_range(0, range);
    }
    // 计算命令提示信息打印位置 / get the  UI position of cmd to output
    // Return (col,row)
    pub fn get_cmd_pos(&self) -> (u16, u16) {
        return (
            (4 * self.level.cols + 6) as u16,
            ((self.level.rows + 1) * 2) as u16,
        );
    }
    // 计算帮助信息打印位置 /get the UI position of "Help" info to print
    // Return (col,row)
    fn get_help_pos(&self) -> (u16, u16) {
        return ((4 * self.level.cols + 6) as u16, 4);
    }
    // 计算余雷统计信息打印位置 / get the UI position of "Mine left" info to print
    // Return (col,row)
    pub fn get_stat_mine_pos(&self) -> (u16, u16) {
        return ((4 * self.level.cols + 6) as u16, 0);
    }
    //计算耗时显示信息打印位置 / get the UI position of "Time consuming" info to print
    // Return (col,row)
    pub fn get_stat_time_pos(&self) -> (u16, u16) {
        return ((4 * self.level.cols + 6) as u16, 1);
    }

    // 获取屏幕表格中间位置，用于显示游戏结果信息
    // Get the center position of the table of UI. for displaying result info of game.
    // Return (col,row)
    fn get_table_mid_pos(&self) -> (u16, u16) {
        return (
            ((4 * self.level.cols + 3) / 2) as u16,
            (self.level.rows) as u16,
        );
    }
    // convert row index from input char
    pub fn get_row_from_char(&self, c: &char) -> i16 {
        let c1 = c.to_ascii_uppercase();

        if !(c1 >= 'A' && c1 <= 'Z') {
            return -1;
        }
        let row = c1 as u8 - 65;
        if row as usize >= self.level.rows {
            return -1;
        }
        return row as i16;
    }
    // convert column index from input char
    // return -1,means error
    pub fn get_col_from_char(&self, c: &char) -> i16 {
        let c1 = c.to_ascii_uppercase();

        if !(c1 >= 'A' && c1 <= 'Z') {
            return -1;
        }
        let col = c1 as u8 - 65;
        if col as usize >= self.level.cols {
            return -1;
        }
        return col as i16;
    }
    // return (row,column)
    // row or col equal -1 ,means error
    pub fn get_row_col_from_str(&self, cmd: &String) -> (i16, i16) {
        let len = cmd.len();
        if len == 0 {
            return (-1, -1);
        }
        let c1 = cmd.chars().nth(0).unwrap();
        if !(c1 >= 'A' && c1 <= 'Z') {
            return (-1, -1);
        }
        match len {
            1 => {
                return (self.get_row_from_char(&c1), -1);
            }
            _ => {
                let c2 = cmd.chars().nth(1).unwrap();
                return (self.get_row_from_char(&c1), self.get_col_from_char(&c2));
            }
        }
    }
    //随机初始化雷阵 / random init the mine array
    pub fn laying_mine(&mut self) {
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
            if self.mine_table[y][x].is_mine == false {
                self.mine_table[y][x].is_mine = true;
                i -= 1;
            }
        }
    }

    pub fn move_to(&mut self, col: u16, row: u16) {
        self.stdout.queue(cursor::MoveTo(col, row)).unwrap();
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

    // 根据行列定位表格坐标，提供给cursor::MoveTo()使用
    // base on table column and row number to locate the screen position,provide to cursor::MoveTo()
    fn pos_from_index(col: u16, row: u16) -> (u16, u16) {
        return ((3 + 4 * col), (2 + row * 2));
    }

    // 刷新单元格内容 / Refresh cell display when cell's status changed
    // ASCI转移字符
    // 红色：\x1B[31m
    // 绿色：\x1B[32m
    // 黄色：\x1B[33m
    // 蓝色：\x1B[34m
    // 洋红色：\x1B[35m
    // 青色：\x1B[36m
    // 92淡绿色

    // 背景：

    //     40：黑色
    //     41：红色
    //     42：绿色
    //     43：黄色
    //     44：蓝色
    //     45：洋红色
    //     46：青色
    //     47：白色

    // 绿色背景：\x1b[42m
    // 重置：\x1B[0m

    pub fn refresh_cell(&mut self, x: &usize, y: &usize) {
        let (x1, y1) = Game::pos_from_index(*x as u16, *y as u16);
        self.move_to(x1 - 1, y1);
        let c = &self.mine_table[*y][*x];
        match c.status {
            Status::Opened => {
                if !c.is_mine {
                    //如果不是雷 / if the cell is not mine.
                    if c.surrnd_mines == 0 {
                        //如果周围没有雷，显示灰色背景， / display three space char with gray background:\x1B[100m
                        print!("\x1B[90m\x1B[100m   \x1B[0m");
                    }
                    if c.surrnd_mines > 0 {
                        // 如果周围有雷，显示绿色数字
                        // if the cell is not mine but its surrounding has mines ,
                        // display surrounding mines with gray BG and Green font
                        print!("\x1B[32;1m\x1B[100m {} \x1B[0m", c.surrnd_mines);
                    }
                } else {
                    //如果是雷，显示红色“M”，黄色背景 / if the cell is mine ,display 'M' with yellow BG( \x1B[43m ) and Red font ( \x1B[31m )
                    print!("\x1B[31m\x1B[43m M \x1B[0m");
                }
            }
            Status::Flaged => {
                // 如果已经标记为雷，显示红色F
                // if flaged the cell ,dispaly 'F' with gray BG ( \x1B[100m ) and Red font ( \x1B[31m )
                //print!("\x1B[31m\x1B[100m F \x1B[0m");
                print!("\x1B[31m F \x1B[0m");
            }
            Status::Pending => {
                // 如果存疑，显示蓝色问号
                // if Pending ,dispaly '?' with gray BG and Blue font
                //print!("\x1B[34m\x1B[100m ? \x1B[0m");
                print!("\x1B[33m ? \x1B[0m");
            }
            Status::Unexplored => {
                // unexplored ,with no color
                print!("   ");
            } //_=>{ }
        }
        self.stdout.flush().unwrap();
    }

    // Restore the paused game
    pub fn resume(&mut self) {
        self.display_refresh();
        self.status = GameStatus::Started;
    }
    // reverse display the row
    pub fn rever_disp_row(&mut self, row: u16) {
        let (x, y) = Game::pos_from_index(0, row);
        self.move_to(x - 1, y);
        for i in 0..(self.mine_table[row as usize].len() - 1) {
            self.rever_disp_cell(&i, &(row as usize));
            print!("\x1B[42m│\x1B[0m");
        }
        self.rever_disp_cell(&(self.level.cols - 1), &(row as usize));
        self.stdout.flush().unwrap();
    }
    // reverse display the column
    pub fn rever_disp_col(&mut self, col: u16) {
        for i in 0..(self.level.rows - 1) {
            let (x, y) = Game::pos_from_index(col, i as u16);
            self.move_to(x - 1, y);
            self.rever_disp_cell(&(col as usize), &(i as usize));
            self.move_to(x - 1, y + 1);
            print!("\x1B[42m───\x1B[0m");
        }
        self.rever_disp_cell(&(col as usize), &(self.level.rows - 1));
        self.stdout.flush().unwrap();
    }

    // reverse display the cell
    // x=col y=row
    fn rever_disp_cell(&mut self, x: &usize, y: &usize) {
        let (x1, y1) = Game::pos_from_index(*x as u16, *y as u16);
        self.move_to(x1 - 1, y1);
        let c = &self.mine_table[*y][*x];
        match c.status {
            Status::Opened => {
                if !c.is_mine {
                    //如果不是雷 / if the cell is not mine.
                    if c.surrnd_mines == 0 {
                        // display three space char with green background:\x1b[42m
                        print!("\x1b[42m   \x1B[0m");
                    }
                    if c.surrnd_mines > 0 {
                        // if the cell is not mine but its surrounding has mines ,
                        // display surrounding mines with gray BG and black font
                        print!("\x1B[42m {} \x1B[0m", c.surrnd_mines);
                    }
                } else {
                    //如果是雷，显示红色“M”，绿色背景 / if the cell is mine ,display 'M' with yellow BG( \x1B[42m ) and Red font ( \x1B[31m )
                    print!("\x1B[31m\x1B[42m M \x1B[0m");
                }
            }
            Status::Flaged => {
                // 如果已经标记为雷，显示红色F
                // if flaged the cell ,dispaly 'F' with gray BG ( \x1B[100m ) and Red font ( \x1B[31m )
                print!("\x1B[31m\x1B[42m F \x1B[0m");
            }
            Status::Pending => {
                // 如果存疑，显示蓝色问号
                // if Pending ,dispaly '?' with gray BG and Blue font
                print!("\x1B[34m\x1B[42m ? \x1B[0m");
            }
            Status::Unexplored => {
                // unexplored ,with green BG
                //print!("   ");
                print!("\x1b[42m   \x1B[0m");
            } //_=>{ }
        }
        self.stdout.flush().unwrap();
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

                if self.mine_table[i][j].is_mine == true
                    && self.mine_table[i][j].status == Status::Opened
                {
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

        return GameResult::Success;
    }

    //更新余雷数量 / refresh mine left info to UI
    fn update_mine_left_disp(&mut self) {
        let (x, y) = self.get_stat_mine_pos();
        self.move_to(x + 10, y);
        print!("{} ", self.mines_left);
        self.stdout.flush().unwrap();
    }
    //清屏 / Clear screen
    fn clear_screen() {
        // 获取标准输出流
        let mut stdout = io::stdout();
        // 清屏 / clear screen.
        stdout
            .queue(terminal::Clear(terminal::ClearType::All))
            .unwrap();
        stdout.queue(cursor::MoveTo(0, 0)).unwrap();
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
        return game;
    }
    // Calculate cell index based on mouse position
    // return value (row,col)
    pub fn pos_to_index(&mut self, row: u16, col: u16) -> (i16, i16) {
        let y = (row - 1).div_euclid(2);
        let x = (col - 1).div_euclid(4);
        // 余数==0,表明鼠标在表格线上
        // remainder==0 means mouse cursor is over the table divider line.
        let y_re = (row - 1) % 2;
        let x_re = (col - 1) % 4;

        let y_ = if y > self.level.rows as u16 || y_re == 0 {
            -1
        } else {
            y as i16
        };
        let x_ = if x > self.level.cols as u16 || x_re == 0 {
            -1
        } else {
            x as i16
        };
        return (y_, x_);
    }
} //impl Game ended
