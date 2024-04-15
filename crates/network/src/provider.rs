use std::str::FromStr;

use alloy::{
    providers::{Provider, ProviderBuilder, RootProvider, layers::SignerProvider},
    transports::{http::Http, TransportResult},
    rpc::client::ClientRef,
    primitives::Address,
};
use reqwest::Client as ReqwestClient;
use super::network::SuaveNetwork;
use super::signer::SuaveSigner;


#[derive(Clone)]
pub struct SuaveProvider {
    root_provider: RootProvider<Http<ReqwestClient>, SuaveNetwork>,
}

impl SuaveProvider {

    pub fn new(url: url::Url) -> Self {
        let root_provider = ProviderBuilder::<_, SuaveNetwork>::default()
            .on_reqwest_http(url).expect("Failed to root provider for SuaveProvider");
        Self { root_provider }
    }

    pub async fn kettle_address(&self) -> TransportResult<Address> {
        kettle_address(self.client()).await
    }

}

impl Provider<Http<ReqwestClient>, SuaveNetwork> for SuaveProvider {

    fn root(&self) -> &RootProvider<Http<ReqwestClient>, SuaveNetwork> { 
        &self.root_provider
    }

}

impl TryFrom<&str> for SuaveProvider {
    type Error = url::ParseError;

    fn try_from(url: &str) -> Result<Self, Self::Error> {
        Ok(SuaveProvider::new(url.parse()?))
    }

}

impl FromStr for SuaveProvider {
    type Err = url::ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(SuaveProvider::new(s.parse()?))
    }

}

pub trait SuaveSignerProvider {
    fn kettle_address(&self) -> impl std::future::Future<Output = TransportResult<Address>> + Send;
}

impl SuaveSignerProvider for SignerProvider<Http<ReqwestClient>, SuaveProvider, SuaveSigner, SuaveNetwork> {
    async fn kettle_address(&self) -> TransportResult<Address> {
        kettle_address(self.client()).await
    }
}

async fn kettle_address<'a>(client: ClientRef<'a ,Http<reqwest::Client>>) -> TransportResult<Address> {
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