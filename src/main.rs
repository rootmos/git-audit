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

extern crate clap;
use clap::{Arg, App, SubCommand};

extern crate ethereum_tx_sign;
use ethereum_tx_sign::RawTransaction;

extern crate web3;
use web3::types::{Bytes, H160, H256, U256};
use web3::futures::Future;
use web3::confirm::send_raw_transaction_with_confirmation;

mod settings;

fn main() {
    env_logger::init();

    let matches = App::new("git-audit")
        .version("0.1.0")
        .author("Gustav Behm <me@rootmos.io>")
        .arg(Arg::with_name("global-config").long("global-config").short("g").takes_value(true))
        .subcommand(SubCommand::with_name("init"))
        .get_matches();

    let mut settings = settings::Settings::new(matches.value_of("global-config")).unwrap();

    let code = hex::decode(include_str!("../build/evm/GitAudit.bin")).unwrap();
    let abi_json = include_str!("../build/evm/GitAudit.abi");

    if let Some(_matches) = matches.subcommand_matches("init") {
        let mut el = tokio_core::reactor::Core::new().unwrap();
        let t = web3::transports::Http::with_event_loop(settings.ethereum_rpc_target(),
                                                        &el.handle(), 1).unwrap();
        let web3 = web3::Web3::new(t);

        let raw = hex::decode(settings.ethereum_private_key()).unwrap();
        let sk = secp256k1::SecretKey::from_slice(&raw).unwrap();
        let pk = secp256k1::PublicKey::from_secret_key(&secp256k1::Secp256k1::new(), &sk);
        let a = &tiny_keccak::keccak256(&pk.serialize_uncompressed()[1..])[12..];

        let tx = web3.eth().gas_price().then(|gp| {
            web3.eth().transaction_count(H160::from_slice(a), None).then(|n| {
                let tx = RawTransaction {
                    nonce: n.unwrap(),
                    to: None,
                    value: U256::from(0),
                    gas_price: gp.unwrap(),
                    gas: U256::from(1000000), // TODO: estimate gas
                    data: code,
                };
                let stx = tx.sign(&H256::from_slice(&raw), &settings.ethereum_chain_id());
                send_raw_transaction_with_confirmation(web3.transport(), Bytes::from(stx), Duration::new(1, 0), 0)
            }).map(|r| r.contract_address.unwrap())
        });

        let txh = el.run(tx).unwrap();
        log::info!("deployed contract: {:?}", txh);
        settings.set_contract(&hex::encode(txh.as_bytes()), abi_json);
        settings.write_repository_settings();
    }
}
