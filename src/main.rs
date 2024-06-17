mod editor;
mod terminal;
mod document;
mod row;
pub use terminal::Terminal;
pub use editor::Position;
use editor::Editor;
pub use row::Row;
pub use document::Document;

fn main() { 
    Editor::default().run();
}
