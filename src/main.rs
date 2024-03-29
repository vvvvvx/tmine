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
    process::exit,
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

fn process_key_char(game: &mut Game, cmd: &mut String, c: &char) {
    // Can not input more than 3 chars when cell operation.
    // Can not input more than 2 chars when program operation

    if !cmd.is_empty() {
        let fc = cmd.chars().next().unwrap();
        if fc == '!' && cmd.len() >= 2 {
            return;
        }
        if cmd.len() >= 3 {
            return;
        }
    }

    *cmd += c.to_ascii_uppercase().to_string().as_str();
    game.echo_cmd(&cmd);

    let c1 = cmd.chars().next().unwrap();

    let game_is_pause_or_finished =
        game.status == GameStatus::Paused || game.status == GameStatus::Finished;

    if !game_is_pause_or_finished {
        match cmd.len() {
            // reverse display row
            1 => {
                // Only process if the first input is a letter,
                // meaning if the first input is a row number.
                // Do not process when first input is '!'.
                if c1.is_ascii_uppercase() {
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
                let c1 = cmd.chars().next().unwrap();

                if !c1.is_ascii_uppercase() {
                    return;
                }

                let col = game.get_col_from_char(&c);
                let row = (c1 as u8 - 65) as usize;
                if col != -1 && c1.is_ascii_uppercase() && row < game.level.rows {
                    game.rever_disp_col(col as u16);
                }
            }
            _ => {}
        }
    }
    game.stdout.flush().expect("Failed to flush output");
}

fn process_key_enter(
    game: &mut Game,
    cmd: &mut String,
    shared_var: &Arc<Mutex<SharePos>>,
    ch_sender: &Sender<TimerStatus>,
) -> Result<(), std::io::Error> {
    let game_is_pause_or_finished =
        game.status == GameStatus::Paused || game.status == GameStatus::Finished;

    if cmd.len() >= 2 {
        // matched the cmd length
        // Row number    like ABCDEF...

        if cmd.chars().next().is_none() {
            cmd.clear();
            game.echo_cmd(&cmd);
            return Ok(());
        }
        // 处理中文输入法下不正常输入
        if cmd.chars().nth(1).is_none() {
            cmd.clear();
            game.echo_cmd(&cmd);
            return Ok(());
        }

        let c_y = cmd.chars().nth(0).unwrap(); // Row number like ABCDEF...
        let c_x = cmd.chars().nth(1).unwrap(); // Column number like ABCDEF...

        let mut c_cmd = ' ';
        if cmd.len() >= 3 {
            c_cmd = cmd.chars().nth(2).unwrap(); // The cmd char,like D-Dig,F-Flag,P-Pending,T-Test
        }
        // confirm c_x,c_y is uppercase letter
        if c_y.is_ascii_uppercase() && c_x.is_ascii_uppercase() {
            let col = c_x as usize - 65; // 65 is the char 'A'
            let row = c_y as usize - 65;
            //  Ensure row and column input is below the most table index.
            if col < game.level.cols && row < game.level.rows && !game_is_pause_or_finished {
                game.dig_cell(&col, &row, &c_cmd); // Begin dig cell
                if game.status == GameStatus::NotStart && (c_cmd == 'D' || c_cmd == ' ') {
                    // if the first cmd ,start timer.
                    game.status = GameStatus::Started;
                    ch_sender.send(TimerStatus::Start).unwrap();
                }
            }
        }
        // !Command ,like !N=New game ,!Q=Quit,!P=Pause
        if c_y == '!' {
            match c_x {
                // Quit program
                'Q' => {
                    execute!(std::io::stdout(), Show).unwrap();
                    //break;
                    //  Show cursor
                    execute!(std::io::stdout(), Show).unwrap();
                    //  disable raw mode
                    disable_raw_mode().expect("Failed to enable raw mode");
                    execute!(std::io::stdout(), DisableMouseCapture)?;
                    let (_, y) = game.get_cmd_pos();
                    game.move_to(0, y + 2);

                    exit(0);
                }
                // Pause
                'P' => {
                    if game.status == GameStatus::Started {
                        game.pause();
                        ch_sender.send(TimerStatus::Pause).unwrap();
                    }
                }
                // Resume the game
                'R' => {
                    if game.status == GameStatus::Paused {
                        game.resume();
                        ch_sender.send(TimerStatus::Resume).unwrap();
                    }
                }
                // New game with current difficulty
                'N' => {
                    let lv = game.level.level;
                    *game = Game::new_game(lv);
                    cmd.clear();

                    // Stop the timer
                    ch_sender.send(TimerStatus::Stop).unwrap();
                }
                // Check error
                'C' => {
                    game.display_err();
                }
                //  Change difficulty
                'D' => {
                    // stop the timer first
                    ch_sender.send(TimerStatus::Stop).unwrap();
                    execute!(std::io::stdout(), DisableMouseCapture)?;
                    //  Change difficulty
                    *game = Game::new_game(0);
                    execute!(std::io::stdout(), EnableMouseCapture)?;
                    cmd.clear();

                    // because thanged the difficulty,the display size has also been changed automaticly.
                    // so must inform the timer the new size.

                    // get the new position that the timer will use.
                    let (x_t, y_t) = game.get_stat_time_pos();
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
    Ok(())
}
fn process_key_backspace(game: &mut Game, cmd: &mut String) {
    let game_is_pause_or_finished =
        game.status == GameStatus::Paused || game.status == GameStatus::Finished;
    // deal with the reverse display
    if !cmd.is_empty() && !game_is_pause_or_finished {
        let c1 = cmd.chars().next().unwrap();
        // Only process if the first input is a letter,
        // meaning if the first input is a row number.
        // Do not process when first input is '!'.
        if c1.is_ascii_uppercase() {
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

fn process_mouse_left(
    game: &mut Game,
    cmd: &mut String,
    shared_var: &Arc<Mutex<SharePos>>,
    ch_sender: &Sender<TimerStatus>,
    column: &u16,
    row: &u16,
) -> Result<(), std::io::Error> {
    let game_is_pause_or_finished =
        game.status == GameStatus::Paused || game.status == GameStatus::Finished;
    let (y, x) = game.pos_to_index(*row, *column);
    if y >= 0 && x >= 0 && !game_is_pause_or_finished {
        // if the first cmd ,start timer.
        if game.status == GameStatus::NotStart {
            // Start timer
            game.status = GameStatus::Started;
            ch_sender.send(TimerStatus::Start).unwrap();
        }
        // Left click = ' ' = Dig or Test
        game.dig_cell(&(x as usize), &(y as usize), &' ');

        return Ok(());
    }
    // 是否点击了“程序命令”
    match game.which_cmd_clicked(*column, *row) {
        'Q' => {
            execute!(std::io::stdout(), Show).unwrap();
            //  Show cursor
            execute!(std::io::stdout(), Show).unwrap();
            //  disable raw mode
            disable_raw_mode().expect("Failed to enable raw mode");
            execute!(std::io::stdout(), DisableMouseCapture)?;
            let (_, y) = game.get_cmd_pos();
            game.move_to(0, y + 2);
            exit(0);
        }
        // Pause
        'P' => {
            if game.status == GameStatus::Started {
                game.pause();
                ch_sender.send(TimerStatus::Pause).unwrap();
            }
        }
        // Resume the game
        'R' => {
            if game.status == GameStatus::Paused {
                game.resume();
                ch_sender.send(TimerStatus::Resume).unwrap();
            }
        }
        // New game with current difficulty
        'N' => {
            let lv = game.level.level;
            *game = Game::new_game(lv);
            cmd.clear();

            // Stop the timer
            ch_sender.send(TimerStatus::Stop).unwrap();
        }
        //  Change difficulty
        'D' => {
            // stop the timer first
            ch_sender.send(TimerStatus::Stop).unwrap();
            execute!(std::io::stdout(), DisableMouseCapture)?;
            //  Change difficulty
            *game = Game::new_game(0);
            execute!(std::io::stdout(), EnableMouseCapture)?;
            cmd.clear();

            // because thanged the difficulty,the display size has also been changed automaticly.
            // so must inform the timer the new size.

            // get the new position that the timer will use.
            let (x_t, y_t) = game.get_stat_time_pos();
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
    Ok(())
}
fn main() -> io::Result<()> {
    let mut game = Game::new_game(0);
    let mut cmd: String = String::new();
    let update_interval = Duration::from_secs(1); //update time consuming interval.
                                                  // Hide cursor
    execute!(std::io::stdout(), Hide).unwrap();
    // enable raw mode to listen keyboard input.
    enable_raw_mode().expect("Failed to enable raw mode");
    execute!(std::io::stdout(), EnableMouseCapture)?;
    // Create a channel to control the timer by main thread.
    let (ch_sender, ch_receiver): (Sender<TimerStatus>, Receiver<TimerStatus>) = channel();

    // Get time consuming display postition
    let (x_t, y_t) = game.get_stat_time_pos();

    // Create a variable to share the position between the main thread and the timer thread.
    let shared_var = Arc::new(Mutex::new(SharePos {
        time_pos_x: x_t,
        time_pos_y: y_t,
    }));

    let shared_var_clone = shared_var.clone();

    // Create a timer
    let mut timer = Timer::new();

    // run a timer thread to update time consuming
    thread::spawn(move || -> ! {
        loop {
            // Get the command from main thread
            let mut status_t = TimerStatus::NotStart;
            if let Ok(new_status) = ch_receiver.try_recv() {
                status_t = new_status;
            };

            // 判断 timer status 变量的值,操作timer
            match status_t {
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
            // refresh time consuming
            let elapsed = timer.last.elapsed();
            if elapsed >= update_interval && timer.is_running {
                timer.last += update_interval;
                // use the data of shared_var
                timer.update_time_consuming(sh_pos.time_pos_x, sh_pos.time_pos_y);
            }
            //must release the lock of shared_var
            drop(sh_pos);
            // sleep to reduce the CPU consuming
            sleep(Duration::from_millis(100));
        }
    });

    // Main thread loop
    loop {
        //cursor to cmd input postion
        let ev = read().expect("Failed to read event");

        match ev {
            Event::Key(KeyEvent {
                code: KeyCode::Char(c),
                kind,
                ..
            }) => {
                // when press a key,will send Press and Release 2 events ,so one char will repeat
                // two times,to avoid it here only deal with the Press event, do nothing when key released.
                if kind == KeyEventKind::Release {
                    continue;
                };

                process_key_char(&mut game, &mut cmd, &c);
            }
            Event::Key(KeyEvent {
                code: KeyCode::Enter,
                kind,
                ..
            }) => {
                if kind == KeyEventKind::Release {
                    continue;
                };

                process_key_enter(&mut game, &mut cmd, &shared_var, &ch_sender)?;
            }
            Event::Key(KeyEvent {
                code: KeyCode::Backspace,
                kind,
                ..
            }) => {
                if kind == KeyEventKind::Release {
                    continue;
                };
                process_key_backspace(&mut game, &mut cmd);
            }
            Event::Key(KeyEvent {
                code: KeyCode::Esc, ..
            }) => {
                let game_is_pause_or_finished =
                    game.status == GameStatus::Paused || game.status == GameStatus::Finished;
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
            // mouse left click
            Event::Mouse(MouseEvent {
                kind: MouseEventKind::Down(MouseButton::Left),
                column,
                row,
                ..
            }) => {
                process_mouse_left(&mut game, &mut cmd, &shared_var, &ch_sender, &column, &row)?;
            }
            // mouse right click
            Event::Mouse(MouseEvent {
                kind: MouseEventKind::Down(MouseButton::Right),
                column,
                row,
                ..
            }) => {
                let game_is_pause_or_finished =
                    game.status == GameStatus::Paused || game.status == GameStatus::Finished;
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
    //  Show cursor
    //execute!(std::io::stdout(), Show).unwrap();
    //  disable raw mode
    //disable_raw_mode().expect("Failed to enable raw mode");
    //execute!(std::io::stdout(), DisableMouseCapture)?;
    //let (_, y) = game.get_cmd_pos();
    //game.move_to(0, y + 2);
    //clear_screen();
    //Ok(())
}
