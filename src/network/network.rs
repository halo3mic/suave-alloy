use alloy::{
    consensus::{self, SignableTransaction, TxEnvelope}, 
    network::{ BuilderResult, Network, NetworkSigner, TransactionBuilder }, 
    primitives::{Address, Bytes, ChainId, TxKind, B256, U256}, 
    rpc::types::eth::{Header as EthHeader, TransactionReceipt}
};
use crate::types::{ConfidentialComputeRequest, ConfidentialCallResponse};


#[derive(Debug, Clone, Copy)]
pub struct SuaveNetwork;

impl Network for SuaveNetwork {
    type TxEnvelope = ConfidentialComputeRequest;
    type UnsignedTx = ConfidentialComputeRequest;
    type ReceiptEnvelope = TxEnvelope;
    type Header = consensus::Header;
    type TransactionRequest = ConfidentialComputeRequest;
    type TransactionResponse = ConfidentialCallResponse;
    type ReceiptResponse = TransactionReceipt;
    type HeaderResponse = EthHeader;
}


impl TransactionBuilder<SuaveNetwork> for ConfidentialComputeRequest {

    fn chain_id(&self) -> Option<ChainId> {
        Some(self.confidential_compute_record.chain_id)
    }

    fn set_chain_id(&mut self, chain_id: ChainId) {
        self.confidential_compute_record.chain_id = chain_id;
    }

    fn nonce(&self) -> Option<u64> {
        Some(self.confidential_compute_record.nonce)
    }

    fn set_nonce(&mut self, nonce: u64) {
        self.confidential_compute_record.nonce = nonce;
    }

    fn input(&self) -> Option<&Bytes> {
        Some(&self.confidential_compute_record.input)
    }

    fn set_input(&mut self, input: Bytes) {
        self.confidential_compute_record.input = input;
    }

    fn from(&self) -> Option<Address> {
        self.confidential_compute_record.signature.map(|sig| {
            let prehash = self.signature_hash();
            sig.recover_address_from_prehash(&B256::from(prehash)).expect("Invalid signature")
        })
    }

    fn set_from(&mut self, _from: Address) {
        panic!("Cannot set from address for confidential compute request");
    }

    fn to(&self) -> Option<TxKind> {
        Some(TxKind::Call(self.confidential_compute_record.to))
    }

    fn set_to(&mut self, to: TxKind) {
        let new_address = match to {
            TxKind::Call(addr) => addr,
            TxKind::Create => Address::ZERO,
        };
        self.confidential_compute_record.to = new_address;
    }

    fn value(&self) -> Option<U256> {
        Some(self.confidential_compute_record.value)
    }

    fn set_value(&mut self, value: U256) {
        self.confidential_compute_record.value = value;
    }

    fn gas_price(&self) -> Option<U256> {
        Some(self.confidential_compute_record.gas_price)
    }

    fn set_gas_price(&mut self, gas_price: U256) {
        self.confidential_compute_record.gas_price = gas_price;
    }

    fn max_fee_per_gas(&self) -> Option<U256> {
        None
    }

    fn set_max_fee_per_gas(&mut self, _max_fee_per_gas: U256) {
        panic!("Cannot set max fee per gas for confidential compute request");
    }

    fn max_priority_fee_per_gas(&self) -> Option<U256> {
        None
    }

    fn set_max_priority_fee_per_gas(&mut self, _max_priority_fee_per_gas: U256) {
        panic!("Cannot set max priority fee per gas for confidential compute request");
    }

    fn max_fee_per_blob_gas(&self) -> Option<U256> {
        None
    }

    fn set_max_fee_per_blob_gas(&mut self, _max_fee_per_blob_gas: U256) {
        panic!("Cannot set max fee per blob gas for confidential compute request");
    }

    fn gas_limit(&self) -> Option<U256> {
        Some(U256::from(self.confidential_compute_record.gas))
    }

    fn set_gas_limit(&mut self, gas_limit: U256) {
        let gas = gas_limit.try_into().expect("Overflowing gas param");
        self.confidential_compute_record.gas = gas;
    }

    fn get_blob_sidecar(&self) -> Option<&alloy::consensus::BlobTransactionSidecar> {
        None
    }

    fn set_blob_sidecar(&mut self, _blob_sidecar: alloy::consensus::BlobTransactionSidecar) {
        panic!("Cannot set blob sidecar for confidential compute request");
    }

    // todo: have a different struct for built and built-unsigned?
    fn build_unsigned(self) -> BuilderResult<<SuaveNetwork as Network>::UnsignedTx>{
        Ok(self)
    }

    async fn build<S: NetworkSigner<SuaveNetwork>>(
        self,
        signer: &S,
    ) -> BuilderResult<<SuaveNetwork as Network>::TxEnvelope> {
        signer.sign_transaction(self.build_unsigned()?).await.map_err(|e| e.into())
    }


}