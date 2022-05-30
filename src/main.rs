use std::collections::BTreeSet;
use std::env::current_exe;
use std::sync::{Arc, Mutex};

use prc::hash40::label_map::LabelMap;
use prc::hash40::Hash40;
use prc::open;
use structopt::StructOpt;

use components::root::Root;

mod args;
mod error;

pub mod components;
pub mod utils;

fn main() -> Result<(), error::AppError> {
    let args = args::Args::from_args();

    let param = args.file.map(|path| open(path).unwrap().into());

    let mut sorted_labels = BTreeSet::new();
    let label_arc = Hash40::label_map();
    let label_map = label_arc.lock().ok();
    let labels = LabelMap::read_custom_labels("ParamLabels.csv")
        .ok()
        .or_else(|| {
            current_exe().ok().and_then(|path| {
                LabelMap::read_custom_labels(path.parent().unwrap().join("ParamLabels.csv")).ok()
            })
        });
    if let Some((labels, mut label_map)) = labels.zip(label_map) {
        sorted_labels = labels.iter().map(|(_, str)| str.to_owned()).collect();
        label_map.strict = true;
        label_map.add_custom_labels(labels.into_iter());
    }

    let mut app = Root::new(param, Arc::new(Mutex::new(sorted_labels)));

    tui_components::run(&mut app, Some("prickly - prc file editor".to_string()))?;
    Ok(())
}
