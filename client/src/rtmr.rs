use crate::{
    client::Client,
    tsm::{read_uint64_file, TsmPath, TSM_PREFIX},
};
use sha2::{Digest, Sha384};
use std::{
    io::{self, ErrorKind},
    num::ParseIntError,
    path::PathBuf,
};

pub const RTMR_SUBSYSTEM: &str = "rtmr";

#[derive(thiserror::Error, Debug)]
pub enum ExtendError {
    #[error("invalid digest length: expected {expected}, got {actual}")]
    InvalidDigestLength { expected: usize, actual: usize },
    #[error("invalid RTMR index {0}")]
    InvalidIndex(isize),
    #[error("IO error: {0}")]
    Io(#[from] io::Error),
    #[error("parse error: {0}")]
    Parse(#[from] ParseIntError),
}

pub fn extend_digest(client: &Client, rtmr: isize, digest: &[u8]) -> Result<(), ExtendError> {
    let expected = Sha384::output_size();
    if digest.len() != expected {
        return Err(ExtendError::InvalidDigestLength {
            expected,
            actual: digest.len(),
        });
    }
    if rtmr < 0 {
        return Err(ExtendError::InvalidIndex(rtmr));
    }
    let entry = get_rtmr_interface(client, rtmr as usize)?;
    entry.extend_digest(digest)
}

struct RtmrEntry {
    client: Client,
    #[allow(dead_code)]
    index: usize,
    path: TsmPath,
}

fn get_rtmr_interface(client: &Client, index: usize) -> Result<RtmrEntry, ExtendError> {
    if let Some(e) = search_rtmr_interface(client, index)? {
        Ok(e)
    } else {
        create_rtmr_interface(client, index)
    }
}

fn search_rtmr_interface(client: &Client, index: usize) -> Result<Option<RtmrEntry>, ExtendError> {
    let entries = client.read_dir(&format!("{}/{}/", TSM_PREFIX, RTMR_SUBSYSTEM))?;
    for dir in entries {
        if dir.file_type()?.is_dir() {
            let name = dir.file_name().to_string_lossy().into_owned();
            let entry = TsmPath {
                subsystem: RTMR_SUBSYSTEM.to_string(),
                entry: name.clone(),
                attribute: None,
            };
            let path_index = entry.clone().with_attr("index").to_string();
            if read_uint64_file(client, &path_index)? as usize == index {
                return Ok(Some(RtmrEntry {
                    client: client.clone(),
                    index,
                    path: entry,
                }));
            }
        }
    }
    Ok(None)
}

fn create_rtmr_interface(client: &Client, index: usize) -> Result<RtmrEntry, ExtendError> {
    let dir = client.mkdir_temp(
        &format!("{}/{}", TSM_PREFIX, RTMR_SUBSYSTEM),
        &format!("rtmr{}-", index),
    )?;
    let entry_name = PathBuf::from(dir)
        .file_name()
        .ok_or_else(|| io::Error::new(ErrorKind::Other, "invalid tempdir name"))?
        .to_string_lossy()
        .to_string();
    let entry = TsmPath {
        subsystem: RTMR_SUBSYSTEM.to_string(),
        entry: entry_name,
        attribute: None,
    };
    let path_index = entry.clone().with_attr("index").to_string();
    client.write_file(&path_index, index.to_string().as_bytes())?;
    Ok(RtmrEntry {
        client: client.clone(),
        index,
        path: entry,
    })
}

impl RtmrEntry {
    fn extend_digest(&self, hash: &[u8]) -> Result<(), ExtendError> {
        let path = self.path.clone().with_attr("digest").to_string();
        self.client.write_file(&path, hash)?;
        Ok(())
    }
}
