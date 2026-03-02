mod cli;
mod serial;
mod model;
mod app;
mod ui;

use anyhow::Result;

fn main() -> Result<()> {
    let args = cli::parse_args();
    if args.debug {
        std::env::set_var("MESHCORESTAT_DEBUG", "1");
    }
    app::run(args)
}
