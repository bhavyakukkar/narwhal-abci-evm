use abci::async_api::Server;
use std::net::SocketAddr;

use clap::Parser;

mod app {
    use std::sync::Arc;

    use evm_abci::{Consensus, Info, Mempool, Snapshot, State};

    use alloy::primitives::utils::parse_ether;
    use foundry_evm::revm::{
        db::{CacheDB, EmptyDB},
        primitives::AccountInfo,
    };
    use tokio::sync::Mutex;

    pub struct App<Db> {
        pub mempool: Mempool,
        pub snapshot: Snapshot,
        pub consensus: Consensus<Db>,
        pub info: Info<Db>,
    }

    impl App<CacheDB<EmptyDB>> {
        pub fn new() -> Self {
            let mut state = State::default();

            let alloc = [
                "0x04cb007880750f76d939393e596c003263e57e14",
                "0x03a088930a3fb9d59b735a93d31aabc78690f523",
                "0x19c65662e3f9c28059623678e99c6e7cc588cad5",
                "0x61289e27123cf781d760542d6e8486662c2891a0",
                "0x752e6ebc456e0b3a6c8cccc7b1644a6043fb2a57",
                "0xb1642cf1506e96d8d268d07910a11880c269e653",
                "0xe775253c4177946c19d3d3518635b7e62845a6f1",
                "0x0d67e950e63bbf90497fde6b35190a6f9277a424",
                "0x508b46a7150285e9cfbe116e1e392266242af6c9",
                "0x22d8c3984f2c01ebc6ba2b325b266c9a53dba960",
                "0x261B430526BFC7826076EcBaCD59480eC690234b",
                "0x6b91318da72D33CaD5f71502101bCD7563068dFe",
                "0x20dD9Eb4f4A75b9F70631c49a5AaC3c48D392D13",
                "0xCFe4DA2084Db71E83b7833Fb267A6caE459e31dD",
                "0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266",
            ];
            for address in alloc {
                // addr(pk = 78aaa1de82137f31ac551fd8e876a6930aadd51b28c25e8c3420100f8e51d5c6)
                state.db.insert_account_info(
                    address.parse().unwrap(),
                    AccountInfo {
                        balance: parse_ether("1000").unwrap(),
                        ..Default::default()
                    },
                );
            }

            let committed_state = Arc::new(Mutex::new(state.clone()));
            let current_state = Arc::new(Mutex::new(state));

            let consensus = Consensus {
                committed_state: committed_state.clone(),
                current_state,
            };
            let mempool = Mempool::default();
            let info = Info {
                state: committed_state,
            };
            let snapshot = Snapshot::default();

            App {
                consensus,
                mempool,
                info,
                snapshot,
            }
        }
    }
}
use app::App;

#[derive(Debug, Clone, Parser)]
struct Args {
    #[clap(default_value = "0.0.0.0:26658")]
    host: String,
}

use tracing_error::ErrorLayer;

use tracing_subscriber::prelude::*;

/// Initializes a tracing Subscriber for logging
#[allow(dead_code)]
pub fn subscriber() {
    tracing_subscriber::Registry::default()
        // .with(tracing_subscriber::EnvFilter::new("evm-app=trace"))
        .with(ErrorLayer::default())
        .with(tracing_subscriber::fmt::layer())
        .with(tracing_subscriber::EnvFilter::new("warn"))
        .init()
}

#[tokio::main]
async fn main() -> eyre::Result<()> {
    let args = Args::parse();
    subscriber();

    let App {
        consensus,
        mempool,
        info,
        snapshot,
    } = App::new();
    let server = Server::new(consensus, mempool, info, snapshot);

    dbg!(&args.host);
    tracing::trace!("test");
    // let addr = args.host.strip_prefix("http://").unwrap_or(&args.host);
    let addr = args.host.parse::<SocketAddr>().unwrap();

    // let addr = SocketAddr::new(addr, args.port);
    server.run(addr).await?;

    Ok(())
}
