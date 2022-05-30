use tui_components::rect_ext::RectExt;
use tui_components::tui::layout::Rect;
use tui_components::tui::text::Text;
use tui_components::tui::widgets::Widget;
use tui_components::tui::{
    layout::Alignment,
    style::{Color, Style},
    widgets::Paragraph,
};
use tui_components::Component;

pub struct Empty;

impl Component for Empty {
    type Response = ();
    type DrawResponse = ();

    fn handle_event(&mut self, _event: tui_components::Event) -> Self::Response {}

    fn draw(
        &mut self,
        rect: tui_components::tui::layout::Rect,
        buffer: &mut tui_components::tui::buffer::Buffer,
    ) -> Self::DrawResponse {
        let mut message = Text::raw("No params loaded. Press\n");
        message.extend(Text::styled(
            "ctrl + o\n",
            Style::default().fg(Color::Green),
        ));
        message.extend(Text::raw("to open a file"));

        let uncentered = Rect {
            x: 0,
            y: 0,
            width: message.width() as u16,
            height: message.height() as u16,
        };
        let center = rect.centered(uncentered);

        let paragraph = Paragraph::new(message).alignment(Alignment::Center);
        paragraph.render(center, buffer);
    }
}
