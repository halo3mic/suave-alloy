use alloy::{
    providers::{Provider, ProviderBuilder, RootProvider},
    transports::{http::Http, TransportResult},
    primitives::Address,
};
use reqwest::Client as ReqwestClient;
use super::network::SuaveNetwork;


pub struct SuaveProvider {
    root_provider: RootProvider<Http<ReqwestClient>, SuaveNetwork>,
}

impl SuaveProvider {

    pub fn from_url(url: &str) -> Self {
        let url = url.parse().unwrap();
        let root_provider = ProviderBuilder::<_, SuaveNetwork>::default()
            .on_reqwest_http(url).expect("Failed to root provider for SuaveProvider");
        Self { root_provider }
    }

    pub async fn kettle_address(&self) -> TransportResult<Address> {
        self.client()
            .request(String::from("eth_kettleAddress"), ())
            .await
            .map(|ks: Vec<Address>| ks[0])
    }

}

impl Provider<Http<ReqwestClient>, SuaveNetwork> for SuaveProvider {

    fn root(&self) -> &RootProvider<Http<ReqwestClient>, SuaveNetwork> { 
        &self.root_provider
    }

}

#[cfg(test)]
mod tests {
    use std::str::FromStr;
    use super::*;

    #[tokio::test]
    async fn test_suave_provider() {
        let provider = SuaveProvider::from_url("https://rpc.rigil.suave.flashbots.net");
        let kettle_address = provider.kettle_address().await.unwrap();
        assert_eq!(kettle_address, Address::from_str("0x03493869959c866713c33669ca118e774a30a0e5").unwrap());
    }

}