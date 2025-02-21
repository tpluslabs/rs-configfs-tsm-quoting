use std::io::{self, ErrorKind};
use anyhow::Result;
use crate::tsm::{parse_tsm_path, read_uint64_file, TsmPath, REPORT_SUBSYSTEM, REPORT_SUBSYSTEM_PATH};
use crate::client::Client;

#[derive(Debug, Default)]
pub struct Privilege {
    pub level: u32,
}

#[derive(Debug)]
pub struct OpenReport {
    pub in_blob: Vec<u8>,
    pub privilege: Option<Privilege>,
    pub get_aux_blob: bool,
    pub service_provider: String,
    pub service_guid: String,
    pub service_manifest_version: String,
    pub entry: TsmPath,
    pub expected_generation: u64,
    pub client: Client,
}

#[derive(Debug)]
pub struct Request {
    pub in_blob: Vec<u8>,
    pub privilege: Option<Privilege>,
    pub get_aux_blob: bool,
}

#[derive(Debug, Default)]
pub struct Response {
    pub aux_blob: Option<Vec<u8>>,
    pub out_blob: Vec<u8>,
    pub provider: String,
    pub manifest_blob: Option<Vec<u8>>,
}


#[derive(Debug, thiserror::Error)]
pub enum ReportError {
    #[error("Error while generating")]
    GenerationErr {
        got: u64,
        want: u64,
        attribute: String
    }
}

impl OpenReport {
    pub fn attribute(&self, subtree: &str) -> String {
        let mut a = self.entry.clone();
        a.attribute = Some(subtree.to_string());
        a.to_string()
    }

    pub fn write_option(&mut self, subtree: &str, data: &[u8]) -> io::Result<()> {
        let path = self.attribute(subtree);
        self.client.write_file(&path, data).map_err(|e| {
            io::Error::new(
                io::ErrorKind::Other,
                format!("could not write report {}: {}", subtree, e),
            )
        })?;
        self.expected_generation += 1;
        Ok(())
    }

    pub fn read_option(&self, subtree: &str) -> Result<Vec<u8>> {
        let path = self.attribute(subtree);
        let data = self.client.read_file(&path).map_err(|e| {
            io::Error::new(ErrorKind::Other, format!("could not read report property {:?}: {}", subtree, e))
        })?;
        let generation_path = self.attribute("generation");
        let got_generation = read_uint64_file(&self.client, &generation_path)?;
        if got_generation != self.expected_generation {
            return Err(anyhow::anyhow!(ReportError::GenerationErr {
                    got: got_generation,
                    want: self.expected_generation,
                    attribute: subtree.to_string(),
                }).into());
        }
        Ok(data)
    }

    pub fn get(&mut self) -> Result<Response> {
        let in_blob = self.in_blob.clone();
        self.write_option("inblob", &in_blob)?;

        if let Some(privilege) = &self.privilege {
            let priv_str = privilege.level.to_string();
            self.write_option("privlevel", priv_str.as_bytes())?;
        }

        if !self.service_provider.is_empty() {
            let sp = self.service_provider.clone();
            self.write_option("service_provider", sp.as_bytes())?;
        }

        if !self.service_guid.is_empty() {
            let sg = self.service_guid.clone();
            self.write_option("service_guid", sg.as_bytes())?;
        }

        if !self.service_manifest_version.is_empty() {
            let smv = self.service_manifest_version.clone();
            self.write_option("service_manifest_version", smv.as_bytes())?;
        }

        let mut resp = Response::default();

        if self.get_aux_blob {
            resp.aux_blob = Some(self.read_option("auxblob").map_err(|e| {
                io::Error::new(ErrorKind::Other, format!("could not read report auxblob: {}", e))
            })?);
        }
        resp.out_blob = self.read_option("outblob").map_err(|e| {
            io::Error::new(ErrorKind::Other, format!("could not read report outblob: {}", e))
        })?;
        let provider_data = self.read_option("provider")?;
        resp.provider = String::from_utf8(provider_data).unwrap_or_default();
        if !self.service_provider.is_empty() {
            resp.manifest_blob = Some(self.read_option("manifestblob").map_err(|e| {
                io::Error::new(ErrorKind::Other, format!("could not read report manifestblob: {}", e))
            })?);
        }
        Ok(resp)
    }
}

pub fn unsafe_wrap(client: Client, entry_path: &str) -> io::Result<OpenReport> {
    let p = parse_tsm_path(entry_path)?;
    let entry = TsmPath {
        subsystem: REPORT_SUBSYSTEM.to_string(),
        entry: p.entry,
        attribute: None,
    };

    let mut open_report = OpenReport {
        client: client.clone(),
        entry,
        in_blob: Vec::new(),
        privilege: None,
        get_aux_blob: false,
        service_provider: String::new(),
        service_guid: String::new(),
        service_manifest_version: String::new(),
        expected_generation: 0,
    };

    let generation_attr = open_report.attribute("generation");
    open_report.expected_generation = read_uint64_file(&client, &generation_attr)?;
    Ok(open_report)
}

pub fn create_open_report(client: Client) -> io::Result<OpenReport> {
    let entry = client.mkdir_temp(REPORT_SUBSYSTEM_PATH, "entry")
        .map_err(|e| io::Error::new(io::ErrorKind::Other, format!("could not create report entry in configfs: {}", e)))?;
    
    println!("created temp dir {}", entry.to_str().unwrap());
    unsafe_wrap(client, entry.to_str().unwrap())
}

pub fn create(client: Client, req: Request) -> io::Result<OpenReport> {
    let mut r = create_open_report(client)?;
    r.in_blob = req.in_blob;
    r.privilege = req.privilege;
    r.get_aux_blob = req.get_aux_blob;
    
    Ok(r)
}
