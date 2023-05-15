use super::*;
use crossterm::{
    cursor,
    terminal::{self},
    QueueableCommand,
};
use std::io::{self, Write};
impl super::Game {
    //清屏 / Clear screen
    pub(super) fn clear_screen() {
        // 获取标准输出流
        let mut stdout = io::stdout();
        // 清屏 / clear screen.
        stdout
            .queue(terminal::Clear(terminal::ClearType::All))
            .unwrap();
        stdout.queue(cursor::MoveTo(0, 0)).unwrap();
    }

    // cancel reverse display the row
    pub fn cancel_rever_row(&mut self, row: u16) {
        let (x, y) = super::Game::pos_from_index(0, row);
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
            let (x, y) = super::Game::pos_from_index(col, i as u16);
            self.move_to(x - 1, y);
            self.refresh_cell(&(col as usize), &i);
            self.move_to(x - 1, y + 1);
            print!("───");
        }
        self.refresh_cell(&(col as usize), &(self.level.rows - 1));
        self.stdout.flush().unwrap();
    }
    // Refresh the mine table of terminal UI
    pub(super) fn display_refresh(&mut self) {
        for i in 0..self.mine_table.len() {
            for j in 0..self.mine_table[i].len() {
                self.refresh_cell(&j, &i)
            }
        }
    }

    // Display all mines in terminal UI
    fn display_mine(&mut self) {
        for i in 0..self.mine_table.len() {
            for j in 0..self.mine_table[i].len() {
                if self.mine_table[i][j].status == Status::Opened
                    || (self.mine_table[i][j].status == Status::Flaged
                        && self.mine_table[i][j].is_mine)
                {
                    //如果已经打开了，则不管。
                    continue;
                }
                if self.mine_table[i][j].is_mine {
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
                if self.mine_table[i][j].status == Status::Flaged && !self.mine_table[i][j].is_mine
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
    pub(super) fn display_failed(&mut self) {
        self.display_mine();
        let (x, y) = self.get_table_mid_pos();
        self.move_to(x - 4, y - 1);
        print!("\x1B[31m\x1B[5m\x1B[1m 您失败了！ \x1B[0m");
        self.move_to(x - 4, y);
        print!("\x1B[31m\x1B[5m\x1B[1mYou failed!\x1B[0m");

        self.stdout.flush().unwrap();
    }
    // Display info of success
    pub(super) fn display_success(&mut self) {
        self.update_mine_left_disp();
        let (x, y) = self.get_table_mid_pos(); // get the center position of the table UI.
        self.move_to(x - 4, y - 1);
        print!("\x1B[32m\x1B[5m\x1B[1m您胜利了!\n\x1B[0m");
        self.move_to(x - 4, y);
        print!("\x1B[32m\x1B[5m\x1B[1mYou won !\n\x1B[0m");
        self.stdout.flush().unwrap();
    }
    //绘制界面 / Draw mine table on terminal UI
    pub(super) fn draw_ui(&mut self) {
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
        for _i in 0..col {
            jj_line += "   │"
        }
        //   打印中间行 / Print middle lines
        for i in 0..row - 1 {
            let c = 65 + i;
            print!("{}", (c as u8) as char); //打印左侧行序号 / Print row numbers on the left ,like ABCDEFG...
            self.stdout.write_all(jj_line.as_bytes()).unwrap();
            println!("{}", (c as u8) as char); //打印右侧行序号 / Print row numbers on the right ,like ABCDEF....
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
                y3_ += 1;
            }
            self.move_to(x3, y3_ + 5);
            print!("Left Click  = D/T");
            self.move_to(x3, y3_ + 6);
            print!("Right Click = F");
            y3_ += 2;
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
    pub(super) fn refresh_cell(&mut self, x: &usize, y: &usize) {
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
            self.rever_disp_cell(&(col as usize), &i);
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

    //更新余雷数量 / refresh mine left info to UI
    pub(super) fn update_mine_left_disp(&mut self) {
        let (x, y) = self.get_stat_mine_pos();
        self.move_to(x + 10, y);
        print!("{} ", self.mines_left);
        self.stdout.flush().unwrap();
    }
}
