use reqwest::Client as ReqwestClient;
use std::str::FromStr;
use alloy::{
    providers::{
        Provider, ProviderBuilder, RootProvider,
        fillers::{FillProvider, TxFiller}, 
    },
    transports::{http::Http, TransportResult, Transport},
    rpc::client::ClientRef,
    primitives::Address,
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