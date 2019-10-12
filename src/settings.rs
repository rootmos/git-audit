use config::{Config, ConfigError};

use std::path::Path;
use std::fs;
use std::io::prelude::*;

#[derive(Debug, Deserialize, Serialize)]
struct Ethereum {
    private_key: Option<String>,
    rpc_target: Option<String>,
    chain_id: Option<u8>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
struct Contract {
    address: String,
    abi: serde_json::Value,
}

#[derive(Debug, Deserialize, Serialize)]
struct SettingsRoot {
    ethereum: Option<Ethereum>,
    contract: Option<Contract>,
}

pub struct Settings {
    repository: SettingsRoot,
    merged: SettingsRoot,
}

pub const REPO_CONFIG_FILE: &str = ".git-audit.json";

impl Settings {
    pub fn new(mgc: Option<&str>) -> Result<Self, ConfigError> {
        let gc_pb = mgc.map(|s| Path::new(s).to_path_buf()).unwrap_or_else(
            || dirs::config_dir().unwrap().join("git-audit.json")
        );
        let gp = gc_pb.as_path().to_str().unwrap();

        let mut g = Config::default();
        g.merge(config::File::new(gp, config::FileFormat::Json))?;

        let mut r = Config::default();
        r.merge(config::File::new(REPO_CONFIG_FILE, config::FileFormat::Json).required(false))?;

        let mut m = Config::new();
        m.merge(g.to_owned())?.merge(r.to_owned())?;

        Ok(Settings { repository: r.try_into()?, merged: m.try_into()? })
    }

    pub fn ethereum_rpc_target(&self) -> &str {
        self.merged.ethereum.as_ref().and_then(|e| e.rpc_target.as_ref()).unwrap()
    }

    pub fn ethereum_private_key(&self) -> &str {
        self.merged.ethereum.as_ref().and_then(|e| e.private_key.as_ref()).unwrap()
    }

    pub fn ethereum_chain_id(&self) -> &u8 {
        self.merged.ethereum.as_ref().and_then(|e| e.chain_id.as_ref()).unwrap()
    }

    pub fn set_contract(&mut self, address: &str, abi: &str) -> &Self {
        let abi_j: serde_json::Value = serde_json::from_str(abi).unwrap();
        self.repository.contract = Some(Contract {
            address: address.to_owned(), abi: abi_j,
        });
        self.merged.contract = self.repository.contract.clone();
        self
    }

    pub fn write_repository_settings(self) -> () {
        let s = serde_json::to_string(&self.repository).unwrap();
        match fs::OpenOptions::new().write(true).create_new(true).open(REPO_CONFIG_FILE) {
            Ok(mut f) => f.write_all(s.as_bytes()).unwrap(),
            Err(e) => panic!("{}", e)
        }
    }
}
