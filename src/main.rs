mod editor;
mod terminal;
pub use terminal::Terminal;
pub use editor::Position;
use editor::Editor;

fn main() { 
    Editor::default().run();
}
