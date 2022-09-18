use editor::Editor;

pub use document::Document;
pub use editor::Position;
pub use editor::SearchDirection;
pub use row::Row;
pub use terminal::Terminal;
pub use highlight::Highlight;

mod document;
mod editor;
mod row;
mod terminal;
mod highlight;

fn main() {
    Editor::default().run();
}
