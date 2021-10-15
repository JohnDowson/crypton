use std::env::{self, var_os};
use std::ffi::OsString;
use std::process::Command;
use thiserror::Error;

pub type CResult<T> = Result<T, Box<dyn std::error::Error>>;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("No editor could be found. Please set $EDITOR of $VISUAL")]
    NoEditor,
    #[error("The note was empty")]
    EmptyNote,
    #[error("No note with this hash could be found")]
    NotFound,
}
pub fn get_editor() -> Result<OsString, AppError> {
    var_os("EDITOR")
        .or_else(|| var_os("VISUAL"))
        .or_else(|| search_path(&["vim", "nvim", "nano"]))
        .ok_or(AppError::NoEditor)
}

fn search_path(exec_names: &[&str]) -> Option<OsString> {
    var_os("PATH").and_then(|paths| {
        env::split_paths(&paths).find_map(|dir| {
            for &exec_name in exec_names {
                let full_path = dir.join(exec_name);
                if full_path.is_file() {
                    return Some(full_path.into_os_string());
                }
            }
            None
        })
    })
}

pub fn open_in_editor(note: &str) -> CResult<String> {
    use std::io::Write;
    let editor = get_editor()?;
    let mut temp_file = tempfile::NamedTempFile::new()?;
    temp_file.write_all(note.as_bytes())?;
    let temp_path = temp_file.into_temp_path();
    Command::new(editor).arg(&temp_path).status()?;
    let note = std::fs::read_to_string(&temp_path)?;
    temp_path.close()?;
    Ok(note)
}

pub fn note_hash(note: &str) -> CResult<String> {
    use sha1::{Digest, Sha1};
    use std::fmt::Write;
    let mut hasher = Sha1::new();
    hasher.update(note.as_bytes());
    let mut string = String::new();
    let hash = hasher.finalize();
    for byte in hash {
        write!(&mut string, "{:X}", byte)?;
    }
    Ok(string)
}
