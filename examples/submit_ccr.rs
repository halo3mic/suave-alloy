use std::str::FromStr;
use eyre::Result;
use alloy::{
    primitives::{Address, Bytes, B256, U256}, 
    providers::{Provider, ProviderBuilder}, 
    rpc::types::eth::TransactionRequest, 
    signers::wallet::LocalWallet,
    network::TransactionBuilder,
};
use suave_alloy::{
    types::{ConfidentialComputeRecord, ConfidentialComputeRequest},
    network::{SuaveProvider, SuaveSigner},
};



#[tokio::main]

async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Args
    let input = Bytes::from_str("0x50723553000000000000000000000000000000000000000000000000000000000000002000000000000000000000000000000000000000000000000000000000000000074554485553445400").unwrap();
    let wallet_address = Address::from_str("0x19E7E376E7C213B7E7e7e46cc70A5dD086DAff2A").unwrap();
    let to_add = Address::from_str("0xc803334c79650708Daf3a3462AC4B48296b1352a").unwrap();
    let gas_price = U256::from_str("0x3c9aca00").unwrap();
    let cinputs = Bytes::new();
    let chain_id = 0x1008c45;
    let gas = 0x0f4240;

    // Create SUAVE provider
    let provider = SuaveProvider::from_url("https://rpc.rigil.suave.flashbots.net");

    // Get nonce and kettle address
    let tx_count: u64 = provider.get_transaction_count(wallet_address, None).await.unwrap().to();
    let kettle = provider.kettle_address().await.unwrap();

    // Create a confidential-compute-request 
    let ccr = ConfidentialComputeRequest::default()
        .with_gas_limit(U256::from(gas))
        .with_to(Some(to_add).into())
        .with_gas_price(gas_price)
        .with_chain_id(chain_id)
        .with_nonce(tx_count)
        .with_input(input)
        .with_confidential_inputs(cinputs)
        .with_kettle_address(kettle);

    // Create a signer
    let wallet: LocalWallet = "0x1111111111111111111111111111111111111111111111111111111111111111".parse().unwrap();    
    let signer = SuaveSigner::from(wallet.clone());
    let provider = ProviderBuilder::default().signer(signer).provider(provider);

    let result = provider.send_transaction(ccr).await.unwrap();
    let tx_hash = B256::from_slice(&result.tx_hash().to_vec());
    let tx_response = provider.get_transaction_by_hash(tx_hash).await.unwrap();

    println!("{tx_response:#?}");

    let price = U256::try_from_be_slice(&tx_response.confidential_compute_result.to_vec()).unwrap();
    println!("Price: {:?}", price.wrapping_to::<u128>());

    Ok(())
}
