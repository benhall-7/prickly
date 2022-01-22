use std::env::current_exe;

use prc::hash40::{read_custom_labels, set_custom_labels};
use prc::open;
use structopt::StructOpt;

mod args;
mod comp;
mod error;
mod app;
mod route;

use app::App;

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

    let mut app = App::new(param);

    tui_components::run(&mut app, Some("prickly - prc file editor".to_string()))?;
    Ok(())
}
