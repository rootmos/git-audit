extern crate config;
extern crate clap;
use clap::{App, SubCommand};
use std::fs;
use std::io::prelude::*;

const REPO_CONFIG_FILE: &str = ".git-audit";

fn main() {
    let matches = App::new("git-audit")
        .version("0.1.0")
        .author("Gustav Behm <me@rootmos.io>")
        .subcommand(SubCommand::with_name("init"))
        .get_matches();

    if let Some(_matches) = matches.subcommand_matches("init") {
        match fs::OpenOptions::new().write(true).create_new(true).open(REPO_CONFIG_FILE) {
            Ok(mut f) => f.write_all(b"foo = bar").unwrap(),
            Err(e) => panic!("{}", e)
        }
    }

    let mut settings = config::Config::default();
    settings.merge(config::File::new(REPO_CONFIG_FILE, config::FileFormat::Ini)).unwrap();
}
