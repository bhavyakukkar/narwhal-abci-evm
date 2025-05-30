use alloy::{
    consensus::{SignableTransaction, TxEip7702, TxEnvelope},
    network::{TransactionResponse as _, TxSigner},
    primitives::Address,
    rpc::types::Transaction,
    signers::local::{coins_bip39::English, MnemonicBuilder},
    sol,
    sol_types::SolConstructor,
};
use clap::Parser;
use evm_abci::types::{Query, QueryResponse};
use foundry_evm::revm::primitives::{ExecutionResult, Output};
use tokio::time::{sleep, Duration};

const HOST: &str = "http://127.0.0.1:3003";

#[derive(Debug, Clone, Parser)]
struct Args {
    #[clap(default_value = "0.0.0.0:26658")]
    host: String,
    #[clap(long, short)]
    demo: bool,
}

sol!(
    #[allow(missing_docs)]
    #[sol(rpc)]
    Lock,
    "../contracts/artifacts/contracts/Lock.sol/Lock.json" // "../contracts/artifacts/TestERC20.json"
);

async fn send_tx/* <S: Into<String>> */(host: &str, tx: &str) -> eyre::Result<()> {
    let client = reqwest::Client::new();
    client
        .get(format!("{}/broadcast_tx", host))
        .query(&[("tx", tx)])
        .send()
        .await?;
    Ok(())
}

async fn send_query(host: &str, query: Query) -> eyre::Result<QueryResponse> {
    let query = serde_json::to_string(&query)?;
    let client = reqwest::Client::new();
    let res = client
        .get(format!("{}/abci_query", host))
        .query(&[("data", query), ("path", "".to_string())])
        .send()
        .await?;
    let val = res.bytes().await?;
    Ok(serde_json::from_slice(&val)?)
}

#[tokio::main]
async fn main() -> eyre::Result<()> {
    let _args = Args::parse();

    let signer = MnemonicBuilder::<English>::default()
        .phrase("test test test test test test test test test test test junk")
        .build()?;

    // let tx = TransactionRequest::default()
    //     .gas_limit(2_000_000)
    //     .from(address!("04cb007880750f76d939393e596c003263e57e14"))
    //     .with_deploy_code(Lock::DEPLOYED_BYTECODE.clone());
    // .build_typed_tx()
    // .unwrap()

    /* {
      chainId: 31337,
      nonce: 0,
      gasLimit: 2_000_000,
      maxFeePerGas: 875_000_000,
      maxPriorityFeePerGas: 10_000,
      // to: TxKind::Create,
      to: ZeroAddress,
      value: 0,
      accessList: [],
      authorizationList: [],
      data: factory.bytecode,
    } */

    let bytecode: &[u8] = &Lock::BYTECODE;
    let mut deploy_code: Vec<_> = bytecode.into();
    deploy_code.extend_from_slice(
        Lock::constructorCall::new((8.try_into().unwrap(),))
            .abi_encode()
            .as_slice(),
    );
    let input = deploy_code.into();

    let mut unsigned_tx = TxEip7702 {
        chain_id: 31337,
        nonce: 0,
        gas_limit: 5_000_000,
        max_fee_per_gas: 1_000_000_000,
        max_priority_fee_per_gas: 10_000,
        // to: TxKind::Create,
        to: Address::ZERO,
        value: 0.try_into().unwrap(),
        access_list: Default::default(),
        authorization_list: Vec::new(),
        input,
    };
    let signature = signer.sign_transaction(&mut unsigned_tx).await?;
    let signed_tx = unsigned_tx.into_signed(signature);

    // let tx = TxEnvelope::Legacy(signed_tx);
    let hash = signed_tx.hash().to_owned();
    println!("{hash}");

    // Send tx to deploy contract
    let tx = Transaction {
        inner: TxEnvelope::Eip7702(signed_tx),
        block_hash: None,
        block_number: None,
        transaction_index: None,
        from: signer.address(),
    };
    let tx_hash = tx.tx_hash();
    println!("hash: {}", tx_hash);

    let signer_balance = send_query(HOST, Query::Balance(signer.address())).await?.as_balance();
    println!("Signer's balance: {}", signer_balance);

    send_tx(HOST, &serde_json::to_string(&tx)?).await?;
    println!("Sent tx");

    // Poll tx receipt
    let query = Query::GetTransactionReceipt(tx_hash);
    let query = serde_json::to_string(&query)?;
    let contract_address = loop {
        let client = reqwest::Client::new();
        let res = client
            .get(format!("{HOST}/abci_query"))
            .query(&[("data", query.clone()), ("path", "".to_string())])
            .send()
            .await?;
        let val = res.bytes().await?;
        let val: QueryResponse = serde_json::from_slice(&val)?;
        let val = val.as_receipt();

        // Transaction now known by evm-app
        if let Some(result) = val {
            println!("Transaction receipt: {:#?}", result);
            break match result {
                ExecutionResult::Success {
                    output: Output::Create(_, Some(address)),
                    ..
                } => address.clone(),
                ExecutionResult::Success {
                    output: Output::Create(_, None),
                    ..
                } => eyre::bail!("success output is `Create` but no address was created"),
                ExecutionResult::Success { output: _, .. } => {
                    eyre::bail!("success output is not `Create`")
                }
                ExecutionResult::Revert { .. } => eyre::bail!("transaction reverted"),
                ExecutionResult::Halt { .. } => eyre::bail!("transaction halted"),
            };
        }
        sleep(Duration::from_secs(1)).await;
    };

    // Get deployed contract's info
    println!("Deploy contract info: {:#?}", send_query(HOST, Query::GetAccount(contract_address)).await?.as_account_info());

    Ok(())
}
