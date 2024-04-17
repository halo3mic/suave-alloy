use alloy::primitives::{Bytes, Address};
use alloy::network::TransactionBuilder;
use alloy::contract::CallBuilder;

use super::SuaveNetwork;

pub trait SuaveCallBuilderExt {
    fn with_cinput(self, cinput: Bytes) -> Self;
    fn with_kettle_address(self, kettle_address: Address) -> Self;
    fn with_chain_id(self, chain_id: u64) -> Self;
}

impl<T, P, D> SuaveCallBuilderExt for CallBuilder<T, P, D, SuaveNetwork> 
    where T: Clone + alloy::transports::Transport
{

    fn with_chain_id(mut self, chain_id: u64) -> Self {
        self.request.set_chain_id(chain_id);
        self
    }

    fn with_cinput(mut self, cinput: Bytes) -> Self {
        self.request.set_confidential_inputs(cinput);
        self
    }

    fn with_kettle_address(mut self, kettle_address: Address) -> Self {
        self.request.set_kettle_address(kettle_address);
        self
    }

}


#[cfg(test)]
mod tests {
    use std::str::FromStr;
    use eyre::Result;
    use alloy::{
        primitives::{Address, Bytes}, 
        providers::{Provider, ProviderBuilder},
        rpc::types::eth::BlockId,
        signers::wallet::LocalWallet,
        sol,
    };
    use super::super::{SuaveProvider, SuaveNetwork, SuaveSigner, SuaveFillProviderExt};
    use super::*;


    #[tokio::test]
    async fn test_send_ccr_with_call_builder() -> Result<()> {
        sol! {
            #[sol(rpc)]
            contract BinanceOracle {
                #[derive(Debug)]
                function queryLatestPrice(string memory ticker) public view returns (uint price);
            }
        }
        let boracle_add = Address::from_str("0xc803334c79650708Daf3a3462AC4B48296b1352a")?;
        let ticker = String::from("ETHUSDT");
        let rpc_url = "https://rpc.rigil.suave.flashbots.net";
        let pk = "0x1111111111111111111111111111111111111111111111111111111111111111";

        // Create SUAVE signer-provider
        let wallet: LocalWallet = pk.parse()?;    
        let provider = ProviderBuilder::<_, _, SuaveNetwork>::default()
            .with_recommended_fillers() // todo: why is this not filling my txs?
            .signer(SuaveSigner::new(wallet.clone()))
            .on_provider(SuaveProvider::try_from(rpc_url)?);
        let nonce = provider.get_transaction_count(wallet.address(), BlockId::latest()).await?;

        // Create call builder
        let contract = BinanceOracle::new(boracle_add, &provider);
        let call_builder = contract.queryLatestPrice(ticker)
            .with_kettle_address(provider.kettle_address().await?)
            .with_cinput(Bytes::new())
            .with_chain_id(0x1008c45)
            .gas(0x0f4240)
            .nonce(nonce);

        // Send tx
        let pending_tx = call_builder.send().await?;
        let confidential_request_res = provider.get_transaction_by_hash(*pending_tx.tx_hash()).await?;
        println!("{:#?}", confidential_request_res);

        Ok(())
    }


}
