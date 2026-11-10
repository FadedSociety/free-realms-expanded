//! Prints the `LoadingScreen.xml` the client reads from its install folder, built from
//! `splash_screens.json` (`osfr_tools::splash_screens`). The client takes its loading screens
//! from that file alone, so whoever lays out the install (the launcher) writes this output there.
//!
//! `osfr-splash <content directory> [live event key ...]`
use osfr_tools::splash_screens::Table;
use std::{path::Path, process::ExitCode};

fn main() -> ExitCode {
    let mut args = std::env::args().skip(1);
    let Some(content) = args.next() else {
        eprintln!("usage: osfr-splash <content directory> [live event key ...]");
        return ExitCode::from(2);
    };
    match Table::load(Path::new(&content)) {
        Ok(Some(table)) => {
            print!("{}", table.loading_screen_xml(&args.collect::<Vec<_>>()));
            ExitCode::SUCCESS
        }
        Ok(None) => {
            eprintln!("no splash_screens.json in {content}");
            ExitCode::FAILURE
        }
        Err(error) => {
            eprintln!("{error}");
            ExitCode::FAILURE
        }
    }
}
