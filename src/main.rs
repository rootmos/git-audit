use std::time::Duration;

#[macro_use] extern crate serde_derive;
extern crate serde_json;
extern crate tokio_core;
extern crate env_logger;
extern crate dirs;
extern crate config;
extern crate hex;
extern crate secp256k1;
extern crate ethabi;
extern crate primitive_types;

extern crate git2;
use git2::{Repository, Signature};

extern crate clap;
use clap::{Arg, App, SubCommand};

extern crate ethereum_tx_sign;
use ethereum_tx_sign::RawTransaction;

extern crate web3;
use web3::types::{Bytes, H160, H256, U256};
use web3::futures::Future;
use web3::confirm::send_raw_transaction_with_confirmation;

mod settings;
use settings::Settings;

fn private_key(s: &Settings) -> (H160, H256) {
    let raw = hex::decode(s.ethereum_private_key()).unwrap();
    let sk = secp256k1::SecretKey::from_slice(&raw).unwrap();
    let pk = secp256k1::PublicKey::from_secret_key(&secp256k1::Secp256k1::new(), &sk);
    let a = &tiny_keccak::keccak256(&pk.serialize_uncompressed()[1..])[12..];
    log::debug!("using address: {}", hex::encode(a));
    (H160::from_slice(a), H256::from_slice(&raw))
}

fn main() {
    env_logger::init();

    let matches = App::new("git-audit")
        .version("0.1.0")
        .author("Gustav Behm <me@rootmos.io>")
        .arg(Arg::with_name("global-config").long("global-config").short("g").takes_value(true))
        .subcommand(SubCommand::with_name("init")
            .arg(Arg::with_name("no-commit").long("no-commit"))
        )
        .subcommand(SubCommand::with_name("anchor"))
        .subcommand(SubCommand::with_name("validate"))
        .get_matches();

    let mut settings = Settings::new(matches.value_of("global-config")).unwrap();

    let mut el = tokio_core::reactor::Core::new().unwrap();
    let t = web3::transports::Http::with_event_loop(settings.ethereum_rpc_target(),
                                                    &el.handle(), 1).unwrap();
    let web3 = web3::Web3::new(t);

    if let Some(matches) = matches.subcommand_matches("init") {
        let r = Repository::open(".").unwrap();

        let (a, sk) = private_key(&settings);

        let code = hex::decode(include_str!("../build/evm/GitAudit.bin")).unwrap();
        let abi_json = include_str!("../build/evm/GitAudit.abi");

        let tx = web3.eth().gas_price().then(|gp| {
            web3.eth().transaction_count(a, None).then(|n| {
                let tx = RawTransaction {
                    nonce: n.unwrap(),
                    to: None,
                    value: U256::from(0),
                    gas_price: gp.unwrap(),
                    gas: U256::from(1000000), // TODO: estimate gas
                    data: code,
                };
                let stx = tx.sign(&sk, &settings.ethereum_chain_id());
                send_raw_transaction_with_confirmation(web3.transport(), Bytes::from(stx), Duration::new(1, 0), 0)
            }).map(|r| r.contract_address.unwrap())
        });

        let txh = el.run(tx).unwrap();
        log::info!("deployed contract: {:?}", txh);
        settings.set_contract(&hex::encode(txh.as_bytes()), abi_json);
        settings.write_repository_settings();

        if ! matches.is_present("no-commit") {

            let p = if r.is_empty().unwrap() { None } else {
                Some(r.head().unwrap().peel_to_commit().unwrap())
            };

            let mut tb = match &p {
                Some(pc) => r.treebuilder(Some(&pc.tree().unwrap())).unwrap(),
                None => r.treebuilder(None).unwrap(),
            };

            tb.insert(
                settings::repository_config_file(),
                r.blob_path(settings::repository_config_file()).unwrap(),
                0o100644,
            ).unwrap();
            let t = tb.write().unwrap();

            let s = Signature::now("git-audit", "git-audit@rootmos.io").unwrap();
            let c = r.commit(Some("HEAD"), &s, &s,
                "Initializing git-audit",
                &r.find_tree(t).unwrap(),
                &p.iter().collect::<Vec<_>>(),
            ).unwrap();

            log::info!("committed git-audit repository config: {}", c);
        }
    } else if let Some(_matches) = matches.subcommand_matches("anchor") {
        let r = Repository::open(".").unwrap();
        let (a, sk) = private_key(&settings);
        let abi = ethabi::Contract::load(settings.contract_abi_json().as_bytes()).unwrap();
        let f = abi.function("anchor").unwrap();
        let h = r.head().unwrap().target().unwrap();
        log::info!("anchoring HEAD: {}", h);
        let input = f.encode_input(&[ethabi::Token::Uint(primitive_types::U256::from_big_endian(h.as_bytes()))]).unwrap();

        let tx = web3.eth().gas_price().then(|gp| {
            web3.eth().transaction_count(a, None).then(|n| {
                let to = H160::from_slice(&hex::decode(settings.contract_address()).unwrap());
                let tx = RawTransaction {
                    nonce: n.unwrap(),
                    to: Some(to),
                    value: U256::from(0),
                    gas_price: gp.unwrap(),
                    gas: U256::from(1000000), // TODO: estimate gas
                    data: input,
                };
                let stx = tx.sign(&sk, &settings.ethereum_chain_id());
                send_raw_transaction_with_confirmation(web3.transport(), Bytes::from(stx), Duration::new(1, 0), 0)
            })
        });

        let _ = el.run(tx).unwrap();
    }
}
