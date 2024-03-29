use std::io;
use std::io::Write;
use std::process::exit;
#[derive(Clone)]
pub struct Level {
    // Game difficulty Level
    pub level: u8,   // current difficulty level
    pub rows: usize, // game table rows
    pub cols: usize, // game table columns
    pub mines: i16,  // this level's sum mines
    pub width: u16,  // screen width needed
    pub height: u16, // screen height needed
}

impl Level {
    pub fn new(level: u8) -> Level {
        // width offset “20” is for the right area to display the help info,
        // this is only for Chinese char's width.
        // If other language displayed abnormally, please modify it bigger
        let width_offset: u16 = 24;
        let height_offset: u16 = 4;

        let mut lv = level;
        if lv == 0 {
            // Select difficult level to init Level / 选择难度级别
            lv = Level::select_level() as u8;
        }
        match lv {
            1 => Level {
                level: lv,
                rows: 8,
                cols: 10,
                mines: 7,
                width: 10 * 4 + 3 + width_offset,
                height: 8 * 2 + height_offset,
            },
            2 => Level {
                level: lv,
                rows: 9,
                cols: 14,
                mines: 15,
                width: 14 * 4 + 3 + width_offset,
                height: 9 * 2 + height_offset,
            },
            3 => Level {
                level: lv,
                rows: 15,
                cols: 20,
                mines: 40,
                width: 20 * 4 + 3 + width_offset,
                height: 15 * 2 + height_offset,
            },
            4 => Level {
                level: lv,
                rows: 19,
                cols: 26,
                mines: 99,
                width: 26 * 4 + 3 + width_offset,
                height: 19 * 2 + height_offset,
            },
            _ => Level {
                level: 1,
                rows: 8,
                cols: 10,
                mines: 7,
                width: 10 * 4 + 3 + width_offset,
                height: 8 * 2 + height_offset,
            },
        }
    }

    pub fn select_level() -> i16 {
        //level==1 新手/Beginner
        //level==2 初级/Basic
        //level==3 中级/Intermediate
        //level==4 高级/Advanced

        print!("\n请选择难度级别|Select difficulty:\n\n   1--新手|Beginner      8x10  [  7 雷|Mines ]\n   2--初级|Basic         9x14  [ 15 雷|Mines ]\n   3--中级|Intermediate 15x20  [ 40 雷|Mines ]\n   4--高级|Advanced     19x26  [ 99 雷|Mines ]\n\n   0--退出|Quit\n\n请选择|Your choice:");

        if io::stdout().flush().is_err() {
            println!("flush err");
        };
        let mut num = Level::input();
        while !(0..=4).contains(&num) {
            println!("请输入0-4以内的数，以确定难度级别");
            num = Level::input();
        }
        if num == 0 {
            exit(0)
        }
        num
    }
    // 处理数字输入
    // deal keyboard input
    pub fn input() -> i16 {
        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();
        let num: i16 = match input.trim().parse::<i16>() {
            Ok(n) => n,
            Err(_) => {
                println!("\x1B[31m\x1B[1m输入错误，请输入数字。\x1B[0m");
                return -1;
            }
        };
        num
    }
}
