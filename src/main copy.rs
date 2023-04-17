//--------TMine--------
//Begin time: 2023-04-07
//Author    : Viaco Xu
//--------TMine--------

//use std::io::Write;
use crossterm::{
	cursor,
	cursor::{Hide, Show},
	execute, terminal::{self, }, QueueableCommand,// Command,
};
use crossterm::{
	event::{read, Event, KeyCode, KeyEvent},
	terminal::{disable_raw_mode, enable_raw_mode,},
};
use rand::Rng;
//use core::time;
use std::io::{self};
use std::process::exit;
use std::vec;
//use crossterm::{cursor::{ MoveTo}, style::{Color, Print, ResetColor, SetBackgroundColor}};
use std::io::{stdout, Write};
use std::time::Instant;

struct Timer{
	start_time:Instant, 
	pause_duration:u64,
	pause_time:Instant,
	is_running:bool
}

impl Timer {
	fn new()->Timer{
		return Timer{
			start_time:Instant::now(),
			pause_duration:0,
			pause_time:Instant::now(),
			is_running:false
		}
	}

	pub fn start(&mut self){
		self.start_time=Instant::now();
		self.pause_duration=0;
		self.is_running=true;
	}
	
	pub fn pause(&mut self){
		self.pause_time=Instant::now();
		self.is_running=false;
	}
	
	pub fn resume(&mut self){
		self.pause_duration+=self.pause_time.elapsed().as_secs();
		self.is_running=true;
	}
	
	pub fn get_elapsed(&self)->u64{
		return if self.is_running {
			self.start_time.elapsed().as_secs()-self.pause_duration
		}else{
			self.pause_time.duration_since(self.start_time).as_secs()-self.pause_duration
		};

	}
}

#[derive(Clone)]
struct Level {
// Game difficulty Level

	rows: i16,   // game table rows
	cols: i16,   // game table columns
	mines: i16,  // this level's sum mines
	width: u16,  // screen width needed
	height: u16, // screen height needed
}

impl Level {
	pub fn new() -> Level {
		// width offset “20” is for the right area to display the help info, 
		// this is only for Chinese char's width.
		// If other language displayed abnormally, please modify it bigger
		let width_offset:u16=20; 
		let height_offset:u16=4;

		// Select difficult level to init Level / 选择难度级别
		let lv = select_level();
		match lv {
			1 => Level {
				rows: 8,
				cols: 10,
				mines: 7,
				width: 10 * 4 + 3 + width_offset,
				height: 8 * 2 + height_offset,
			},
			2 => Level {
				rows: 9,
				cols: 14,
				mines: 15,
				width: 14 * 4 + 3 + width_offset,
				height: 9 * 2 + height_offset,
			},
			3 => Level {
				rows: 15,
				cols: 20,
				mines: 40,
				width: 20 * 4 + 3 + width_offset,
				height: 15 * 2 + height_offset,
			},
			4 => Level {
				rows: 19,
				cols: 26,
				mines: 99,
				width: 26 * 4 + 3 + width_offset,
				height: 19 * 2 + height_offset,
			},
			_ => Level {
				rows: 8,
				cols: 10,
				mines: 7,
				width: 10 * 4 + 3 + width_offset,
				height: 8 * 2 + height_offset,
			},
		}
	}
}

#[derive(Clone, PartialEq)]
enum Status {
	Opened,     // 已开/have opened
	Flaged,     // 已标记雷/Flaged mine
	Unexplored, // 未探明 /Have not explored
	Pending,    // 未决 / Pending
}

#[derive(Clone)]
struct Cell {
// Mine cell
	is_mine: bool,    // 是否有雷   / Is it self a mine 
	surrnd_mines: i8, // 周围雷数   / Sum of surrounding mines
	status: Status,   // 单元格状态 / Cell status
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

//#[derive(Clone)]
pub struct Game {
	level: Level,               // Game difficult level
	mine_table: Vec<Vec<Cell>>, // mine table array
	timer: Timer,               // to calculate time consuming
	mines_left: i16, 
}

impl Game {
	pub fn new() -> Game {
		//level==1 新手/ Beginner 
		//level==2 初级/ Basic
		//level==3 中级/ Intermediate
		//level==4 高级/ Advanced
		let lv = Level::new();

		Game {
			level: lv.clone(),
			mine_table: (vec![vec![Cell::new(); lv.cols as usize]; lv.rows as usize]),
			timer: Timer::new(),
			mines_left: (lv.mines),
		}
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
			let x = get_rand(self.level.cols);
			let y = get_rand(self.level.rows);
			//布雷
			if self.mine_table[y as usize][x as usize].is_mine == false {
				self.mine_table[y as usize][x as usize].is_mine = true;
				i -= 1;
			}
		}
	}

	// Refresh the mine table of terminal UI
	fn display_refresh(&mut self) {
		for i in 0..self.mine_table.len() {
			for j in 0..self.mine_table[i].len() {
				refresh_cell(&j, &i, &mut self.mine_table[i][j])
			}
		}
	}

	// Display all mines in terminal UI
	fn display_mine(&mut self) {
		for i in 0..self.mine_table.len() {
			for j in 0..self.mine_table[i].len() {
				//refresh_cell(&j, &i, &mut self.mine_table[i][j])
				if self.mine_table[i][j].status == Status::Opened
					|| (self.mine_table[i][j].status == Status::Flaged
						&& self.mine_table[i][j].is_mine)
				{
					//如果已经打开了，则不管。

					continue;
				}
				if self.mine_table[i][j].is_mine == true {
					let mut stdout = io::stdout();
					let (x, y) = locate_from_table_col_row(j as u16, i as u16);
					stdout.queue(cursor::MoveTo(x, y)).unwrap();
					print!("\x1B[31mM\x1B[0m");
					stdout.flush().unwrap();
				}
			}
		}
	}
	//计算余雷统计信息打印位置 / get the UI position of "Mine left" info to print
	fn get_stat_mine_pos(&self) -> (u16, u16) {
		return ((4 * self.level.cols + 6) as u16, 0);
	}
	//计算耗时显示信息打印位置 / get the UI position of "Time consuming" info to print
	fn get_stat_time_pos(&self) -> (u16, u16) {
		return ((4 * self.level.cols + 6) as u16, 1);
	}
	//计算帮助信息打印位置 /get the UI position of "Help" info to print
	fn get_help_pos(&self) -> (u16, u16) {
		return ((4 * self.level.cols + 6) as u16, 4);
	}
	//计算命令提示信息打印位置 / get the  UI position of cmd to output
	fn get_cmd_pos(&self) -> (u16, u16) {
		return ( (4 * self.level.cols + 6) as u16,((self.level.rows + 1) * 2) as u16, );
	}

	// 获取屏幕表格中间位置，用于显示游戏结果信息 
	// Get the center position of the table of UI. for displaying result info of game.
	fn get_table_mid_pos(&self)->(u16,u16){
		return ( ((4 * self.level.cols+3)/2) as u16,(self.level.rows) as u16);
	}
	//计算周围雷数 / calculate the surrounding mines of cur cell
	fn calc_surrnd_mines(&mut self) {
		for i in 0..self.mine_table.len() {
			for j in 0..self.mine_table[i].len() {
				if self.mine_table[i][j].is_mine == false {
					self.mine_table[i][j].surrnd_mines = calc_mines(&j, &i, &self.mine_table);
				}
			}
		}
	}
	//失败，画面爆炸 / display infos of game failed
	fn failed(&mut self) {
		self.display_mine();
		let (x,y)=self.get_table_mid_pos();
		let mut stdout = io::stdout();
		stdout.queue(cursor::MoveTo(x -4, y-1)).unwrap();
		print!("\n\x1B[31m\x1B[5m\x1B[1m 您失败了！ \x1B[0m");
		stdout.queue(cursor::MoveTo(x -4, y)).unwrap();
		print!("\n\x1B[31m\x1B[5m\x1B[1mYou failed!\x1B[0m");

		stdout.flush().unwrap();
	}

	
   
   //Display error flaged cell
	fn display_err(&self){
		for i in 0..self.mine_table.len() {
			for j in 0..self.mine_table[i].len() {
				if self.mine_table[i][j].status==Status::Flaged && self.mine_table[i][j].is_mine == false {
					let (x,y)=locate_from_table_col_row(j as u16, i as u16); 
					let mut stdout = io::stdout();
					stdout.queue(cursor::MoveTo(x , y)).unwrap();
					print!("\x1B[31m\x1B[43m F \x1B[0m");
					stdout.flush().unwrap();
					}
			}
		}
	}
/*
	 // 检查flag是否正确
	fn check_flag_correct(&self)->bool{
		for i in 0..self.mine_table.len() {
			for j in 0..self.mine_table[i].len() {
				if self.mine_table[i][j].status==Status::Flaged && self.mine_table[i][j].is_mine == false {
					return false;
				}
			}
		}
		return true;
	}
	// 检查是否全部翻完
	fn check_all_flaged(&self)->bool{
		for i in 0..self.mine_table.len() {
			for j in 0..self.mine_table[i].len() {
				if self.mine_table[i][j].status==Status::Unexplored  {
					return false;
				}
			}
		}
		return true;
	}
*/
	// 检查是否胜利结束 / Check if the game has been successfully ended
	fn success_check(&mut self){
		// Success conditions:
		// 1. 地雷数==0     / mines_left==0 
		// 2. 无未探明的cell / no unexplored cell
		// 3. 无错误的雷标记  / no wrong flaged cell
		for i in 0..self.mine_table.len() {
			for j in 0..self.mine_table[i].len() {
				if self.mine_table[i][j].status==Status::Unexplored ||
				(self.mine_table[i][j].status==Status::Flaged && self.mine_table[i][j].is_mine == false)
				{
					return ;
				}
			}
		}

		if self.mines_left==0 {
			//打印胜利信息 / print success info
			self.timer.pause();
			self.update_mine_left_disp();
			let (x,y)=self.get_table_mid_pos();
			let mut stdout = io::stdout();
			stdout.queue(cursor::MoveTo(x -4, y-1)).unwrap();
			print!("\n\x1B[32m\x1B[5m\x1B[1m您胜利了!\n\x1B[0m");
			stdout.queue(cursor::MoveTo(x -4, y)).unwrap();
			print!("\n\x1B[32m\x1B[5m\x1B[1mYou won !\n\x1B[0m");
			stdout.flush().unwrap();
		}        
		//显示所用时间 / refresh time consuming info to UI
		self.update_time_consuming();
	}
	
	// 翻开单元格 / Dig cell function
	// x、y为mine_arr数组index坐标 
	// x y is the mine array's index
	fn dig_cell(&mut self, x: &usize, y: &usize, cmd: &char) {
		//let  m: &mut Cell=&mut (self.mine_table[*y][*x]);
		match self.mine_table[*y][*x].status {
			//根据单元格状态 / depend on cell's status
			Status::Unexplored => {//未开状态
				match cmd {
					// 标记命令 / Flag cmd
					'F' => { 
						self.mine_table[*y][*x].status = Status::Flaged;
						self.mines_left -= 1; //剩余雷减1
						refresh_cell(x, y, &self.mine_table[*y][*x]);
						self.update_mine_left_disp(); //更新余雷数量显示
						if self.mines_left==0 {
							self.success_check();
						}
					}
					//存疑标记 / Pending cmd
					'P' => { 
						self.mine_table[*y][*x].status = Status::Pending;
						refresh_cell(x, y, &self.mine_table[*y][*x]);
					}
					//挖开命令 / Dig cmd
					'D' | ' ' | '\n' => { 
						//D、Space、Enter
						self.mine_table[*y][*x].status = Status::Opened; //修改会未开
																		 //修改单元格界面
						refresh_cell(x, y, &self.mine_table[*y][*x]);
						if self.mine_table[*y][*x].is_mine {
							self.failed(); //触发爆炸，程序结束 /if digged mine ,game over
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
									if self.mine_table[yy as usize][xx as usize].status== Status::Unexplored
										&& !(xx == *x && yy == *y)
									{
										self.dig_cell(&(xx as usize), &(yy as usize), &'D');
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
								if self.mine_table[yy as usize][xx as usize].status
									== Status::Flaged
								{
									sum_mines += 1;
								}
							}
						}
						// 如果标记的数量等于单元格总雷数，则把除标记之外的单元格都打开
						// if cell's surrounding mines== cell.surrnd_mine, dig open the left unopened cells surrounding.
						if self.mine_table[*y][*x].surrnd_mines == sum_mines {
							for yy in *min_y as usize..*max_y {
								for xx in *min_x as usize..*max_x {
									if self.mine_table[yy as usize][xx as usize].status
										!= Status::Flaged
										&& !(xx == *x && yy == *y)
									{
										if self.mine_table[yy as usize][xx as usize].surrnd_mines > 0
										{
											self.mine_table[yy as usize][xx as usize].status = Status::Opened;
											refresh_cell(&xx, &yy, &self.mine_table[yy][xx]);
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
						self.mines_left+=1;
						refresh_cell(x, y, &self.mine_table[*y][*x]);
					}
					// Pending cmd 
					'P' => {
						// 重新标记为Pending  / been flaged ,now re-mark it as pending
						self.mine_table[*y][*x].status = Status::Pending;
						refresh_cell(x, y, &self.mine_table[*y][*x]);
					}
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
						refresh_cell(x, y, &self.mine_table[*y][*x]);
					}
					_ => {}
				}
			}
			//_=>{}
		}
		self.success_check();
	}

	// Pause game
	fn pause(&mut self) {
		self.update_time_consuming();
		self.timer.pause();
		let mut stdout = io::stdout();
		for i in 0..self.mine_table.len() {
			for j in 0..self.mine_table[i].len() {
				let (x, y) = locate_from_table_col_row(j as u16, i as u16);
				stdout.queue(cursor::MoveTo(x-1, y)).unwrap();
				print!("   ");
				stdout.flush().unwrap();
			}
			
		}
	}
	// Restore the paused game
	fn restore(&mut self){
		self.display_refresh();
		self.update_time_consuming();
		self.timer.resume();
	}
	//绘制界面 / Draw mine table on terminal UI
	fn draw_ui(&self)  {
		let row = self.level.rows;
		let col = self.level.cols;
		// 获取标准输出流
		let mut stdout = io::stdout();

		// 清屏 / clear screen
		stdout.queue(terminal::Clear(terminal::ClearType::All)).unwrap();
		stdout.queue(cursor::MoveTo(0, 0)).unwrap();
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
		stdout.write_all(first_line.as_bytes()).unwrap();
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
			//jj_line+="\n";
			let c = 65 + i;
			print!("{}", (c as u8) as char); //打印左侧行序号 / Print row numbers on the left ,like ABCDEFG...
			stdout.write_all(jj_line.as_bytes()).unwrap();
			print!("{}\n", (c as u8) as char); //打印右侧行序号 / Print row numbers on the right ,like ABCDEF....
			stdout.write_all(mid_line.as_bytes()).unwrap();
		}
		//打印底行 / Print bottom line
		let mut bot_line: String = String::from(" └");
		for _i in 0..col - 1 {
			bot_line += "───┴";
		}
		bot_line += "───┘\n";
		let c = 65 + row - 1;
		print!("{}", (c as u8) as char); //打印最后一行左侧行序号 / Print the last line's row number on the left
		stdout.write_all(jj_line.as_bytes()).unwrap();
		print!("{}\n", (c as u8) as char); //打印最后一行右侧行序号 / Print the last line's row number on the right
		stdout.write_all(bot_line.as_bytes()).unwrap();
		//打印底边列号 / Print the column nubmer on bottom.
		print!("   {}", (65) as char);
		for i in 0..col - 1 {
			let c = 66 + i;
			print!("   {}", (c as u8) as char);
		}
		print!("\n");

		//打印提示信息 / Print statistics info
		let (x, y) = self.get_stat_mine_pos();
		//stdout.queue(cursor::MoveTo(x, y - 1)).unwrap();
		//print!("─────Stat─────");      //──Statistics──
		stdout.queue(cursor::MoveTo(x, y)).unwrap();
		print!("余雷Mine: {}", self.level.mines); //Mines left
		let (x1, y1) = self.get_stat_time_pos();
		stdout.queue(cursor::MoveTo(x1, y1)).unwrap();
		print!("耗时Time:");                     //Time used
		let (x2, y2) = self.get_cmd_pos();
		//stdout.queue(cursor::MoveTo(x2, y2 - 1)).unwrap();
		//print!("───── CMD ─────");      //──Command──
		stdout.queue(cursor::MoveTo(x2, y2)).unwrap();
		print!("Input:");
		// 打印帮助信息
		let (x3, y3) = self.get_help_pos();
		stdout.queue(cursor::MoveTo(x3, y3-1)).unwrap();
		print!("─────Help─────");       //──Help──
		stdout.queue(cursor::MoveTo(x3, y3 )).unwrap();
		print!("操作:行+列+命令");          // print!("CMD:Row+Col+Cmd")
		stdout.queue(cursor::MoveTo(x3, y3 + 1)).unwrap();
		print!("Oper:Row+Col+CMD");          // print!("CMD:Row+Col+Cmd")
		stdout.queue(cursor::MoveTo(x3, y3 + 2)).unwrap();
		print!("───命令|CMD───");          // print!("CMD:Row+Col+Cmd")

		stdout.queue(cursor::MoveTo(x3, y3 + 3)).unwrap();
		print!("F-Flag    标记");
		stdout.queue(cursor::MoveTo(x3, y3 + 4)).unwrap();
		print!("D-Dig     翻开");
		stdout.queue(cursor::MoveTo(x3, y3 + 5)).unwrap();
		print!("T-Test    测试");
		stdout.queue(cursor::MoveTo(x3, y3 + 6)).unwrap();
		print!("P-Pending 疑问");
		stdout.queue(cursor::MoveTo(x3, y3 + 8)).unwrap();
		print!("!Q-Quit   退出");
		stdout.queue(cursor::MoveTo(x3, y3 + 9)).unwrap();
		print!("!P-Pause  暂停");
		stdout.queue(cursor::MoveTo(x3, y3 + 10)).unwrap();
		print!("!R-Restore恢复");
		stdout.queue(cursor::MoveTo(x3, y3 + 11)).unwrap();
		print!("!N-New    重玩");
		stdout.flush().unwrap();

	}
    
    //更新余雷数量 / refresh mine left info to UI
	fn update_mine_left_disp(&self) {
		let (x, y) = self.get_stat_mine_pos();
		let mut stdout = io::stdout();
		stdout.queue(cursor::MoveTo(x + 10, y)).unwrap();
		print!("{} ", self.mines_left);
		stdout.flush().unwrap();
	}
	// update time comsuming info to UI
	fn update_time_consuming(&mut self){
		
		//更新耗时显示
		let (x,y)=self.get_stat_time_pos();
		let mut stdout = stdout();

		stdout.queue(cursor::MoveTo(x + 10, y)).unwrap();
		print!("{}s",self.timer.get_elapsed());
		stdout.flush().expect("Failed to flush output");
	}
} //impl Game ended



//////////////////////Global functions below//////////////////////
fn new_game() -> Game {
	clear_screen();

	disable_raw_mode().expect("Failed to enable raw mode");
	let mut game = Game::new();

	// 获取窗口大小 / Get the teminal windows size
	// let (width, height) = match terminal::size() {
	//     Ok((w, h)) => (w, h),
	//     Err(e) => {
	//         println!("Get terminal size error: {:?}", e);
	//         (0, 0)
	//     }
	// };

	//调整窗口大小，以符合该游戏级别的窗口尺寸要求 / Adjust the terminal windows size to fit the game level
	print!("\x1B[8;{};{}t", game.level.height, game.level.width);
	stdout().flush().unwrap();

	// Initialize the game
	game.laying_mine();
	game.draw_ui();
	game.calc_surrnd_mines();
	enable_raw_mode().expect("Failed to enable raw mode");
	return game;
}


/*
fn reverse_display_row(row:&u8,level:&Level){
	let r=row-65;//根据字母ASC码计算行号
	let (_,y)=locate(0, r as u16);//定位行位置

	let mut stdout = stdout();
	execute!(stdout, MoveTo(1, y)).unwrap();
	// 获取当前光标位置
	//let (x, y) = position().unwrap();
	// 将光标移动到光标所在位置
	//execute!(stdout, MoveTo(x, y)).unwrap();
	// 获取屏幕上该位置的字符

	let mut buffer: [u8; 1] = [0; 1];
   // let _ = stdout.read(&mut buffer);
	// let _ = (&mut stdout).read(&mut buffer);
	// let mut stdout = stdout.into_raw_mode().unwrap(); // 转换为原始模式
	// 获取屏幕上该位置的字符
	let mut buffer: [u8; 1] = [0; 1];
	//let _ = (&mut stdout).read(&mut buffer);
	let c = std::char::from_u32(buffer[0] as u32).unwrap();
	let c = std::char::from_u32(buffer[0] as u32).unwrap();
	// 反转颜色并打印字符
	execute!(stdout, SetBackgroundColor(Color::Black), Print(c), ResetColor).unwrap();
}
*/
/* ASCI转移字符
红色：\x1B[31m
绿色：\x1B[32m
黄色：\x1B[33m
蓝色：\x1B[34m
洋红色：\x1B[35m
青色：\x1B[36m
92淡绿色

背景：

	40：黑色
	41：红色
	42：绿色
	43：黄色
	44：蓝色
	45：洋红色
	46：青色
	47：白色

绿色背景：\x1b[42m
重置：\x1B[0m
*/

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
//刷新单元格内容 / Refresh cell display when cell's status changed
fn refresh_cell(x: &usize, y: &usize, cell: &Cell) {
	let mut stdout = io::stdout();
	let (x1, y1) = locate_from_table_col_row(*x as u16, *y as u16);
	stdout.queue(cursor::MoveTo(x1 - 1, y1)).unwrap();
	//cursor::Hide;
	let c = cell;
	match c.status {
		Status::Opened => {
			if !c.is_mine { //如果不是雷 / if the cell is not mine.
				
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
			print!("\x1B[31m\x1B[100m F \x1B[0m");
		}
		Status::Pending => {
			// 如果存疑，显示蓝色问号
			// if Pending ,dispaly '?' with gray BG and Blue font
			print!("\x1B[34m\x1B[100m ? \x1B[0m");
		}
		Status::Unexplored => { 
			// unexplored ,with no color
			print!("   ");
		}
		//_=>{ }
	}
	stdout.flush().unwrap();

}

// 计算（x,y）单元格周围的雷数
// Calculte surrounding mines of the cell (x,y) 
fn calc_mines(x: &usize, y: &usize, mine_arr: &Vec<Vec<Cell>>) -> i8 {
	let max_x = mine_arr[0].len();
	let max_y = mine_arr.len();

	let min_x: usize = if *x == 0 { 0 } else { x - 1 };
	let min_y: usize = if *y == 0 { 0 } else { y - 1 };

	let mut sum: i8 = 0;
	for i in min_y..y + 2 {
		for j in min_x..x + 2 {
			if !(i == *y && j == *x) && i < max_y && j < max_x && mine_arr[i][j].is_mine == true {
				sum += 1;
			}
		}
	}
	return sum;
}

//产生随机数
fn get_rand(range: i16) -> i16 {
	let mut rng = rand::thread_rng();
	return rng.gen_range(0, range);
}
//处理数字输入
// deal keyboard input
fn input() -> i16 {
	let mut input = String::new();
	io::stdin().read_line(&mut input).unwrap();
	let num: i16 = match input.trim().parse::<i16>() {
		Ok(n) => n,
		Err(_) => {
			println!("\x1B[31m\x1B[1m输入错误，请输入数字。\x1B[0m");
			return -1;
		}
	};
	return num;
}

fn select_level() -> i16 {
	//level==1 新手/Beginner 
	//level==2 初级/Basic
	//level==3 中级/Intermediate
	//level==4 高级/Advanced

	print!("\n请选择难度级别|Select difficulty:\n\n   1--新手|Beginner      8x10  [  7 雷|Mines ]\n   2--初级|Basic         9x14  [ 15 雷|Mines ]\n   3--中级|Intermediate 15x20  [ 40 雷|Mines ]\n   4--高级|Advanced     19x26  [ 99 雷|Mines ]\n\n   0--退出|Quit\n\n请选择|Your choice:");
	// print!("\nPlease select the difficult level:\n   1--Beginner 8x10 [  7Mines ]\n   2--Basic....")

	if io::stdout().flush().is_err() {
		println!("flush err");
	};
	let mut num = input();
	while num < 0 || num > 4 {
		if num > 4 {
			println!("请输入0-4以内的数，以确定难度级别");
			//println!("Please input number between 0-4");
		}
		num = input();
	}
	if num == 0 {
		exit(0)
	}
	return num;
}

// 根据行列定位表格坐标，提供给cursor::MoveTo()使用
// base on table column and row number to locate the screen position,provide to cursor::MoveTo()
pub fn locate_from_table_col_row(col: u16, row: u16) -> (u16, u16) {
	return ((3 + 4 * col), (2 + row * 2));
}


fn main() -> io::Result<()> {

	let mut stdout = stdout();
	let mut game = new_game();
	let mut cmd: String = String::new();
	let (mut x, mut y) = game.get_cmd_pos();
	let mut c_count: i8 = 0;
	let mut game_started=false; //是否开始游戏计时，当打开第一个单元格时开始计时。
									  //timer switch，start it when the first open command sent
	//隐藏光标 / Hide cursor
	execute!(std::io::stdout(), Hide).unwrap();
	//开启raw mode,监听键盘输入 / enable raw mode to listen keyboard input.
	enable_raw_mode().expect("Failed to enable raw mode");
	loop {
		stdout.queue(cursor::MoveTo(x + 7, y)).unwrap();

		match read().expect("Failed to read event") {
			Event::Key(KeyEvent { code: KeyCode::Char(c), .. }) => {
				c_count += 1;
				cmd += c.to_ascii_uppercase().to_string().as_str();
				print!("{} ", cmd);
				stdout.flush().expect("Failed to flush output");
				game.update_time_consuming();
			}
			Event::Key(KeyEvent { code: KeyCode::Enter, .. }) => {
				stdout.flush().expect("Failed to flush output");
				if cmd == "!EXIT" || cmd == "!Q" || cmd == "!QUIT" {
					break;
				}
				if c_count >= 2 { // matched the cmd length
					//如果命令长度已满足 
					let c_y = cmd.chars().nth(0).unwrap(); //Y坐标字母 / Row number    like ABCDEF...
					let c_x = cmd.chars().nth(1).unwrap(); //X坐标字母 / Column number like ABCDEF...

					let mut c_cmd = ' ';
					if c_count >= 3 {
						c_cmd = cmd.chars().nth(2).unwrap(); //命令字符 / The cmd char,like D-Dig,F-Flag,P-Pending,T-Test
					}
					if c_y != '!' && c_x as u8 >= 65 && c_y as u8 >= 65 {
						//确保输入的是A以上的字母 / Ensure the input char is ABCDE...
						let x = c_x as usize - 65;  // 65 is the char 'A'
						let y = c_y as usize - 65;  
						//确保未超最大行列 / Ensure row and column input is below the most table index.
						if x < game.level.cols as usize && y < game.level.rows as usize {

							game.dig_cell(&x, &y, &c_cmd); //挖开此单元格 / Begin dig cell
							if game_started==false{ //如果是第一个单元格，开始计时 / if the first cmd ,start timer.
								//开始计时 / Start timer
								game_started=true;
								game.timer.start();
							}
						}
					}
					if c_y == '!' { // !Command ,like !N=New game ,!Q=Quit,!P=Pause
						match c_x {
							'Q' => {
								//退出程序 / Exit program

								execute!(std::io::stdout(), Show).unwrap();
								disable_raw_mode().expect("Failed to enable raw mode");
								exit(0);
							}
							'P' => { //暂停游戏 / Pause
								game.pause();
							}
							'R' => { //继续游戏 / Restore
								game.restore();
							}
							'N' => {
								//新开游戏 / New game
								game = new_game();
								(x, y) = game.get_cmd_pos();
								cmd.clear();
								c_count = 0;
								game_started=false;
							}
							'C' => {
								game.display_err();
							}
							_ => {}
						}
					}
				}
				c_count = 0;
				cmd.clear();
			}
			Event::Key(KeyEvent { code: KeyCode::Backspace, .. }) => {
				//print!("\u{8}");//退格
				cmd.pop(); //删除最后一个字符 / delete the last char of cmd string
				print!("{} ", cmd);
				c_count -= 1;
				stdout.flush().expect("Failed to flush output");
			}
			Event::Key(KeyEvent { code: KeyCode::Esc, .. }) => {
				disable_raw_mode().expect("Failed to disable raw mode");
				cmd.clear();
				c_count = 0;
			}
			_ => game.update_time_consuming(),
		}

	}
	//显示光标 / Show cursor
	execute!(std::io::stdout(), Show).unwrap();
	//关闭raw mode / disable raw mode
	disable_raw_mode().expect("Failed to enable raw mode");
	clear_screen();
	Ok(())
}
