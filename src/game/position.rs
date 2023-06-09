use crossterm::{cursor, QueueableCommand};
impl super::Game {
    // convert row index from input char
    pub fn get_row_from_char(&self, c: &char) -> i16 {
        let c1 = c.to_ascii_uppercase();

        //if !(c1 >= 'A' && c1 <= 'Z') {
        if !c1.is_ascii_uppercase() {
            return -1;
        }
        let row = c1 as u8 - 65;
        if row as usize >= self.level.rows {
            return -1;
        }
        row as i16
    }
    // convert column index from input char
    // return -1,means error
    pub fn get_col_from_char(&self, c: &char) -> i16 {
        let c1 = c.to_ascii_uppercase();

        //if !(c1 >= 'A' && c1 <= 'Z') {
        if !c1.is_ascii_uppercase() {
            return -1;
        }
        let col = c1 as u8 - 65;
        if col as usize >= self.level.cols {
            return -1;
        }
        col as i16
    }
    // return (row,column)
    // row or col equal -1 ,means error
    pub fn get_row_col_from_str(&self, cmd: &String) -> (i16, i16) {
        let len = cmd.len();
        if len == 0 {
            return (-1, -1);
        }
        //let c1 = cmd.chars().nth(0).unwrap();
        let c1 = cmd.chars().next().unwrap();
        //if !(c1 >= 'A' && c1 <= 'Z') {
        if !c1.is_ascii_uppercase() {
            return (-1, -1);
        }
        match len {
            1 => (self.get_row_from_char(&c1), -1),
            _ => {
                let c2 = cmd.chars().nth(1).unwrap();
                (self.get_row_from_char(&c1), self.get_col_from_char(&c2))
            }
        }
    }
    // 计算命令提示信息打印位置 / get the  UI position of cmd to output
    // Return (col,row)
    pub fn get_cmd_pos(&self) -> (u16, u16) {
        (
            (4 * self.level.cols + 6) as u16,
            ((self.level.rows + 1) * 2) as u16,
        )
    }
    // 计算帮助信息打印位置 /get the UI position of "Help" info to print
    // Return (col,row)
    pub(super) fn get_help_pos(&self) -> (u16, u16) {
        ((4 * self.level.cols + 6) as u16, 4)
    }
    // 计算余雷统计信息打印位置 / get the UI position of "Mine left" info to print
    // Return (col,row)
    pub fn get_stat_mine_pos(&self) -> (u16, u16) {
        ((4 * self.level.cols + 6) as u16, 0)
    }
    //计算耗时显示信息打印位置 / get the UI position of "Time consuming" info to print
    // Return (col,row)
    pub fn get_stat_time_pos(&self) -> (u16, u16) {
        ((4 * self.level.cols + 6) as u16, 1)
    }

    // 获取屏幕表格中间位置，用于显示游戏结果信息
    // Get the center position of the table of UI. for displaying result info of game.
    // Return (col,row)
    pub(super) fn get_table_mid_pos(&self) -> (u16, u16) {
        (
            ((4 * self.level.cols + 3) / 2) as u16,
            (self.level.rows) as u16,
        )
    }

    pub fn move_to(&mut self, col: u16, row: u16) {
        self.stdout.queue(cursor::MoveTo(col, row)).unwrap();
    }

    // 根据行列定位表格坐标，提供给cursor::MoveTo()使用
    // base on table column and row number to locate the screen position,provide to cursor::MoveTo()
    pub(super) fn pos_from_index(col: u16, row: u16) -> (u16, u16) {
        ((3 + 4 * col), (2 + row * 2))
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
        (y_, x_)
    }

    // 哪条红色命令被左击，返回命令字符
    pub fn which_cmd_clicked(&mut self, col: u16, row: u16) -> char {
        let x_left = (4 * self.level.cols + 6) as u16;
        let x_right: u16 = x_left + 18;
        // quit命令显示所在的Y坐标
        let y_quit: u16 = match self.level.level {
            1 => 12,
            2 => 14,
            _ => 16,
        };
        // col位置不在红色cmd区域，直接返回
        if !(col >= x_left && col <= x_right) {
            return 'X';
        }
        // row位置离Quit cmd行的偏差
        match row - y_quit {
            0 => 'Q', // Quit
            1 => 'P', // Pause
            2 => 'R', // Resume
            3 => 'N', // New
            4 => 'D', // Difficulty
            _ => 'X', // Not cmd
        }
    }
}
