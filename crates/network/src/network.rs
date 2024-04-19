use suave_alloy_types::{ConfidentialComputeRequest, ConfidentialCallResponse};
use alloy::{
    network::{ BuildResult, Network, NetworkSigner, TransactionBuilder, TransactionBuilderError }, 
    rpc::types::eth::{Header as EthHeader, TransactionReceipt},
    primitives::{Address, Bytes, ChainId, TxKind, B256, U256}, 
    consensus::{self, SignableTransaction, TxEnvelope}, 
    eips::eip2930::AccessList,
    eips::eip2718::Eip2718Error,
};


#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum SuaveTxType {
    /// Legacy transaction type.
    Legacy = 0,
    /// EIP-2930 transaction type.
    Eip2930 = 1,
    /// EIP-1559 transaction type.
    Eip1559 = 2,
    /// EIP-4844 transaction type.
    Eip4844 = 3,
    /// SUAVE "transaction" type
    ConfidentialComputeRequest = 4,
}

impl From<SuaveTxType> for u8 {
    fn from(value: SuaveTxType) -> Self {
        value as u8
    }
}

impl TryFrom<u8> for SuaveTxType {
    type Error = Eip2718Error;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        Ok(match value {
            0 => SuaveTxType::Legacy,
            1 => SuaveTxType::Eip2930,
            2 => SuaveTxType::Eip1559,
            3 => SuaveTxType::Eip4844,
            _ => return Err(Eip2718Error::UnexpectedType(value)),
        })
    }
}

impl std::fmt::Display for SuaveTxType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SuaveTxType::Legacy => write!(f, "Legacy"),
            SuaveTxType::Eip2930 => write!(f, "EIP-2930"),
            SuaveTxType::Eip1559 => write!(f, "EIP-1559"),
            SuaveTxType::Eip4844 => write!(f, "EIP-4844"),
            SuaveTxType::ConfidentialComputeRequest => write!(f, "ConfidentialComputeRequest"),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct SuaveNetwork;

impl Network for SuaveNetwork {
    type TxType = SuaveTxType;
    type TxEnvelope = ConfidentialComputeRequest;
    type UnsignedTx = ConfidentialComputeRequest;
    type ReceiptEnvelope = TxEnvelope;
    type Header = consensus::Header;
    type TransactionRequest = ConfidentialComputeRequest;
    type TransactionResponse = ConfidentialCallResponse;
    type ReceiptResponse = TransactionReceipt;
    type HeaderResponse = EthHeader;
}

type SuaveBuildResult<T> = BuildResult<T, SuaveNetwork>;

impl TransactionBuilder<SuaveNetwork> for ConfidentialComputeRequest {

    fn chain_id(&self) -> Option<ChainId> {
        self.confidential_compute_record.chain_id
    }

    fn set_chain_id(&mut self, chain_id: ChainId) {
        self.confidential_compute_record.chain_id = Some(chain_id);
    }

    fn nonce(&self) -> Option<u64> {
        self.confidential_compute_record.nonce
    }

    fn set_nonce(&mut self, nonce: u64) {
        self.confidential_compute_record.nonce = Some(nonce);
    }

    fn input(&self) -> Option<&Bytes> {
        Some(&self.confidential_compute_record.input)
    }

    fn set_input(&mut self, input: Bytes) {
        self.confidential_compute_record.input = input;
    }

    fn from(&self) -> Option<Address> {
        self.confidential_compute_record.from
        // self.confidential_compute_record.signature.map(|sig| {
        //     let prehash = self.signature_hash();
        //     sig.recover_address_from_prehash(&B256::from(prehash)).expect("Invalid signature")
        // })
    }

    fn set_from(&mut self, from: Address) {
        // panic!("Cannot set from address for confidential compute request");
        self.confidential_compute_record.from = Some(from);
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

    fn gas_price(&self) -> Option<u128> {
        self.confidential_compute_record.gas_price
    }

    fn set_gas_price(&mut self, gas_price: u128) {
        self.confidential_compute_record.gas_price = Some(gas_price);
    }

    fn max_fee_per_gas(&self) -> Option<u128> {
        None
    }

    fn set_max_fee_per_gas(&mut self, _max_fee_per_gas: u128) {
        panic!("Cannot set max fee per gas for confidential compute request");
    }

    fn max_priority_fee_per_gas(&self) -> Option<u128> {
        None
    }

    fn set_max_priority_fee_per_gas(&mut self, _max_priority_fee_per_gas: u128) {
        panic!("Cannot set max priority fee per gas for confidential compute request");
    }

    fn max_fee_per_blob_gas(&self) -> Option<u128> {
        None
    }

    fn set_max_fee_per_blob_gas(&mut self, _max_fee_per_blob_gas: u128) {
        panic!("Cannot set max fee per blob gas for confidential compute request");
    }

    fn gas_limit(&self) -> Option<u128> {
        self.confidential_compute_record.gas
    }

    fn set_gas_limit(&mut self, gas_limit: u128) {
        let gas = gas_limit.try_into().expect("Overflowing gas param");
        self.confidential_compute_record.gas = gas;
    }

    fn set_blob_sidecar(&mut self, _blob_sidecar: alloy::consensus::BlobTransactionSidecar) {
        panic!("Cannot set blob sidecar for confidential compute request");
    }

    fn build_unsigned(self) -> SuaveBuildResult<<SuaveNetwork as Network>::UnsignedTx>{
        // todo: Instead of returning CCR with optional fields, return a struct with required fields
        Ok(self)
    }

    async fn build<S: NetworkSigner<SuaveNetwork>>(
        self,
        signer: &S,
    ) -> Result<<SuaveNetwork as Network>::TxEnvelope, TransactionBuilderError<SuaveNetwork>> {
        match self.build_unsigned() {
            Ok(tx) => {
                signer.sign_transaction(tx).await.map_err(|e| e.into())
            },
            Err(e) => {
                todo!() // todo: handle (why is this different err than build_unsigned?)
            }, 
        }
    }

    fn access_list(&self) -> Option<&AccessList> {
        None
    }

    fn set_access_list(&mut self, _access_list: AccessList) {
        panic!("Cannot set access list for confidential compute request");
    }

    fn blob_sidecar(&self) -> Option<&consensus::BlobTransactionSidecar> {
        None
    }

    // todo: implement types
    fn complete_type(&self, ty: SuaveTxType) -> Result<(), Vec<&'static str>> {
        unimplemented!("complete_type")
    }

    fn can_submit(&self) -> bool {
        true
    }

    fn can_build(&self) -> bool {
        // todo: this check goes into crecord
        self.confidential_compute_record.nonce.is_some() && 
        self.confidential_compute_record.gas.is_some() &&
        self.confidential_compute_record.from.is_some() &&
        self.confidential_compute_record.chain_id.is_some()
    }

    fn output_tx_type(&self) -> SuaveTxType {
        panic!("Not supported")
    }

    fn output_tx_type_checked(&self) -> Option<SuaveTxType> {
        panic!("Not supported")
    }

    fn prep_for_submission(&mut self) {
        unimplemented!("prep_for_submission")
    }

}