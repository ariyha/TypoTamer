use crate::Terminal;
use crate::Document;
use crate::Row;
use termion::event::Key;
use termion::color;
use std::time::Duration;
use std::time::Instant;

const VERSION: &str = env!("CARGO_PKG_VERSION");
const STATUS_BG_COLOR: color::Rgb = color::Rgb(239, 239, 0);
const STATUS_FG_COLOR: color::Rgb = color::Rgb(63, 63, 63);
const QUIT_TIMES: u8 = 3;

struct StatusMsg{  
    text: String,
    time: Instant,
}

impl StatusMsg{
    pub fn from(msg: String) -> Self {
        Self{
            text: msg,
            time: Instant::now(),
        }
    }
}   

#[derive(Default)]
pub struct Position {
    pub x: usize,
    pub y: usize,
}

pub struct Editor {
    shld_quit: bool,
    terminal: Terminal,
    cursor_position: Position,
    document: Document,
    offset: Position,
    status_msg: StatusMsg,
    quit_times: u8,
}

impl Editor {

    pub fn run(&mut self) {
        loop {
            if let Err(error) = self.refresh_screen() {
                die(error);
            }

            if self.shld_quit {
                break;
            }

            if let Err(error) = self.process_key() {
                die(error);
            }
        }
    }

    pub fn default() -> Self {
        
        let args: Vec<String> = std::env::args().collect();
        let mut initial_status = String::from("HELP: Ctrl-Q = quit");
        let document = if args.len() > 1 {
            let filename = &args[1];
            let doc = Document::open(&filename);
            if doc.is_ok(){
                doc.unwrap()
            }
            else{
                initial_status = format!("ERR: Could not open file: {}", filename);
                Document::default()
            }
        } else {
            Document::default()
        };  

        Self {
            shld_quit: false,
            terminal: Terminal::default().expect("Failed to initialize terminal"),
            cursor_position: Position::default(),
            document,
            offset: Position::default(),
            status_msg: StatusMsg::from(initial_status),
            quit_times: QUIT_TIMES,
        }
    }

    fn draw_status_bar(&self){
        
        let mut status;
        let modified_indicator = if self.document.is_dirty(){
            "(modified)"
        }
        else{
            ""
        };
        let width = self.terminal.size().width as usize;
        let mut file_name = "[No Name]".to_string();
        if let Some(name) = &self.document.file_name {
            file_name = name.clone();
            file_name.truncate(20);
        }
        status = format!("{} - {} lines {}", file_name, self.document.len(),modified_indicator);

        let line_indicator = format!("{}:{}", self.cursor_position.y.saturating_add(1), self.cursor_position.x.saturating_add(1));  

        let len = status.len() + line_indicator.len();

        if len<width{
            status.push_str(&" ".repeat(width - len));
        }
        status = format!("{}{}", status, line_indicator);
        status.truncate(width);
        Terminal::set_bg_color(STATUS_BG_COLOR);
        Terminal::set_fg_color(STATUS_FG_COLOR);
        println!("{}\r", status);
        Terminal::reset_bg_color();
        Terminal::reset_fg_color();
    }

    fn draw_msg_bar(&self){
        Terminal::clear_current_line();
        let message = &self.status_msg;
        if Instant::now() - self.status_msg.time < Duration::new(5,0){
            let mut text = message.text.clone();
            text.truncate(self.terminal.size().width as usize);
            println!("{}\r", text);
        }
    }

    fn save(&mut self) {
        if self.document.file_name.is_none() {
            let new_name = self.prompt("Save as: ", |_, _, _| {}).unwrap_or(None);
            if new_name.is_none() {
                self.status_msg = StatusMsg::from("Save aborted.".to_string());
                return;
            }
            self.document.file_name = new_name;
        }

        if self.document.save().is_ok() {
            self.status_msg = StatusMsg::from("File saved successfully.".to_string());
        } else {
            self.status_msg = StatusMsg::from("Error writing file!".to_string());
        }
    }
    fn process_key(&mut self) -> Result<(), std::io::Error> {
        let pressed_key = Terminal::read_key()?;

        match pressed_key {
            Key::Ctrl('q') => {
                if self.quit_times>0 && self.document.is_dirty(){
                    self.status_msg = StatusMsg::from(format!("WARNING!!! File has unsaved changes. Press Ctrl-Q {} more times to quit.", self.quit_times));
                    self.quit_times -= 1;
                    return Ok(());
            
            }
            self.shld_quit = true;
        }   ,
            Key::Ctrl('s') => self.save(),
            Key::Ctrl('f') =>{
                    if let Some(query) = self
                        .prompt("Search: ", |editor, _, query| {
                            if let Some(position) = editor.document.find(&query) {
                                editor.cursor_position = position;
                                editor.scroll();
                            }
                        })
                        .unwrap_or(None)
                    {
                        if let Some(position) = self.document.find(&query[..]){
                        self.cursor_position = position;
                        self.scroll();
                    }
                    else{
                        self.status_msg = StatusMsg::from(format!("Search for '{}' failed", query));
                    }
                    
                }
            }
            Key::Delete => self.document.delete(&self.cursor_position),
            Key::Char(c) => {
                self.document.insert(&self.cursor_position, c);
                self.move_cursor(Key::Right);
                },
            Key::Backspace=> {
                if self.cursor_position.x > 0 || self.cursor_position.y > 0 {
                    self.move_cursor(Key::Left);
                    self.document.delete(&self.cursor_position);
                }
            },
              Key::Up 
            | Key::Down 
            | Key::Left 
            | Key::Right
            | Key::End
            | Key::Home
            | Key::PageDown
            | Key::PageUp => self.move_cursor(pressed_key),
            _ => (),
        }
        self.scroll();
        Ok(())
    }

    fn prompt<C>(&mut self, prompt: &str, callback: C) -> Result<Option<String>, std::io::Error>
    where
        C: Fn(&mut Self, Key, &String),
    {
        let mut result = String::new();
        loop {
            self.status_msg = StatusMsg::from(format!("{}{}", prompt, result));
            self.refresh_screen()?;
            let key = Terminal::read_key()?;
            match key {
                Key::Backspace => result.truncate(result.len().saturating_sub(1)),
                Key::Char('\n') => break,
                Key::Char(c) => {
                    if !c.is_control() {
                        result.push(c);
                    }
                }
                Key::Esc => {
                    result.truncate(0);
                    break;
                }
                _ => (),
            }
            callback(self, key, &result);
        }
        self.status_msg = StatusMsg::from(String::new());
        if result.is_empty() {
            return Ok(None);
        }
        Ok(Some(result))
    }
    fn refresh_screen(&self) -> Result<(), std::io::Error> {
        Terminal::cursor_hide();
        Terminal::cursor_position(&Position { x: 0, y: 0 });
        if self.shld_quit {
            Terminal::clear_screen();
            println!("Don't forget to commit!\r");
        } else {
            self.draw_tilde();
            self.draw_status_bar();
            self.draw_msg_bar();
            Terminal::cursor_position(&Position{
                x:self.cursor_position.x.saturating_sub(self.offset.x),
                y:self.cursor_position.y.saturating_sub(self.offset.y),
            } )
        }
        Terminal::cursor_show();
        Terminal::flush()
    }

    fn draw_tilde(&self) {
        let height = self.terminal.size().height;
        for terminal_row in 0..height {
            Terminal::clear_current_line();
            if let Some(row) = self.document.row(terminal_row as usize + self.offset.y) {
                self.draw_row(row);
            }
            else if self.document.is_empty() && terminal_row == height / 3 {
                self.welcome_message();
            } else {
                println!("~\r");
            }
        }
    }
    fn welcome_message(&self) {
        let mut welcome_message = format!("TypoTamer -- version {}. If u find any bugs/issues try not to create it again(I aint responsible for those)", VERSION);
        let width = self.terminal.size().width as usize;
        let len = welcome_message.len();
        let padding = width.saturating_sub(len) / 2;
        let spaces = " ".repeat(padding.saturating_sub(1));
        welcome_message = format!("~{}{}", spaces, welcome_message);
        welcome_message.truncate(width);
        println!("{}\r", welcome_message);
    }
    fn scroll(&mut self) {            
        let Position { x, y } = self.cursor_position;            
        let width = self.terminal.size().width as usize;            
        let height = self.terminal.size().height as usize;            
        let offset: &mut Position = &mut self.offset;            
        if y < offset.y {            
            offset.y = y;               
        } else if y >= offset.y.saturating_add(height) {            
            offset.y = y.saturating_sub(height).saturating_add(1);            
        }            
        if x < offset.x {            
            offset.x = x;            
        } else if x >= offset.x.saturating_add(width) {            
            offset.x = x.saturating_sub(width).saturating_add(1);            
        }            
    }            

    fn move_cursor(&mut self, key: Key) {
        let terminal_height = self.terminal.size().height as usize;
        let Position { mut y, mut x } = 
        self.cursor_position;
        let height = self.document.len();
        let mut width = if let Some(row) = self.document.row(y) {
            row.len()
        } else {
            0
        };

        match key {
            Key::Up => y = y.saturating_sub(1),
            Key::Down => {
                if y < height {
                    y = y.saturating_add(1);
                }
            }
            Key::Left => {
                if x > 0 {
                    x-=1;
                }
                else if y > 0 {
                    y-=1;
                    x = if let Some(row) = self.document.row(y) {
                        row.len()
                    } else{
                        0
                    }
                    
                }
            },
            Key::Right => {
                if x < width {
                    x+=1;
                }
                else if y < height {
                    y+=1;
                    x = 0;
                }
            },
            Key::PageUp => {
                y = if y > terminal_height {
                    y - terminal_height
                } else {
                    0
                }
            },
            Key::PageDown => {
                y = if y.saturating_add(terminal_height)<height{
                    y + terminal_height
                } else {
                    height
                }
            },
            Key::Home => x = 0,
            Key::End => x = width,
            _ => (),
        }

        width = if let Some(row) = self.document.row(y) {
            row.len()
        } else {
            0
        };

        if x>width {
            x = width;
        }

        self.cursor_position = Position { x, y }
    }

    pub fn draw_row(&self, row: &Row) {
        let width = self.terminal.size().width as usize;
        let start = self.offset.x;
        let end = self.offset.x + width;
        let row = row.render(start, end);
        println!("{}\r", row);
    }
}



fn die(e: std::io::Error) {
    Terminal::clear_screen();
    panic!("{}", e);
}
