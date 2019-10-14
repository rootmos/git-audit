use std::time::Duration;

#[macro_use] extern crate mdo;
extern crate mdo_future;
use mdo_future::future::{bind, ret};

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
use clap::{Arg, App, SubCommand, ArgMatches};

extern crate ethereum_tx_sign;
use ethereum_tx_sign::RawTransaction;

extern crate web3;
use web3::types::{Bytes, H160, H256, U256};
use web3::futures::Future;
use web3::confirm::send_raw_transaction_with_confirmation;

mod settings;
use settings::Settings;

struct Context<'a, T: web3::Transport> {
    settings: &'a mut Settings,
    web3: &'a web3::Web3<T>,
    repo: &'a Repository,
}

impl <T: web3:: Transport> Context<'_, T> {
    fn abi(self: &Self) -> ethabi::Contract {
        ethabi::Contract::load(self.settings.contract_abi_json().as_bytes()).unwrap()
    }

    fn contract_address_h160(self: &Self) -> H160 {
        H160::from_slice(&hex::decode(self.settings.contract_address()).unwrap())
    }

    fn private_key(self: &Self) -> (H160, H256) {
        let raw = hex::decode(self.settings.ethereum_private_key()).unwrap();
        let sk = secp256k1::SecretKey::from_slice(&raw).unwrap();
        let pk = secp256k1::PublicKey::from_secret_key(&secp256k1::Secp256k1::new(), &sk);
        let a = &tiny_keccak::keccak256(&pk.serialize_uncompressed()[1..])[12..];
        log::debug!("using address: {}", hex::encode(a));
        (H160::from_slice(a), H256::from_slice(&raw))
    }
}

fn init<'a, T: web3::Transport>(
    matches: &'a ArgMatches<'a>,
    ctx: &'a mut Context<'a, T>,
) -> Box<dyn Future<Item=i32, Error=web3::Error> + 'a> {
    let (a, sk) = ctx.private_key();

    let code = hex::decode(include_str!("../build/evm/GitAudit.bin")).unwrap();
    let abi_json = include_str!("../build/evm/GitAudit.abi");

    Box::new(mdo! {
        gp =<< ctx.web3.eth().gas_price();
        n =<< ctx.web3.eth().transaction_count(a, None);
        let tx = RawTransaction {
            nonce: n,
            to: None,
            value: U256::from(0),
            gas_price: gp,
            gas: U256::from(1000000), // TODO: estimate gas
            data: code,
        }.sign(&sk, &ctx.settings.ethereum_chain_id());
        rc =<< send_raw_transaction_with_confirmation(
            ctx.web3.transport(), Bytes::from(tx), Duration::new(1, 0), 0);
        let txh = rc.contract_address.unwrap();
        let () = log::info!("deployed contract: {:?}", txh);
        let _ = ctx.settings.set_contract(&hex::encode(txh.as_bytes()), abi_json);
        let () = ctx.settings.write_repository_settings();
        let () = if ! matches.is_present("no-commit") {
            let p = if ctx.repo.is_empty().unwrap() { None } else {
                Some(ctx.repo.head().unwrap().peel_to_commit().unwrap())
            };

            let mut tb = match &p {
                Some(pc) => ctx.repo.treebuilder(Some(&pc.tree().unwrap())).unwrap(),
                None => ctx.repo.treebuilder(None).unwrap(),
            };

            tb.insert(
                settings::repository_config_file(),
                ctx.repo.blob_path(settings::repository_config_file()).unwrap(),
                0o100644,
            ).unwrap();
            let t = tb.write().unwrap();

            let s = Signature::now("git-audit", "git-audit@rootmos.io").unwrap();
            let c = ctx.repo.commit(Some("HEAD"), &s, &s,
                "Initializing git-audit",
                &ctx.repo.find_tree(t).unwrap(),
                &p.iter().collect::<Vec<_>>(),
            ).unwrap();

            log::info!("committed git-audit repository config: {}", c);
        };
        ret ret(0)
    })
}

fn anchor<'a, T: web3::Transport>(
    _matches: &'a ArgMatches<'a>,
    ctx: &'a Context<'a, T>,
) -> Box<dyn Future<Item=i32, Error=web3::Error> + 'a> {
    let (a, sk) = ctx.private_key();
    let f = ctx.abi().function("anchor").unwrap().to_owned();
    let h = ctx.repo.head().unwrap().target().unwrap();
    log::info!("anchoring HEAD: {}", h);
    let input = f.encode_input(&[ethabi::Token::Uint(primitive_types::U256::from_big_endian(h.as_bytes()))]).unwrap();

    Box::new(mdo! {
        gp =<< ctx.web3.eth().gas_price();
        n =<< ctx.web3.eth().transaction_count(a, None);
        let tx = RawTransaction {
            nonce: n,
            to: Some(ctx.contract_address_h160()),
            value: U256::from(0),
            gas_price: gp,
            gas: U256::from(1000000), // TODO: estimate gas
            data: input,
        }.sign(&sk, &ctx.settings.ethereum_chain_id());
        _ =<< send_raw_transaction_with_confirmation(
            ctx.web3.transport(), Bytes::from(tx), Duration::new(1, 0), 0);
        ret ret(0)
    })
}

fn validate<'a, T: web3::Transport>(
    _matches: &'a ArgMatches<'a>,
    ctx: &'a Context<'a, T>,
) -> Box<dyn Future<Item=i32, Error=web3::Error> + 'a> {
    let f = ctx.abi().function("commits").unwrap().to_owned();
    let data = Some(Bytes::from(f.encode_input(&[]).unwrap()));
    let cr = web3::types::CallRequest {
        from: None,
        to: ctx.contract_address_h160(),
        gas: None,
        gas_price: None,
        value: None,
        data,
    };

    Box::new(mdo! {
        Bytes(rsp) =<< ctx.web3.eth().call(cr, None);
        let cs_contract = match &f.decode_output(&rsp).unwrap()[..] {
            [ethabi::Token::Array(ts)] => {
                let mut cs = vec![];
                for t in ts.iter() {
                    if let ethabi::Token::Uint(ui) = t {
                        let mut buf = vec![0; 32];
                        ui.to_big_endian(&mut buf);
                        cs.push(buf.split_off(12))
                    }
                }
                cs
            },
            _ => panic!("unexpected return types from contract function"),
        };
        let cs_repo = {
            let mut w = ctx.repo.revwalk().unwrap();
            w.push_head().unwrap();
            w.map(|i| i.unwrap().as_bytes().to_owned()).collect::<Vec<_>>()
        };
        let () = if log::log_enabled!(log::Level::Debug) {
            for c in &cs_contract { log::debug!("contract commit: {}", hex::encode(c)) }
            for c in &cs_repo { log::debug!("repository commit: {}", hex::encode(c)) }
        };
        ret ret({
            let mut good = 0;
            let mut bad = 0;

            for c in cs_contract.iter() {
                if cs_repo.contains(&c) {
                    good += 1;
                    log::debug!("commit present in repository: {}", hex::encode(c))
                } else {
                    bad += 1;
                    log::warn!("commit not present in repository: {}", hex::encode(c))
                };
            }

            log::info!("validation result: good={} bad={}", good, bad);

            if bad > 0 { 1 } else { 0 }
        })
    })
}

fn main() {
    env_logger::init();

    let matches = &App::new("git-audit")
        .version("0.1.0")
        .author("Gustav Behm <me@rootmos.io>")
        .arg(Arg::with_name("global-config").long("global-config").short("g").takes_value(true))
        .subcommand(SubCommand::with_name("init")
            .arg(Arg::with_name("no-commit").long("no-commit"))
        )
        .subcommand(SubCommand::with_name("anchor"))
        .subcommand(SubCommand::with_name("validate"))
        .get_matches();

    let settings = &mut Settings::new(matches.value_of("global-config")).unwrap();
    let mut el = tokio_core::reactor::Core::new().unwrap();
    let t = web3::transports::Http::with_event_loop(settings.ethereum_rpc_target(),
                                                    &el.handle(), 1).unwrap();
    let web3 = &web3::Web3::new(t);
    let repo = &Repository::open(".").unwrap();
    let mut ctx = Context { settings, web3, repo };

    let f = if let Some(matches) = matches.subcommand_matches("init") {
        init(matches, &mut ctx)
    } else if let Some(matches) = matches.subcommand_matches("anchor") {
        anchor(matches, &ctx)
    } else if let Some(matches) = matches.subcommand_matches("validate") {
        validate(matches, &ctx)
    } else {
        panic!("invalid subcommand match")
    };

    let ec = el.run(f).unwrap();
    std::process::exit(ec);
}
