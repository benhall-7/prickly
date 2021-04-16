use structopt::StructOpt;

#[derive(Debug, StructOpt)]
pub struct Args {
    /// The param file to open on startup, if any
    pub file: Option<String>,
}
