use crate::client::Client;
use const_format::concatcp;
use std::fs;
use std::io::{self, ErrorKind};
use std::num::ParseIntError;
use std::path::Path;
pub const TSM_PREFIX: &str = "/sys/kernel/config/tsm";
pub const REPORT_SUBSYSTEM: &str = "report";
pub const REPORT_SUBSYSTEM_PATH: &str = concatcp!(TSM_PREFIX, "/", REPORT_SUBSYSTEM);

#[derive(Debug, Clone)]
pub struct TsmPath {
    pub subsystem: String,
    pub entry: String,
    pub attribute: Option<String>,
}

impl TsmPath {
    pub fn to_string(&self) -> String {
        match &self.attribute {
            Some(attr) => format!("{}/{}/{}/{}", TSM_PREFIX, self.subsystem, self.entry, attr),
            None => format!("{}/{}/{}", TSM_PREFIX, self.subsystem, self.entry),
        }
    }

    pub fn with_attr(mut self, attr: &str) -> Self {
        self.attribute = Some(attr.to_string());
        self
    }
}

fn kstrtouint(data: &[u8], base: u32, _bits: u32) -> Result<u64, ParseIntError> {
    let s = String::from_utf8_lossy(data).trim().to_string();
    u64::from_str_radix(&s, base)
}

pub fn read_uint64_file(client: &Client, p: &str) -> io::Result<u64> {
    let data = client.read_file(p)?;
    kstrtouint(&data, 10, 64)
        .map_err(|e| io::Error::new(io::ErrorKind::Other, format!("could not read {}: {}", p, e)))
}

pub fn parse_tsm_path(filepath: &str) -> io::Result<TsmPath> {
    let s = Path::new(filepath)
        .to_str()
        .ok_or_else(|| io::Error::new(ErrorKind::InvalidInput, "invalid UTF-8"))?;
    if !s.starts_with(TSM_PREFIX) {
        return Err(io::Error::new(
            ErrorKind::InvalidInput,
            format!("{:?} does not begin with {:?}", s, TSM_PREFIX),
        ));
    }
    let rest = s[TSM_PREFIX.len()..].trim_start_matches('/');
    if rest.is_empty() {
        return Err(io::Error::new(
            ErrorKind::InvalidInput,
            format!("{:?} does not contain a subsystem", s),
        ));
    }
    let parts: Vec<&str> = rest.split('/').collect();
    match parts.len() {
        1 => Ok(TsmPath {
            subsystem: parts[0].to_string(),
            entry: String::new(),
            attribute: None,
        }),
        2 => Ok(TsmPath {
            subsystem: parts[0].to_string(),
            entry: parts[1].to_string(),
            attribute: None,
        }),
        3 => Ok(TsmPath {
            subsystem: parts[0].to_string(),
            entry: parts[1].to_string(),
            attribute: Some(parts[2].to_string()),
        }),
        _ => Err(io::Error::new(
            ErrorKind::InvalidInput,
            format!(
                "{:?} suffix expected to be of form subsystem[/entry[/attribute]]",
                rest
            ),
        )),
    }
}

pub fn make_client() -> io::Result<Client> {
    let check = Path::new(REPORT_SUBSYSTEM_PATH);
    let metadata = fs::metadata(check)?;
    if !metadata.is_dir() {
        return Err(io::Error::new(
            io::ErrorKind::Other,
            format!("expected {} to be a directory", check.display()),
        ));
    }
    Ok(Client)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_single_component() {
        let input = "/sys/kernel/config/tsm/report";
        let tsmpath = parse_tsm_path(input).unwrap();
        assert_eq!(tsmpath.subsystem, "report");
        assert_eq!(tsmpath.entry, "");
        assert!(tsmpath.attribute.is_none());
    }

    #[test]
    fn parse_two_components() {
        let input = "/sys/kernel/config/tsm/report/entrytest";
        let tsmpath = parse_tsm_path(input).unwrap();
        assert_eq!(tsmpath.subsystem, "report");
        assert_eq!(tsmpath.entry, "entrytest");
        assert!(tsmpath.attribute.is_none());
    }

    #[test]
    fn parse_three_components() {
        let input = "/sys/kernel/config/tsm/report/entrytest/generation";
        let tsmpath = parse_tsm_path(input).unwrap();
        assert_eq!(tsmpath.subsystem, "report");
        assert_eq!(tsmpath.entry, "entrytest");
        assert_eq!(tsmpath.attribute.unwrap(), "generation");
    }

    #[test]
    fn error_on_empty_rest() {
        let input = "/sys/kernel/config/tsm/";
        assert!(parse_tsm_path(input).is_err());
    }

    #[test]
    fn error_on_invalid_prefix() {
        let input = "/invalid/tsm/path";
        assert!(parse_tsm_path(input).is_err());
    }

    #[test]
    fn error_on_too_many_components() {
        let input = "/sys/kernel/config/tsm/report/entrytest/generation/what";
        assert!(parse_tsm_path(input).is_err());
    }
}
