use std::io::{stdout, Cursor};
use std::time::Duration;

use crossterm::event::{poll, read, Event};
use crossterm::execute;
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen, SetTitle,
};
use prc::hash40::{read_custom_labels, set_custom_labels};
use prc::{param::*, read_stream};
use tui::backend::CrosstermBackend;
use tui::Terminal;

mod comp;
mod error;

use comp::*;

const TEMP_PARAM: &'static [u8] = include_bytes!("fighter_param.prc");

fn main() -> Result<(), error::AppError> {
    // TODO:
    // load param file from args

    let param = ParamKind::from(read_stream(&mut Cursor::new(TEMP_PARAM)).unwrap());
    if let Ok(l) = read_custom_labels("ParamLabels.csv") {
        set_custom_labels(l.into_iter())
    }

    execute!(
        stdout(),
        SetTitle("prickly - prc file editor"),
        EnterAlternateScreen
    )?;
    enable_raw_mode()?;
    let mut t = Terminal::new(CrosstermBackend::new(stdout())).unwrap();
    t.clear().unwrap();

    let mut app = App::new(param);
    let mut should_refresh = true;

    loop {
        if should_refresh {
            t.draw(|f| {
                let size = f.size();
                f.render_stateful_widget(comp::Wrapper, size, &mut app);
            })
            .unwrap();
            should_refresh = false;
        }

        if poll(Duration::from_secs_f64(1.0 / 60.0)).unwrap() {
            should_refresh = true;
            let event = read().unwrap();
            let comp_event = match event {
                Event::Resize(..) => continue,
                Event::Mouse(m) => comp::Event::Mouse(m),
                Event::Key(k) => comp::Event::Key(k),
            };
            match app.handle_event(comp_event) {
                AppResponse::Exit => break,
                AppResponse::None => {}
            }
        }
    }

    disable_raw_mode()?;
    execute!(stdout(), LeaveAlternateScreen)?;
    t.clear().unwrap();

    Ok(())
}
