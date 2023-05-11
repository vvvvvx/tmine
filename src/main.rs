//-----------TMine-----------
// Begin date : 2023-04-07
// Finish date: 2023-04-11
// Author     : vvvvvx
// Address    : China
//-----------TMine-----------

use crossterm::{
    cursor::{Hide, Show},
    event::{read, Event, KeyCode, KeyEvent},
    event::{DisableMouseCapture, EnableMouseCapture},
    event::{KeyEventKind, MouseEventKind},
    event::{MouseButton, MouseEvent},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode},
};
use std::{
    io::{self, Write},
    sync::{
        mpsc::{channel, Receiver, Sender},
        Arc, Mutex,
    },
    thread::{self, sleep},
    time::Duration,
};

struct SharePos {
    time_pos_x: u16,
    time_pos_y: u16,
}
use tmine::game::*;
use tmine::timer::{Timer, TimerStatus};
fn main() -> io::Result<()> {
    let mut game = Game::new_game(0);
    let mut cmd: String = String::new();
    let update_interval = Duration::from_secs(1); //update time consuming interval.
                                                  //隐藏光标 / Hide cursor
    execute!(std::io::stdout(), Hide).unwrap();
    //开启raw mode,监听键盘输入 / enable raw mode to listen keyboard input.
    enable_raw_mode().expect("Failed to enable raw mode");
    execute!(std::io::stdout(), EnableMouseCapture)?;
    // 创建一个通信通道，用于主线程向子线程发送 timer status 变量
    // Create a channel to control the timer by main thread.
    let (ch_sender, ch_receiver): (Sender<TimerStatus>, Receiver<TimerStatus>) = channel();

    // Get time consuming display postition
    let (mut x_t, mut y_t) = game.get_stat_time_pos();

    // Create a variable to share the position between the main thread and the timer thread.
    let shared_var = Arc::new(Mutex::new(SharePos {
        time_pos_x: x_t,
        time_pos_y: y_t,
    }));

    let shared_var_clone = shared_var.clone();

    // Create a timer
    let mut timer = Timer::new();

    // 启动一个子线程，用于更新耗时信息
    // run a timer thread to update time consuming
    thread::spawn(move || -> ! {
        loop {
            // Get the command from main thread
            let mut status_t = TimerStatus::NotStart;
            match ch_receiver.try_recv() {
                Ok(new_status) => {
                    status_t = new_status;
                }
                Err(_) => {}
            };
            // 判断 timer status 变量的值,操作timer
            match status_t {
                //match *status {
                TimerStatus::Pause => {
                    timer.pause();
                    continue;
                }
                TimerStatus::Start => {
                    timer.start();
                }
                TimerStatus::Resume => {
                    timer.resume();
                }
                TimerStatus::Stop => {
                    timer.stop();
                }
                _ => {}
            }
            // get the lock of shared_var
            let sh_pos = shared_var_clone.lock().unwrap();
            // 检查是否到了定时器的时间间隔，如果是则更新时间并输出到屏幕上
            // refresh time consuming
            let elapsed = timer.last.elapsed();
            if elapsed >= update_interval && timer.is_running {
                timer.last += update_interval;
                // use the data of shared_var
                timer.update_time_consuming(sh_pos.time_pos_x, sh_pos.time_pos_y);
            }
            //must release the lock of shared_var
            drop(sh_pos);
            // 程序休眠 100 毫秒，以减少 CPU 资源的消耗
            // sleep to reduce the CPU consuming
            sleep(Duration::from_millis(100));
        }
    });

    // Main thread loop
    loop {
        //(x,y)=game.get_cmd_pos();
        //game.move_to(x+7, y); //cursor to cmd input postion
        let ev = read().expect("Failed to read event");
        let game_is_pause_or_finished =
            game.status == GameStatus::Paused || game.status == GameStatus::Finished;
        //match read().expect("Failed to read event") {
        match ev {
            Event::Key(KeyEvent {
                code: KeyCode::Char(c),
                kind,
                ..
            }) => {
                // 按下一个键会同时发送Press和Release两个Event,所以会收到重复字符，仅接收Press事件类型。
                // when press a key,will send Press and Release 2 events ,so one char will repeat
                // two times,to avoid it here only deal with the Press event, do nothing when key released.
                if kind == KeyEventKind::Release {
                    continue;
                };

                // Can not input more than 3 chars when cell operation.
                // Can not input more than 2 chars when program operation
                if cmd.len() > 0 {
                    let fc = cmd.chars().nth(0).unwrap();
                    if fc == '!' {
                        if cmd.len() >= 2 {
                            continue;
                        }
                    }
                    if cmd.len() >= 3 {
                        continue;
                    }
                }

                cmd += c.to_ascii_uppercase().to_string().as_str();
                game.echo_cmd(&cmd);

                let c1 = cmd.chars().nth(0).unwrap();
                if !game_is_pause_or_finished {
                    match cmd.len() {
                        // reverse display row
                        1 => {
                            // Only process if the first input is a letter,
                            // meaning if the first input is a row number.
                            // Do not process when first input is '!'.
                            if c1 >= 'A' && c1 <= 'Z' {
                                let row = game.get_row_from_char(&c);
                                if row != -1 {
                                    game.rever_disp_row(row as u16);
                                }
                            }
                        }
                        // reverse display column
                        2 => {
                            // Only process if the first input is a letter,
                            // meaning if the first input is a row number.
                            // Do not process when first input is '!'.
                            let c1 = cmd.chars().nth(0).unwrap();
                            if !c1.is_ascii_uppercase() {
                                continue;
                            }
                            let col = game.get_col_from_char(&c);
                            let row = (c1 as u8 - 65) as usize;
                            if col != -1 && c1 >= 'A' && c1 <= 'Z' && row < game.level.rows {
                                game.rever_disp_col(col as u16);
                            }
                        }
                        _ => {}
                    }
                }
                game.stdout.flush().expect("Failed to flush output");
            }
            Event::Key(KeyEvent {
                code: KeyCode::Enter,
                kind,
                ..
            }) => {
                if kind == KeyEventKind::Release {
                    continue;
                };

                //game.stdout.flush().expect("Failed to flush output");

                if cmd.len() >= 2 {
                    // matched the cmd length
                    //如果命令长度已满足
                    let c_y = cmd.chars().nth(0).unwrap(); //Y坐标字母 / Row number    like ABCDEF...
                    let c_x = cmd.chars().nth(1).unwrap(); //X坐标字母 / Column number like ABCDEF...

                    let mut c_cmd = ' ';
                    if cmd.len() >= 3 {
                        c_cmd = cmd.chars().nth(2).unwrap(); //命令字符 / The cmd char,like D-Dig,F-Flag,P-Pending,T-Test
                    }
                    // confirm c_x,c_y is uppercase letter
                    if c_y >= 'A' && c_y <= 'Z' && c_x >= 'A' && c_x <= 'Z' {
                        let col = c_x as usize - 65; // 65 is the char 'A'
                        let row = c_y as usize - 65;
                        // 确保未超最大行列 / Ensure row and column input is below the most table index.
                        if col < game.level.cols
                            && row < game.level.rows
                            && !game_is_pause_or_finished
                        {
                            game.dig_cell(&col, &row, &c_cmd); //挖开此单元格 / Begin dig cell
                            if game.status == GameStatus::NotStart && (c_cmd == 'D' || c_cmd == ' ')
                            {
                                //如果是第一个单元格，开始计时 / if the first cmd ,start timer.
                                //开始计时 / Start timer
                                game.status = GameStatus::Started;
                                ch_sender.send(TimerStatus::Start).unwrap();
                            }
                        }
                    }
                    // !Command ,like !N=New game ,!Q=Quit,!P=Pause
                    if c_y == '!' {
                        match c_x {
                            //退出程序 / Quit program
                            'Q' => {
                                //execute!(std::io::stdout(), Show).unwrap();
                                //disable_raw_mode().expect("Failed to enable raw mode");
                                //exit(0);
                                break;
                            }
                            //暂停游戏 / Pause
                            'P' => {
                                if game.status == GameStatus::Started {
                                    game.pause();
                                    ch_sender.send(TimerStatus::Pause).unwrap();
                                }
                            }
                            //继续游戏 / Resume the game
                            'R' => {
                                if game.status == GameStatus::Paused {
                                    game.resume();
                                    ch_sender.send(TimerStatus::Resume).unwrap();
                                }
                            }
                            //新开游戏 / New game with current difficulty
                            'N' => {
                                let lv = game.level.level;
                                game = Game::new_game(lv);
                                //(x, y) = game.get_cmd_pos();
                                cmd.clear();

                                // Stop the timer
                                ch_sender.send(TimerStatus::Stop).unwrap();
                            }
                            // Check error
                            'C' => {
                                game.display_err();
                            }
                            // 换游戏难度 / Change difficulty
                            'D' => {
                                // stop the timer first
                                ch_sender.send(TimerStatus::Stop).unwrap();
                                // 换难度 / Change difficulty
                                game = Game::new_game(0);
                                cmd.clear();

                                // because thanged the difficulty,the display size has also been changed automaticly.
                                // so must inform the timer the new size.

                                // get the new position that the timer will use.
                                (x_t, y_t) = game.get_stat_time_pos();
                                // get the lock of shared_var
                                let mut sh_pos = shared_var.lock().unwrap();
                                // modify the values of shared_var to inform the timer that game size has changed
                                sh_pos.time_pos_x = x_t;
                                sh_pos.time_pos_y = y_t;
                                // must release the lock of shared_var
                                drop(sh_pos);
                            }
                            _ => {}
                        }
                    }
                }
                // cancel reversed row and column
                if !game_is_pause_or_finished {
                    let (row, col) = game.get_row_col_from_str(&cmd);
                    if row != -1 {
                        game.cancel_rever_row(row as u16);
                    }
                    if col != -1 {
                        game.cancel_rever_col(col as u16);
                    }
                }
                cmd.clear();
            }
            Event::Key(KeyEvent {
                code: KeyCode::Backspace,
                kind,
                ..
            }) => {
                //print!("\u{8}\u{8}");//退格
                if kind == KeyEventKind::Release {
                    continue;
                };

                // deal with the reverse display
                if cmd.len() > 0 && !game_is_pause_or_finished {
                    let c1 = cmd.chars().nth(0).unwrap();
                    // Only process if the first input is a letter,
                    // meaning if the first input is a row number.
                    // Do not process when first input is '!'.
                    if c1 >= 'A' && c1 <= 'Z' {
                        match cmd.len() {
                            // if len==1,cancel row reverse displaying
                            1 => {
                                let row = game.get_row_from_char(&c1);
                                if row != -1 {
                                    game.cancel_rever_row(row as u16);
                                }
                            }
                            // if len==2,cancel col reverse displaying
                            2 => {
                                let c2 = cmd.chars().nth(1).unwrap();
                                let col = game.get_col_from_char(&c2);
                                let row = game.get_row_from_char(&c1);
                                if col != -1 {
                                    game.cancel_rever_col(col as u16);
                                    // re-display the row.otherwise there is a cell not reversed after canceling the column reverse.
                                    if row != -1 {
                                        game.rever_disp_row(row as u16);
                                    }
                                }
                            }
                            // otherwise do nothing
                            _ => {}
                        }
                    }
                }

                cmd.pop(); //删除最后一个字符 / delete the last char of cmd string
                game.echo_cmd(&cmd);

                game.stdout.flush().expect("Failed to flush output");
            }
            Event::Key(KeyEvent {
                code: KeyCode::Esc, ..
            }) => {
                // cancel reverse
                if !game_is_pause_or_finished {
                    let (row, col) = game.get_row_col_from_str(&cmd);
                    if row != -1 {
                        game.cancel_rever_row(row as u16);
                    }
                    if col != -1 {
                        game.cancel_rever_col(col as u16);
                    }
                }
                cmd.clear();
            }
            //鼠标左击事件 / mouse left click
            Event::Mouse(MouseEvent {
                kind: MouseEventKind::Down(MouseButton::Left),
                column,
                row,
                ..
            }) => {
                let (y, x) = game.pos_to_index(row, column);
                if y >= 0 && x >= 0 && !game_is_pause_or_finished {
                    //如果是第一个单元格，开始计时 / if the first cmd ,start timer.
                    if game.status == GameStatus::NotStart {
                        //开始计时 / Start timer
                        game.status = GameStatus::Started;
                        ch_sender.send(TimerStatus::Start).unwrap();
                    }
                    // Left click = ' ' = Dig or Test
                    game.dig_cell(&(x as usize), &(y as usize), &' ');
                }
            }
            // 处理鼠标右击事件 / mouse right click
            Event::Mouse(MouseEvent {
                kind: MouseEventKind::Down(MouseButton::Right),
                column,
                row,
                ..
            }) => {
                let (y, x) = game.pos_to_index(row, column);
                if y >= 0 && x >= 0 && !game_is_pause_or_finished {
                    // Right click=Flag
                    game.dig_cell(&(x as usize), &(y as usize), &'F');
                }
            }
            _ => {} //game.update_time_consuming(),
        }

        // Every loop check if success
        // Success or Failed ,Pause the timer
        if game.success_check() != GameResult::NotOver {
            ch_sender.send(TimerStatus::Pause).unwrap();
            game.status = GameStatus::Finished;
        }
    }
    // 显示光标 / Show cursor
    execute!(std::io::stdout(), Show).unwrap();
    // 关闭raw mode / disable raw mode
    disable_raw_mode().expect("Failed to enable raw mode");
    execute!(std::io::stdout(), DisableMouseCapture)?;
    let (_, y) = game.get_cmd_pos();
    game.move_to(0, y + 2);
    //clear_screen();
    Ok(())
}
