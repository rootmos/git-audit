use std::time::Duration;

extern crate log;
use log::{info, warn, debug};

#[macro_use] extern crate mdo;
extern crate mdo_future;
use mdo_future::future::{bind, ret};

#[macro_use] extern crate serde_derive;
extern crate serde_json;

extern crate tokio_core;
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
use web3::types::{Bytes, H160, H256, U256, U64, CallRequest};
use web3::futures::Future;
use web3::confirm::send_raw_transaction_with_confirmation;

mod settings;
use settings::Settings;

mod loggers;
use loggers::UnixSocketLogger;

struct Context<'a, T: web3::Transport> {
    settings: Settings,
    web3: &'a web3::Web3<T>,
    repo: Repository,
}

impl <T: web3:: Transport> Context<'_, T> {
    fn abi(self: &Self) -> ethabi::Contract {
        ethabi::Contract::load(self.settings.contract_abi_json().as_bytes()).unwrap()
    }

    fn contract_address(self: &Self) -> Option<H160> {
        self.settings.contract_address().map(|a| H160::from_slice(&hex::decode(a).unwrap()))
    }

    fn contract_owner(self: &Self) -> Option<H160> {
        self.settings.contract_owner().map(|a| H160::from_slice(&hex::decode(a).unwrap()))
    }

    fn private_key(self: &Self) -> (H160, H256) {
        let raw = hex::decode(self.settings.ethereum_private_key()).unwrap();
        let sk = secp256k1::SecretKey::from_slice(&raw).unwrap();
        let pk = secp256k1::PublicKey::from_secret_key(&secp256k1::Secp256k1::new(), &sk);
        let a = &tiny_keccak::keccak256(&pk.serialize_uncompressed()[1..])[12..];
        debug!("using address: {}", hex::encode(a));
        (H160::from_slice(a), H256::from_slice(&raw))
    }
}

fn init<'a, T: web3::Transport>(
    matches: &'a ArgMatches,
    ctx: &'a mut Context<T>,
) -> Box<dyn Future<Item=i32, Error=web3::Error> + 'a> {
    let (a, sk) = ctx.private_key();

    let code = hex::decode(include_str!("../build/evm/GitAudit.bin")).unwrap();
    let abi_json = include_str!("../build/evm/GitAudit.abi");

    if let Some(ca) = ctx.contract_address() {
        eprintln!("repository is already initialized and anchored to contract: 0x{}", hex::encode(ca));
        return Box::new(ret(1))
    }

    Box::new(mdo! {
        gp =<< ctx.web3.eth().gas_price();
        n =<< ctx.web3.eth().transaction_count(a, None);
        let g0 = 21000 + 32000 + 68 * code.len();
        let tx = RawTransaction {
            nonce: n,
            to: None,
            value: U256::from(0),
            gas_price: gp,
            gas: U256::from(g0 + 123757), // TODO: don't hardcode this value
            data: code,
        }.sign(&sk, &ctx.settings.ethereum_chain_id());
        rc =<< send_raw_transaction_with_confirmation(
            ctx.web3.transport(), Bytes::from(tx), Duration::new(1, 0), 0);
        let txh = rc.contract_address.unwrap();
        let () = info!("deployed contract: {:?}", txh);
        let _ = ctx.settings.set_contract(
            &hex::encode(txh.as_bytes()), &hex::encode(a.as_bytes()), abi_json);
        let wd = ctx.repo.workdir().unwrap(); // TODO: make this work in a bare repo
        let rp = ctx.settings.write_repository_settings(wd);
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
                ctx.repo.blob_path(&rp).unwrap(),
                0o100644,
            ).unwrap();
            let t = tb.write().unwrap();

            let s = Signature::now("git-audit", "git-audit@rootmos.io").unwrap();
            let c = ctx.repo.commit(Some("HEAD"), &s, &s,
                "Initializing git-audit",
                &ctx.repo.find_tree(t).unwrap(),
                &p.iter().collect::<Vec<_>>(),
            ).unwrap();

            info!("committed git-audit repository config: {}", c);
        };
        ret ret(0)
    })
}

fn anchor<'a, T: web3::Transport>(
    _matches: &'a ArgMatches,
    ctx: &'a Context<T>,
) -> Box<dyn Future<Item=i32, Error=web3::Error> + 'a> {
    match ctx.contract_address() {
        None => Box::new(mdo! {
            let () = eprintln!("repository isn't initialized");
            ret ret(1)
        }),
        Some(to) => {
            let (a, sk) = ctx.private_key();
            let f = ctx.abi().function("anchor").unwrap().to_owned();
            let h = ctx.repo.head().unwrap().target().unwrap();
            info!("anchoring HEAD: {}", h);
            let data = f.encode_input(&[
                ethabi::Token::Uint(primitive_types::U256::from_big_endian(h.as_bytes()))
            ]).unwrap();

            Box::new(mdo! {
                gas_price =<< ctx.web3.eth().gas_price();
                gas =<< ctx.web3.eth().estimate_gas(CallRequest {
                    from: ctx.contract_owner(), to, gas: None, gas_price: Some(gas_price), value: None,
                    data: Some(Bytes::from(data.to_owned()))
                }, None);
                let () = debug!("estimated gas_limit for anchor() call: {}", gas);
                n =<< ctx.web3.eth().transaction_count(a, None);
                let tx = RawTransaction { nonce: n, to: Some(to), gas_price, gas, data,
                    value: U256::from(0),
                }.sign(&sk, &ctx.settings.ethereum_chain_id());
                rc =<< send_raw_transaction_with_confirmation(
                    ctx.web3.transport(), Bytes::from(tx), Duration::new(1, 0), 0);
                let () = debug!("anchor transaction call receipt: {:?}", rc);
                ret match rc.status {
                    Some(U64([1])) => ret(0),
                    Some(U64([0])) => {
                        warn!("anchor transaction call failed: {}", hex::encode(rc.transaction_hash));
                        eprintln!("unable to anchor commit in contract: 0x{}", hex::encode(to));
                        ret(1)
                    },
                    Some(U64([s])) => panic!("unexpected status: {}", s),
                    None => panic!("no status returned"),
                }
            })
        },
    }
}

fn validate<'a, T: web3::Transport>(
    _matches: &'a ArgMatches,
    ctx: &'a Context<T>,
) -> Box<dyn Future<Item=i32, Error=web3::Error> + 'a> {
    match ctx.contract_address() {
        None => Box::new(mdo! {
            let () = eprintln!("repository isn't initialized");
            ret ret(1)
        }),
        Some(to) => {
            let f = ctx.abi().function("commits").unwrap().to_owned();
            let data = Some(Bytes::from(f.encode_input(&[]).unwrap()));

            let cr = CallRequest {
                from: None, to, gas: None, gas_price: None, value: None, data,
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
                            debug!("commit present in repository: {}", hex::encode(c))
                        } else {
                            bad += 1;
                            warn!("commit not present in repository: {}", hex::encode(c))
                        };
                    }

                    info!("validation result: good={} bad={}", good, bad);

                    if bad > 0 { 1 } else { 0 }
                })
            })
        },
    }
}

fn run() -> i32 {
    let matches = &App::new("git-audit")
        .version("0.1.0")
        .about("Manages an audit trail for a Git repository by considering it as an Ethereum side-chain")
        .author("Gustav Behm <me@rootmos.io>")
        .arg(Arg::with_name("global-config").long("global-config").short("g").takes_value(true))
        .arg(Arg::with_name("repository").long("repository").short("r").takes_value(true))
        .subcommand(
            SubCommand::with_name("init")
                .about("Deploys a Ethereum smart contract to collect the audit trail")
                .arg(Arg::with_name("no-commit").long("no-commit"))
        )
        .subcommand(
            SubCommand::with_name("anchor")
                .about("Anchors a commit in the audit trail")
        )
        .subcommand(
            SubCommand::with_name("validate")
                .about("Validates the audit trail")
        )
        .get_matches();

    let settings = Settings::new(matches.value_of("global-config")).unwrap();

    if let Some(fp) = settings.log_file_path() {
        let logger = UnixSocketLogger::new(fp).unwrap();
        log::set_boxed_logger(Box::new(logger)).unwrap();
        log::set_max_level(log::LevelFilter::Trace);
    } else {
        env_logger::init();
    };

    let mut el = tokio_core::reactor::Core::new().unwrap();
    let t = web3::transports::Http::with_event_loop(settings.ethereum_rpc_target(),
                                                    &el.handle(), 1).unwrap();
    let web3 = web3::Web3::new(t);
    let repo_path = matches.value_of("repository").unwrap_or(".");
    let repo = match Repository::open(repo_path) {
        Ok(r) => {
            info!("working with repository at: {:?}", r.path());
            r
        },
        Err(ref e) if e.code() == git2::ErrorCode::NotFound
            && e.class() == git2::ErrorClass::Repository => {
            debug!("unable to open repository: {}", e.message());
            eprintln!("unable to open a git repository at: {}", repo_path);
            return 1
        },
        Err(ref e) => panic!("unable to open repository: {}", e),
    };

    let mut ctx = Context { settings, web3: &web3, repo };

    let f = if let Some(matches) = matches.subcommand_matches("init") {
        init(matches, &mut ctx)
    } else if let Some(matches) = matches.subcommand_matches("anchor") {
        anchor(matches, &ctx)
    } else if let Some(matches) = matches.subcommand_matches("validate") {
        validate(matches, &ctx)
    } else {
        panic!("invalid subcommand match")
    };

    el.run(f).unwrap()
}

fn main() -> () {
    std::process::exit(run());
}
