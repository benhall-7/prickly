mod app;
mod filter;
mod input;
mod tree;

pub use app::*;
pub use filter::*;
pub use input::*;
pub use tree::*;

use crossterm::event::{KeyEvent, MouseEvent};
use tui::buffer::Buffer;
use tui::layout::Rect;
use tui::widgets::StatefulWidget;

pub struct Wrapper;

impl StatefulWidget for Wrapper {
    type State = App;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut App) {
        state.draw(area, buf)
    }
}

/// A trait enabling a nested layout of structs
pub trait Component {
    type Response;

    fn handle_event(&mut self, event: Event) -> Self::Response;

    fn draw(&mut self, rect: Rect, buffer: &mut Buffer);
}

#[derive(Debug, Copy, Clone)]
pub enum Event {
    Key(KeyEvent),
    Mouse(MouseEvent),
}
