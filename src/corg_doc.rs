use crate::corg_file::CorgFile;
use std::fs::File;
use std::io::prelude::*;
use std::io::Error;
use std::path::{Path, PathBuf};
use std::process::{Child, Command};
use std::{env, fs};
use walkdir::{DirEntry, WalkDir};

#[derive(Debug, Clone)]
pub struct CorgDoc<'a> {
    pub file_name: &'a str,
    pub file_contents: Option<String>,
    pub file_type: CorgFileType,
}

impl CorgDoc<'_> {
    pub fn new(file_name: &str) -> CorgDoc {
        let mut corg_doc = CorgDoc {
            file_name,
            file_contents: None,
            file_type: CorgFileType::Markdown,
        };

        corg_doc.read_file(file_name);

        corg_doc
    }

    fn read_file(&mut self, file_name: &str) {
        if self.file_contents == None {
            let file_path = Path::new(file_name);
            let mut file_buffer = String::new();
            let mut file = match File::open(&file_path) {
                Ok(file) => file,
                Err(_) => {
                    println!("Unable to find file matching {}", file_name);
                    return;
                }
            };

            // I really need to like look up how to handle errors better.
            file.read_to_string(&mut file_buffer)
                .unwrap_or_else(|err| panic!("Error reading corgdown file! [{}]", err));

            self.file_contents = Some(String::from(file_buffer));
        }
    }

    fn find_document(name: &str) -> Option<PathBuf> {
        let current_dir = env::current_dir().unwrap();
        let mut document_path: Option<PathBuf> = None;
        if let Ok(files) = fs::read_dir(current_dir) {
            for entry in files {
                match entry {
                    Ok(entry) => {
                        let path = entry.path();
                        // YOU FIGURE THIS OUT. YOU NEED TO ITERATE THROUGH
                        // THE SUB DIRECTORIES TO FIND THE MATCHING FILE.
                        // THEN EXECUTE AND/OR SCP the MOTHERFUCKER, BEEBz.
                        if path.is_dir() {}
                        if let Some(file_stem) = path.file_name() {
                            if let Some(maybe_file_name) = file_stem.to_str() {
                                if maybe_file_name.contains(name) {
                                    let success_path = path.clone();
                                    document_path = Some(success_path);
                                    break;
                                }
                            }
                        }
                    }
                    Err(_) => (),
                }
            }
        }
        return document_path;
    }
}

fn is_hidden(entry: &DirEntry) -> bool {
    entry
        .file_name()
        .to_str()
        .map(|s| s.starts_with("."))
        .unwrap_or(false)
}

#[derive(Debug, Clone, Copy)]
pub enum CorgFileType {
    Markdown,
    ShellScript
}

impl CorgFileType {
    pub fn to_string(&self) -> String {
        match self {
            Self::Markdown => "md".to_string(),
            Self::ShellScript => "sh".to_string(),
        }
    }

    pub fn is_file_type(&self, entry: &DirEntry) -> bool {
        let extension = self.to_string();

        let result = entry
            .file_name()
            .to_str()
            .map(|s| s.ends_with(extension.as_str()))
            .unwrap_or(false);

        if result {
            let val = entry.file_name().to_str().unwrap();
            println!("found markdown file {}", val);
        }

        result
    }

    fn is_matching_file(&self, entry: &DirEntry, file_name: &str) -> bool {
        let file_ext = self.to_string();
        let file_name_with_ext = format!("{}.{}", file_name, file_ext);

        entry
            .file_name()
            .to_str()
            .map(|s| s.contains(file_name_with_ext.as_str()))
            .unwrap_or(false)
    }
}

fn find_file(file_name: &str, file_type: CorgFileType) -> Option<PathBuf> {
    let current_dir = env::current_dir().unwrap();
    let walker = WalkDir::new(current_dir)
        .contents_first(true)
        .into_iter()
        .filter_entry(|e| !is_hidden(e))
        .filter_map(|e| e.ok());
    let mut matching_entry: Option<DirEntry> = None;

    for entry in walker {
        if file_type.is_matching_file(&entry, file_name) {
            matching_entry = Some(entry);
            break;
        }
    }

    match matching_entry {
        Some(entry) => Some(entry.into_path()),
        None => None
    }
}

pub struct CorgRunner {
    pub host: String,
    pub script: String,
    pub result: String,
}

impl CorgRunner {
    fn exec(script_name: &str, args: Vec<String>) -> Result<Child, Error> {
        let mut command = Command::new(script_name);

        for arg in args.iter() {
            command.arg(arg);
        }

        command.spawn()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        let corg_doc = CorgDoc::new("examples/faye.md");
        assert_ne!(corg_doc.file_contents, None);
    }

    #[test]
    fn test_find_document() {
        let example_doc = "faye";
        let expected = Some(PathBuf::from("/Users/seve/Code/Projects/corg/faye.md"));
        let result = CorgDoc::find_document("faye");
        assert_eq!(result, expected);
    }

    #[test]
    fn test_is_file_type() {
        let corg_file_type = CorgFileType::Markdown;
        let path_buf = PathBuf::from("/Users/seve/code/Projects/corg/examples/faye.md");
    }

    #[test]
    fn test_find_file() {
        let example_doc = "faye";
        let actual = find_file(example_doc, CorgFileType::Markdown);
        let expected = Some(PathBuf::from("/Users/seve/Code/Projects/corg/examples/faye.md"));
        assert_eq!(actual, expected);
    }
}
