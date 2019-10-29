use std::path::PathBuf;
use std::{env, fs};
use walkdir::{DirEntry, WalkDir, FilterEntry, IntoIter, Error};
use std::iter::FilterMap;

pub mod event;

fn is_hidden(entry: &DirEntry) -> bool {
    entry
        .file_name()
        .to_str()
        .map(|s| s.starts_with("."))
        .unwrap_or(false)
}

fn is_file_type(entry: &DirEntry, extension: &str) -> bool {
    let result = entry
        .file_name()
        .to_str()
        .map(|s| s.ends_with(extension))
        .unwrap_or(false);

    result
}

fn is_matching_file(entry: &DirEntry, file_name: &str, file_ext: &str) -> bool {
    let file_ext = file_ext.to_string();
    let file_name_with_ext = format!("{}.{}", file_name, file_ext);

    entry
        .file_name()
        .to_str()
        .map(|s| s.contains(&file_name_with_ext))
        .unwrap_or(false)
}

pub type Walker = FilterMap<FilterEntry<IntoIter, fn(&DirEntry) -> bool>, fn(Result<DirEntry, Error>) -> Option<DirEntry>>;

pub fn find_file(file_name: &str, file_type: &str, walker: Walker) -> Option<PathBuf> {
    let mut matching_entry: Option<DirEntry> = None;

    for entry in walker {
        if is_matching_file(&entry, &file_name, &file_type) {
            matching_entry = Some(entry);
            break;
        }
    }

    match matching_entry {
        Some(entry) => Some(entry.into_path()),
        None => None
    }
}

pub fn find_files_by_type(file_type: &str, walker: Walker) -> Vec<String> {
    let mut files: Vec<String> = vec![];
    for entry in walker {
        if is_file_type(&entry,file_type) {
            let path = entry.into_path();
            match path.to_str() {
                Some(path) => {
                    files.push(String::from(path));
                },
                _ => ()
            }
        }
    }

    files
}

fn find(file_name: &str, file_type: &str) -> Option<PathBuf> {
    let current_dir = env::current_dir().unwrap();
    let walker = WalkDir::new(current_dir)
        .contents_first(true)
        .into_iter()
        .filter_entry(|e| !is_hidden(e))
        .filter_map(|e| e.ok());
    let mut matching_entry: Option<DirEntry> = None;

    for entry in walker {
        if is_matching_file(&entry, &file_name, &file_type) {
            matching_entry = Some(entry);
            break;
        }
    }

    match matching_entry {
        Some(entry) => Some(entry.into_path()),
        None => None
    }
}
