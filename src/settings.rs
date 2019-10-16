use config::{Config, ConfigError};

use log::debug;

use std::path::{Path, PathBuf};
use std::fs;
use std::io::prelude::*;

#[derive(Debug, Deserialize, Serialize)]
struct Ethereum {
    #[serde(skip_serializing)]
    private_key: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    rpc_target: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    chain_id: Option<u8>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
struct Contract {
    address: String,
    owner: String,
    abi: serde_json::Value,
}

#[derive(Debug, Deserialize, Serialize)]
struct Logging {
    #[serde(skip_serializing_if = "Option::is_none")]
    file: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
struct SettingsRoot {
    #[serde(skip_serializing_if = "Option::is_none")]
    ethereum: Option<Ethereum>,
    #[serde(skip_serializing_if = "Option::is_none")]
    logging: Option<Logging>,
    contract: Option<Contract>,
}

pub struct Settings {
    repository: SettingsRoot,
    merged: SettingsRoot,
}

const REPO_CONFIG_FILE: &str = ".git-audit.json";

pub fn repository_config_file() -> &'static Path {
    &Path::new(REPO_CONFIG_FILE)
}

impl Settings {
    pub fn new(mgc: Option<&str>) -> Result<Self, ConfigError> {
        let gp = mgc.map(|s| Path::new(s).to_path_buf()).unwrap_or_else(
            || dirs::config_dir().unwrap().join("git-audit.json")
        );

        let mut g = Config::default();
        g.merge(config::File::from(gp).required(false))?;

        let mut r = Config::default();
        r.merge(config::File::from(repository_config_file()).required(false))?;

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

    pub fn contract_address(&self) -> Option<&str> {
        self.merged.contract.as_ref().map(|c| c.address.as_str())
    }

    pub fn contract_owner(&self) -> Option<&str> {
        self.merged.contract.as_ref().map(|c| c.owner.as_str())
    }

    pub fn contract_abi_json(&self) -> String {
        self.merged.contract.as_ref()
            .and_then(|c| serde_json::to_string(&c.abi).ok()).unwrap()
    }

    pub fn log_file_path(&self) -> Option<&Path> {
        self.merged.logging.as_ref().and_then(|l| l.file.as_ref().map(|s| Path::new(s)))
    }

    pub fn set_contract(&mut self, address: &str, owner: &str, abi: &str) -> &Self {
        let abi_j: serde_json::Value = serde_json::from_str(abi).unwrap();
        self.repository.contract = Some(Contract {
            address: address.to_owned(), owner: owner.to_owned(), abi: abi_j,
        });
        self.merged.contract = self.repository.contract.clone();
        self
    }

    pub fn write_repository_settings(&self, root: &Path) -> PathBuf {
        let s = serde_json::to_string(&self.repository).unwrap();
        let p = root.join(repository_config_file());
        debug!("writing repository settings to: {:?}", p);
        match fs::OpenOptions::new().write(true).create_new(true).open(&p) {
            Ok(mut f) => f.write_all(s.as_bytes()).unwrap(),
            Err(e) => panic!("{}", e)
        };
        p
    }

}
