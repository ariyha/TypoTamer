use crate::Terminal;
use termion::event::Key;

const VERSION: &str = env!("CARGO_PKG_VERSION");

pub struct Position {
    pub x: usize,
    pub y: usize,
}

pub struct Editor {
    shld_quit: bool,
    terminal: Terminal,
    cursor_position: Position,
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
        Self {
            shld_quit: false,
            terminal: Terminal::default().expect("Failed to initialize terminal"),
            cursor_position: Position { x: 0, y: 0 },
        }
    }

    fn process_key(&mut self) -> Result<(), std::io::Error> {
        let pressed_key = Terminal::read_key()?;

        match pressed_key {
            Key::Ctrl('q') => self.shld_quit = true,
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
        Ok(())
    }

    fn refresh_screen(&self) -> Result<(), std::io::Error> {
        Terminal::cursor_hide();
        Terminal::cursor_position(&Position { x: 0, y: 0 });
        if self.shld_quit {
            Terminal::clear_screen();
            println!("Don't forget to commit!\r");
        } else {
            self.draw_tilde();
            Terminal::cursor_position(&self.cursor_position )
        }
        Terminal::cursor_show();
        Terminal::flush()
    }

    fn draw_tilde(&self) {
        let height = self.terminal.size().height;
        for row in 0..height - 1 {
            Terminal::clear_current_line();
            if row == height / 3 {
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

    fn move_cursor(&mut self, key: Key) {            
        let Position { mut y, mut x } = self.cursor_position;     
        let size = self.terminal.size();
        let height = size.height.saturating_sub(1) as usize;
        let width = size.width.saturating_sub(1) as usize;       
        match key {            
            Key::Up => y = y.saturating_sub(1),            
            Key::Down => {
                if y < height {                    
                    y = y.saturating_add(1)                
                }  
            },            
            Key::Left => {            
                if x > 0 {            
                    x = x.saturating_sub(1);            
                }            
                else{
                    if y > 0 {
                        y = y.saturating_sub(1);
                        x = width;
                    }
                }            
            },            
            Key::Right => {            
                if x < width {            
                    x = x.saturating_add(1);            
                }
                else{
                    y = y.saturating_add(1);
                    x = 0;  
                }            
            },
            Key::PageUp => y = 0,            
            Key::PageDown => y = height,            
            Key::Home => x = 0,            
            Key::End => x = width,            
            
        _ => (),            
        }            
        self.cursor_position = Position { x, y }            
    }            

}

fn die(e: std::io::Error) {
    Terminal::clear_screen();
    panic!("{}", e);
}
