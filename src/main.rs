use std::fs;
use std::fs::File;
use std::io::prelude::*;
use clap::{App, Arg, SubCommand};
use once_cell::sync::OnceCell;
use pulldown_cmark::{Event, Options, Parser, Tag};
use std::path::Path;
use std::io::Write;
use std::io;
use termcolor::{Color, ColorChoice, ColorSpec, StandardStream, WriteColor};
use termion::raw::IntoRawMode;
use termion::screen::AlternateScreen;
use tui::Terminal;
use tui::backend::TermionBackend;
use tui::widgets::{Widget, Block, Borders};
use tui::layout::{Layout, Constraint, Direction};

mod ui;
mod shell;
mod corg_file;
mod corg_doc;
mod clogger;
mod util;

use corg_file::CorgFile;

use util::*;
use clogger::*;
use termion::event::Key;

const CORG_LOGGER_SHELL_SCRIPT: &'static [u8] = include_bytes!("../static/scripts/corg-logger.sh");
pub const CORG_VERSION: &'static str = env!("CARGO_PKG_VERSION");

fn read_file(name: &str) -> String {
    let mut file_buffer = String::new();
    let mut file = match File::open(&name) {
        Ok(file) => file,
        Err(_) => {
            println!("Unable to open file {}", name);
            return String::new();
        }
    };

    file.read_to_string(&mut file_buffer)
        .unwrap_or_else(|err| panic!("Error reading config! [{}]", err));

    return String::from(file_buffer)
}

fn write_corg_logger() {
    let corg_logger_file_path = "scripts/utils/corg-logger.sh";
    let corg_logger_sh = File::create(corg_logger_file_path);

    match corg_logger_sh {
        Ok(mut file_handler) => {
            println!("Writing logger util to file: {}", &corg_logger_file_path);
            let _ = file_handler.write_all(b"\n# - start logger:\n");
            let _ = file_handler.write_all(CORG_LOGGER_SHELL_SCRIPT);
            let _ = file_handler.write_all(b"\n# - end logger:\n");
        },
        _ => log_error_message("Cannot write log file")
    }
}

fn play(file_name: &str, clogger: &mut Clog) {
    let mut app = ui::App::new(file_name, vec![], vec![]);
    let stdout = io::stdout().into_raw_mode().unwrap();
    /// Note: AlternateScreen
    /// Causes the terminal to show a new empty screen for drawing. This is good
    /// unless you wanna shit on-top of whats already been printed to stdout.
    let stdout = AlternateScreen::from(stdout);
    let backend = TermionBackend::new(stdout);
    let mut terminal = Terminal::new(backend).unwrap();
    let events = event::Events::new();

    loop {
        ui::draw(&mut terminal, &app);

        match events.next() {
            Ok(event) => {
                match event {
                    event::Event::Input(input) => {
                        match input {
                            Key::Ctrl('q') => {
                                break;
                            },
                            Key::Char('\n') => {
                                app.commands.push(app.input.drain(..).collect());
                            }
                            Key::Char(c) => {
                                app.input.push(c);
                            }
                            Key::Backspace => {
                                app.input.pop();
                            },
                            _ => ()
                        }
                    }
                    _ => ()
                }
            }
            _ => ()
        }
    }
//    terminal.draw(|mut f| {
//        let size = f.size();
//        Block::default()
//            .title("Block")
//            .borders(Borders::ALL)
//            .render(&mut f, size);
//    });
}

fn convert(file: &str, clogger: &mut Clog) {
    let log_message = format!("Converting {}", &file);
    clogger.info(&log_message);

    let file_path = Path::new(file);
    let maybe_file_name = file_path.file_stem().unwrap();

    if let Some(file_name) = maybe_file_name.to_str() {
        let corgdown_source = read_file(file);
        let out_shell_filename = format!("scripts/{}.sh", file_name);
        let mut le_file = CorgFile::new(&out_shell_filename, &corgdown_source);
        le_file.push_corgdown();

        match le_file.write_file() {
            Ok(_) => {
                let message = format!("Wrote file to {}", &le_file.file_name);
                clogger.success(&message)
            },
            Err(_) => {
                let message = format!("Le fuck... failed to write file to {}", &le_file.file_name);
                clogger.error(&message)
            }
        }
        // Write supporting files
        write_corg_logger();
    } else {
        clogger.error("Ru-roh! No matching file found!")
    }
}

fn log_error_message(message: &str) {
    println!("Error! {}", message)
}

fn main() {
    // Initialize logging singleton
    let mut stdout = StandardStream::stdout(ColorChoice::Always);
    let mut clogger = Clog::new(ClogLevel::Info, &mut stdout);

    let matches = App::new("Corg")
        .version("0.1")
        .author("tehprofessor <me@tehprofessor.com>")
        .about("Read and execute shell scripts from notes written in Markdown")
        .arg(
            Arg::with_name("convert")
                .short("cc")
                .long("convert")
                .value_name("FILE")
                .takes_value(true)
                .help("Converts the given markdown file into an executable shell script."),
        )
        .subcommand(
            SubCommand::with_name("run")
                .about("Run a corg script locally or remotely")
                .version("0.3312")
                .author("tehprofessor <me@tehprofessor.com>")
                .arg(
                    Arg::with_name("script")
                        .short("s")
                        .long("script")
                        .value_name("CORG_SCRIPT")
                        .takes_value(true)
                        .help("Path of the Corg shell script to execute, e.g. scripts/nix.sh")
                )
                .arg(
                    Arg::with_name("host")
                        .short("c")
                        .long("host")
                        .value_name("CORG_HOST")
                        .takes_value(true)
                        .help("The hostname where the script will be executed (requires ssh).")
                )
        )
        .get_matches();

    if let Some(file) = matches.value_of("convert") {
        convert(file, &mut clogger);
    } else if let Some(file) = matches.subcommand_matches("run") {
        play("Fart Salads", &mut clogger);
    } else {
        println!("No command found, please see --help for usage information");
    };

    // shell::write_html(&mut out_file_handle, parser).unwrap();
}
