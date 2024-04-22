use reqwest::Client as ReqwestClient;
use std::sync::{Arc, OnceLock};
use std::str::FromStr;
use alloy::{
    transports::{http::Http, Transport, TransportResult},
    providers::{
        fillers::{FillProvider, FillerControlFlow, TxFiller}, 
        Provider, ProviderBuilder, RootProvider, SendableTx,
    },
    rpc::client::ClientRef, 
    primitives::Address, 
    network::Network, 
};
use super::network::SuaveNetwork;


#[derive(Clone)]
pub struct SuaveProvider<T> 
    where T: Transport + Clone
{
    root_provider: RootProvider<T, SuaveNetwork>,
}

impl<T> SuaveProvider<T> 
    where T: Transport + Clone
{
    pub fn new(root_provider: RootProvider<T, SuaveNetwork>) -> Self {
        Self { root_provider }
    }

    pub async fn kettle_address(&self) -> TransportResult<Address> {
        kettle_address(self.client()).await
    }
}

type ReqwestHttp = Http<ReqwestClient>;

impl SuaveProvider<ReqwestHttp> {

    pub fn from_http(url: url::Url) -> SuaveProvider<ReqwestHttp> {
        let root_provider = ProviderBuilder::<_, _, SuaveNetwork>::default()
            .on_http(url).expect("Failed to root provider for SuaveProvider");
        Self { root_provider }
    }

}

impl<T> Provider<T, SuaveNetwork> for SuaveProvider<T> 
    where T: Transport + Clone
{

    fn root(&self) -> &RootProvider<T, SuaveNetwork> { 
        &self.root_provider
    }

}

impl TryFrom<&str> for SuaveProvider<ReqwestHttp> {
    type Error = url::ParseError;

    fn try_from(url: &str) -> Result<Self, Self::Error> {
        Ok(SuaveProvider::from_http(url.parse()?))
    }

}

impl FromStr for SuaveProvider<ReqwestHttp> {
    type Err = url::ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(SuaveProvider::from_http(s.parse()?))
    }

}

pub trait SuaveFillProviderExt {
    fn kettle_address(&self) -> impl std::future::Future<Output = TransportResult<Address>> + Send;
}

// todo: optimize for wasm
// #[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
// #[cfg_attr(not(target_arch = "wasm32"), async_trait)]
impl<S, T> SuaveFillProviderExt for FillProvider<S, SuaveProvider<T>, T, SuaveNetwork> 
    where S: TxFiller<SuaveNetwork>, T: Transport + Clone
{
    async fn kettle_address(&self) -> TransportResult<Address> {
        kettle_address(self.client()).await
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct KettleFiller(Arc<OnceLock<Address>>);

impl KettleFiller {

    pub fn new(kettle_address: Option<Address>) -> Self {
        let lock = OnceLock::new();
        if let Some(kettle_address) = kettle_address {
            lock.set(kettle_address).expect("brand new");
        }
        Self(Arc::new(lock))
    }

}

impl TxFiller<SuaveNetwork> for KettleFiller {
    type Fillable = Address;

    fn status(&self, tx: &<SuaveNetwork as Network>::TransactionRequest) -> FillerControlFlow {
        if tx.kettle_address().is_some() {
            FillerControlFlow::Finished
        } else {
            FillerControlFlow::Ready
        }
    }

    async fn prepare<P, T>(
        &self,
        provider: &P,
        _tx: &<SuaveNetwork as Network>::TransactionRequest,
    ) -> TransportResult<Self::Fillable>
    where
        P: Provider<T, SuaveNetwork>,
        T: Transport + Clone,
    {
        match self.0.get().cloned() {
            Some(kettle) => Ok(kettle),
            None => {
                let kettle = kettle_address(&provider.client()).await?;
                let kettle = *self.0.get_or_init(|| kettle);
                Ok(kettle)
            }
        }
    }

    async fn fill(
        &self,
        fillable: Self::Fillable,
        mut tx: SendableTx<SuaveNetwork>,
    ) -> TransportResult<SendableTx<SuaveNetwork>> {
        if let Some(builder) = tx.as_mut_builder() {
            if builder.kettle_address().is_none() {
                builder.set_kettle_address(fillable)
            }
        };
        Ok(tx)
    }

}

async fn kettle_address<'a, T>(client: ClientRef<'a , T>) -> TransportResult<Address> 
    where T: Transport + Clone
{
    client.request(String::from("eth_kettleAddress"), ()).await
        .map(|ks: Vec<Address>| ks[0])
}


#[cfg(test)]
mod tests {
    use std::str::FromStr;
    use eyre::Result;
    use super::*;

    #[tokio::test]
    async fn test_suave_provider() -> Result<()> {
        let provider = SuaveProvider::try_from("https://rpc.rigil.suave.flashbots.net")?;
        let kettle_address = provider.kettle_address().await.unwrap();
        assert_eq!(kettle_address, Address::from_str("0x03493869959c866713c33669ca118e774a30a0e5").unwrap());
        Ok(())
    }

}