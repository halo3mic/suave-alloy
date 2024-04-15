use std::str::FromStr;
use eyre::{OptionExt, Result};
use alloy::{
    primitives::{Address, Bytes, B256, U256}, 
    providers::{Provider, ProviderBuilder}, 
    signers::wallet::LocalWallet,
    network::TransactionBuilder,
};
use suave_alloy::{
    types::ConfidentialComputeRequest,
    network::{SuaveProvider, SuaveSigner, SuaveSignerProvider},
};


#[tokio::main]

async fn main() -> Result<()> {
    // Args
    let input = Bytes::from_str("0x50723553000000000000000000000000000000000000000000000000000000000000002000000000000000000000000000000000000000000000000000000000000000074554485553445400").unwrap();
    let wallet_address = Address::from_str("0x19E7E376E7C213B7E7e7e46cc70A5dD086DAff2A").unwrap();
    let to_add = Address::from_str("0xc803334c79650708Daf3a3462AC4B48296b1352a").unwrap();
    let gas_price = U256::from_str("0x3c9aca00").unwrap();
    let cinputs = Bytes::new();
    let chain_id = 0x1008c45;
    let gas = 0x0f4240;

    // Create SUAVE signer-provider
    let rpc_url = "https://rpc.rigil.suave.flashbots.net";
    let wallet: LocalWallet = "0x1111111111111111111111111111111111111111111111111111111111111111".parse()?;    
    let provider = ProviderBuilder::default()
        .signer(SuaveSigner::from(wallet.clone()))
        .provider(SuaveProvider::try_from(rpc_url)?);

    // Get nonce and kettle address
    let tx_count: u64 = provider.get_transaction_count(wallet_address, None).await?.to();
    let kettle = provider.kettle_address().await?;

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
    
    let result = provider.send_transaction(ccr).await?;
    let tx_hash = B256::from_slice(&result.tx_hash().to_vec());
    let tx_response = provider.get_transaction_by_hash(tx_hash).await?;

    println!("{tx_response:#?}");

    let price = U256::try_from_be_slice(&tx_response.confidential_compute_result.to_vec())
        .ok_or_eyre("conf result is not U256")?;
    println!("Price: {:?}", price.wrapping_to::<u128>());

    Ok(())
}
