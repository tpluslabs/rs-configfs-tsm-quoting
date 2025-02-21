use std::{fs, io::{self, Write}, os::unix::fs::OpenOptionsExt, path::PathBuf};

use tempfile::Builder;

#[derive(Debug, Clone, Copy)]
pub struct Client;

impl Client {
    pub fn mkdir_temp(&self, dir: &str, pattern: &str) -> io::Result<PathBuf> {
        Builder::new().prefix(pattern).tempdir_in(dir).map(|d| d.into_path())
    }
    pub fn read_file(&self, name: &str) -> io::Result<Vec<u8>> {
        fs::read(name)
    }
    pub fn write_file(&self, name: &str, contents: &[u8]) -> io::Result<()> {
        fs::OpenOptions::new().write(true).create(true).mode(0o220).open(name)?.write_all(contents)
    }
    pub fn remove_all(&self, path: &str) -> io::Result<()> {
        fs::remove_dir_all(path)
    }
    pub fn read_dir(&self, dirname: &str) -> io::Result<Vec<fs::DirEntry>> {
        let mut entries: Vec<_> = fs::read_dir(dirname)?.collect::<Result<_, _>>()?;
        entries.sort_by_key(|e| e.file_name());
        Ok(entries)
    }
}
