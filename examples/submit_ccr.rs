use std::str::FromStr;
use eyre::{Result, OptionExt};
use alloy::{
    network::TransactionBuilder, 
    primitives::{Address, Bytes, B256, U256}, 
    providers::{Provider, ProviderBuilder}, 
    signers::wallet::LocalWallet
};
use suave_alloy::{
    network::{SuaveNetwork, SuaveProvider, SuaveSigner, SuaveFillProviderExt}, 
    types::ConfidentialComputeRequest
};


#[tokio::main]

async fn main() -> Result<()> {
    // Args
    let input = Bytes::from_str("0x50723553000000000000000000000000000000000000000000000000000000000000002000000000000000000000000000000000000000000000000000000000000000074554485553445400").unwrap();
    let to_add = Address::from_str("0xc803334c79650708Daf3a3462AC4B48296b1352a").unwrap();
    let gas_price = 0x4c9aca00;
    let cinputs = Bytes::new();
    let gas = 0x0f4240; // Estimate gas doesn't work well with MEVM

    // Create SUAVE signer-provider
    let rpc_url = "https://rpc.rigil.suave.flashbots.net";
    let wallet: LocalWallet = "0x1111111111111111111111111111111111111111111111111111111111111111".parse()?; 
    let provider = ProviderBuilder::<_, _, SuaveNetwork>::default()
        .with_recommended_fillers()
        .signer(SuaveSigner::new(wallet))
        .on_provider(SuaveProvider::try_from(rpc_url)?);

    // Get nonce and kettle address
    let kettle = provider.kettle_address().await?;

    // Create a confidential-compute-request 
    let ccr = ConfidentialComputeRequest::default()
        .with_to(Some(to_add).into())
        .with_gas_price(gas_price)
        .with_gas_limit(gas)
        .with_input(input)
        .with_confidential_inputs(cinputs) // No need to specify it if no confidential input
        .with_kettle_address(kettle);
    
    // Send CCR
    let result = provider.send_transaction(ccr).await?;
    let tx_hash = B256::from_slice(&result.tx_hash().to_vec());

    // Obtain CCR Response with record and compute-result
    let tx_response = provider.get_transaction_by_hash(tx_hash).await?;
    println!("{tx_response:#?}");

    let price = U256::try_from_be_slice(&tx_response.confidential_compute_result.to_vec())
        .ok_or_eyre("conf result is not U256")?;
    println!("Price: {:?}", price.wrapping_to::<u128>());

    Ok(())
}
