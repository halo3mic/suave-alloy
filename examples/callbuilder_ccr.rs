use std::str::FromStr;
use eyre::Result;
use alloy::{
    providers::{Provider, ProviderBuilder},
    primitives::{Address, Bytes}, 
    signers::wallet::LocalWallet,
    sol
};
use suave_alloy::prelude::*;


#[tokio::main]

async fn main() -> Result<()> {
    sol! {
        #[sol(rpc)]
        contract BinanceOracle {
            #[derive(Debug)]
            function queryLatestPrice(string memory ticker) public view returns (uint price);
        }
    }
    let boracle_add = Address::from_str("0xc803334c79650708Daf3a3462AC4B48296b1352a")?;
    let pk = "0x1111111111111111111111111111111111111111111111111111111111111111";
    let rpc_url = "https://rpc.rigil.suave.flashbots.net";
    let ticker = String::from("ETHUSDT");
    let gas = 0x0f4240; // Estimate gas doesn't work well with MEVM

    // Create SUAVE signer-provider
    let wallet: LocalWallet = pk.parse()?;    
    let provider = ProviderBuilder::<_, _, SuaveNetwork>::default()
        .with_recommended_fillers()
        .filler(KettleFiller::default())
        .signer(SuaveSigner::new(wallet.clone()))
        .on_provider(SuaveProvider::try_from(rpc_url)?);

    // Create call builder
    let contract = BinanceOracle::new(boracle_add, &provider);
    let call_builder = contract.queryLatestPrice(ticker)
        .with_cinput(Bytes::new())
        .gas(gas);

    // Send tx
    let pending_tx = call_builder.send().await?;
    let confidential_request_res = provider.get_transaction_by_hash(*pending_tx.tx_hash()).await?;
    println!("{:#?}", confidential_request_res);

    Ok(())
}
