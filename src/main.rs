use std::env::current_exe;
use std::io::stdout;
use std::time::Duration;

use crossterm::event::{poll, read, Event};
use crossterm::execute;
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen, SetTitle,
};
use prc::hash40::{read_custom_labels, set_custom_labels};
use prc::open;
use structopt::StructOpt;
use tui::backend::CrosstermBackend;
use tui::Terminal;

mod args;
mod comp;
mod error;
mod rect_ext;

use comp::*;

fn main() -> Result<(), error::AppError> {
    let args = args::Args::from_args();

    let param = args
        .file
        .map(|path| open(path).unwrap())
        .unwrap_or_default()
        .into();

    if let Ok(l) = read_custom_labels("ParamLabels.csv") {
        set_custom_labels(l.into_iter())
    } else if let Some(l) = current_exe()
        .ok()
        .and_then(|path| read_custom_labels(path.parent().unwrap().join("ParamLabels.csv")).ok())
    {
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
