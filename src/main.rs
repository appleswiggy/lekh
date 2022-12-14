use editor::Editor;

pub use document::Document;
pub use editor::Position;
pub use editor::SearchDirection;
pub use highlight::Highlighter;
pub use row::Row;
pub use terminal::Terminal;

mod document;
mod editor;
mod highlight;
mod row;
mod terminal;

fn main() {
    Editor::default().run();
}
