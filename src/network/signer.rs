use alloy::{
    network::{TxSigner, NetworkSigner, Network},
    signers::Result as SignerResult,
    primitives::Signature,
};
use async_trait::async_trait;
use std::sync::Arc;

use crate::types::ConfidentialComputeRequest;


#[derive(Clone)]
pub struct SuaveSigner(Arc<dyn TxSigner<Signature> + Send + Sync>);

impl SuaveSigner {
    pub fn new<S>(signer: S) -> Self
    where
        S: TxSigner<Signature> + Send + Sync + 'static,
    {
        Self(Arc::new(signer))
    }

    async fn sign_transaction(
        &self,
        tx: &mut ConfidentialComputeRequest,
    ) -> SignerResult<ConfidentialComputeRequest> {
        self.0.sign_transaction(tx).await.map(|sig| {
            tx.confidential_compute_record.set_sig(sig);
            tx.clone()
        })
    }
}

#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
impl<N> NetworkSigner<N> for SuaveSigner
where
    N: Network<UnsignedTx = ConfidentialComputeRequest, TxEnvelope = ConfidentialComputeRequest>,
{
    async fn sign_transaction(
        &self,
        tx: ConfidentialComputeRequest,
    ) -> SignerResult<ConfidentialComputeRequest> {
        let mut tx = tx;
        self.sign_transaction(&mut tx).await
    }
}

impl std::fmt::Debug for SuaveSigner {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("SuaveSigner").finish()
    }
}

impl<S> From<S> for SuaveSigner
where
    S: TxSigner<Signature> + Send + Sync + 'static,
{
    fn from(signer: S) -> Self {
        Self::new(signer)
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;
    use eyre::Result;
    use alloy::{
        primitives::{Address, Bytes, B256, U256}, 
        providers::{Provider, ProviderBuilder}, 
        rpc::types::eth::TransactionRequest, 
        signers::wallet::LocalWallet,
        network::TransactionBuilder,
    };
    use crate::types::ConfidentialComputeRecord;
    use super::super::{
        provider::SuaveProvider,
        network::SuaveNetwork,
    };
    use super::*;

    // todo: dont rely on external conditions for testing
    #[tokio::test]
    async fn test_send_tx_rigil() -> Result<()> {
        let provider = SuaveProvider::from_url("https://rpc.rigil.suave.flashbots.net");
        let wallet_address = Address::from_str("0x19E7E376E7C213B7E7e7e46cc70A5dD086DAff2A").unwrap();

        let tx_count: u64 = provider.get_transaction_count(wallet_address, None).await.unwrap().to();
        let kettle = provider.kettle_address().await.unwrap();

        // Create a cc request 
        let cinputs = Bytes::new();
        let to_add = Address::from_str("0xc803334c79650708Daf3a3462AC4B48296b1352a").unwrap();
        let gas = 0x0f4240;
        let gas_price = U256::from_str("0x3c9aca00").unwrap();
        let input = Bytes::from_str("0x50723553000000000000000000000000000000000000000000000000000000000000002000000000000000000000000000000000000000000000000000000000000000074554485553445400").unwrap();
        let chain_id = 0x1008c45;
        let tx = TransactionRequest::default()
            .to(Some(to_add))
            .gas_limit(U256::from(gas))
            .with_gas_price(gas_price)
            .with_chain_id(chain_id)
            .with_nonce(tx_count)
            .with_input(input);
        let cc_record = ConfidentialComputeRecord::from_tx_request(tx, kettle)?;
        let cc_request = ConfidentialComputeRequest::new(cc_record, cinputs);
        
        let wallet: LocalWallet = "0x1111111111111111111111111111111111111111111111111111111111111111".parse().unwrap();    
        let signer = SuaveSigner::from(wallet.clone());
        let provider = ProviderBuilder::<_, SuaveNetwork>::default()
            .signer(signer).provider(provider);
        
        let result = provider.send_transaction(cc_request).await.unwrap();
        let tx_hash = B256::from_slice(&result.tx_hash().to_vec());
        let tx_response = provider.get_transaction_by_hash(tx_hash).await.unwrap();
  
        let price = U256::try_from_be_slice(&tx_response.confidential_compute_result.to_vec()).unwrap();
        assert!(price > U256::ZERO);

        Ok(())
    }

}