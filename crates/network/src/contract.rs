use alloy::{
    contract::{CallBuilder, CallDecoder},
    primitives::{Bytes, Address},
    providers::Provider,
};
use super::SuaveNetwork;


pub trait SuaveCallBuilderExt {
    fn with_cinput(self, cinput: Bytes) -> Self;
    fn with_kettle_address(self, kettle_address: Address) -> Self;
}

impl<T, P, D> SuaveCallBuilderExt for CallBuilder<T, P, D, SuaveNetwork> 
    where 
        T: Clone + alloy::transports::Transport,
        P: Provider<T, SuaveNetwork>,
        P: Sync,
        D: CallDecoder
{

    fn with_cinput(self, cinput: Bytes) -> Self {
        self.map(|tx| tx.with_confidential_inputs(cinput))
    }

    fn with_kettle_address(self, kettle_address: Address) -> Self {
        self.map(|tx| tx.with_kettle_address(kettle_address))
    }

}
