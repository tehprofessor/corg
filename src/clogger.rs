use once_cell::sync::OnceCell;
use std::io::Write;
use termcolor::{Color, ColorChoice, ColorSpec, StandardStream, WriteColor};
use std::{fmt, mem};
use std::sync::Once;

#[derive(Debug, Copy, Clone)]
pub enum ClogLevel {
    Debug,
    Info,
    Success,
    Warning,
    Error,
    Trace,
    Announce,
}

impl ClogLevel {
    pub fn color(&self) -> Color {
        match self {
            Self::Debug => {
                Color::Blue
            }
            Self::Info => {
                Color::Cyan
            }
            Self::Success => {
                Color::Green
            }
            Self::Warning => {
                Color::Yellow
            }
            Self::Error => {
                Color::Red
            }
            Self::Trace => {
                Color::Rgb(255, 165, 0) // Orange
            }
            Self::Announce => {
                Color::Magenta
            }
        }
    }

    fn should_log(&self, level: ClogLevel) -> bool {
        match (self, level) {
            (Self::Info, Self::Debug) => false,
            (Self::Success, Self::Debug) => false,
            (Self::Success, Self::Info) => false,
            (Self::Warning, Self::Debug) => false,
            (Self::Warning, Self::Info) => false,
            (Self::Warning, Self::Success) => false,
            (Self::Warning, Self::Success) => false,
            (Self::Error, Self::Debug) => false,
            (Self::Error, Self::Info) => false,
            (Self::Error, Self::Success) => false,
            (Self::Error, Self::Success) => false,
            (Self::Error, Self::Warning) => false,
            (_, _) => true
        }
    }

    fn to_string(&self) -> String {
        match self {
            Self::Debug => String::from("debug"),
            Self::Info => String::from("info"),
            Self::Success => String::from("success"),
            Self::Warning => String::from("warning"),
            Self::Error => String::from("error"),
            Self::Trace => String::from("trace"),
            Self::Announce => String::from("announce"),
        }
    }
}

pub struct Clog<'a> {
    pub level: ClogLevel,
    pub output: &'a mut StandardStream
}

impl<'a> Clog<'a> {
    pub fn new(level: ClogLevel, output: &'a mut StandardStream) -> Clog<'a> {
        Self { level, output }
    }

    pub fn debug(&mut self, message: &str) {
        self.write_log(ClogLevel::Announce, message);
    }

    pub fn info(&mut self, message: &str) {
        self.write_log(ClogLevel::Info, message);
    }

    pub fn success(&mut self, message: &str) {
        self.write_log(ClogLevel::Success, message);
    }

    pub fn warning(&mut self, message: &str) {
        self.write_log(ClogLevel::Warning, message);
    }

    pub fn error(&mut self, message: &str) {
        self.write_log(ClogLevel::Error, message);
    }

    pub fn trace(&mut self, message: &str) {
        self.write_log(ClogLevel::Trace, message);
    }

    pub fn announce(&mut self, message: &str) {
        self.write_log(ClogLevel::Announce, message);
    }

    fn set_log_level_color(&mut self) {
        let log_level_color = self.level.color();

        self.set_bold();
        self.set_color(log_level_color);
    }

    fn write(&mut self, message: &str) {
        let _ = write!(&mut self.output, "{}", message);
    }

    fn write_log(&mut self, level: ClogLevel, message: &str) {
        if self.level.should_log(level) {
            self.write_log_level();
            self.write(message);
            self.write_newline();
        }
    }

    fn write_log_level(&mut self) {
        let level_text = self.level.to_string();
        self.set_white();
        self.set_bold();
        self.write("[");
        self.set_log_level_color();
        self.write(&level_text);
        self.set_white();
        self.set_bold();
        self.write("]");
        self.set_default();
        self.write(" ");
    }

    fn write_newline(&mut self) {
        self.write("\n");
    }

    fn set_bold(&mut self) {
        let _ = self.output.set_color(ColorSpec::new().set_bold(true));
    }

    fn set_color(&mut self, color: Color) {
        let color = Some(color);
        let _ = self.output.set_color(ColorSpec::new().set_fg(color));
    }

    fn set_white(&mut self) {
        self.set_color(Color::White);
    }

    fn set_default(&mut self) {
        let _ = self.output.set_color(ColorSpec::new().set_bold(false));
        self.set_white()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

//    #[test]
//    fn test_write_info() {
//        let mut stdout = StandardStream::stdout(ColorChoice::Always);
//        let mut clogger = Clog::new(ClogLevel::Info, &mut stdout);
//        let mut file_buf: Vec<u8> = vec![];
//    }
}
