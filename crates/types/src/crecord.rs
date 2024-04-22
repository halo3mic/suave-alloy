use serde::{Deserialize, Serialize};
use alloy_rlp::{Encodable, RlpDecodable, RlpEncodable};
use eyre::{Result, eyre};
use alloy::{
    primitives::{self, Address, Bytes, FixedBytes, U256, Signature}, 
    rpc::types::eth::TransactionRequest,
    serde as alloy_serde,
};


pub const EMPTY_BYTES_HASH: FixedBytes<32> = FixedBytes([
    197,210,70,1,134,247,35,60,146,126,125,178,220,199,3,192,229,0,182,83,202,130,39,59,123,250,216,4,93,133,164,112
]);

#[derive(Debug, Default, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ConfidentialComputeRecord {
    #[serde(with = "alloy_serde::num::u64_hex_opt")]
    pub nonce: Option<u64>,
    pub to: Address,
    #[serde(with = "alloy_serde::num::u128_hex_or_decimal_opt")]
    pub gas: Option<u128>,
    #[serde(with = "alloy_serde::num::u128_hex_or_decimal_opt")]
    pub gas_price: Option<u128>,
    pub value: U256,
    pub input: Bytes,
    pub kettle_address: Option<Address>,
    #[serde(with = "alloy_serde::num::u64_hex_opt")]
    pub chain_id: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub confidential_inputs_hash: Option<FixedBytes<32>>,
    #[serde(flatten, skip_serializing_if = "Option::is_none")]
    pub signature: Option<Signature>,
    #[serde(skip)]
    pub from: Option<Address>,
}

impl ConfidentialComputeRecord {
    pub const TYPE: u8 = 0x42;

    pub fn from_tx_request(
        tx_req: TransactionRequest, 
        kettle_address: Address, 
    ) -> Result<Self> {
        Ok(Self {
            input: tx_req.input.input.unwrap_or(Bytes::new()),
            gas_price: tx_req.gas_price,
            value: tx_req.value.unwrap_or(U256::ZERO),
            to: tx_req.to.unwrap_or(Address::ZERO),
            nonce: tx_req.nonce,
            kettle_address: Some(kettle_address),
            chain_id: tx_req.chain_id,
            gas: tx_req.gas,
            confidential_inputs_hash: None,
            signature: None,
            from: None,
        })
    }

    pub fn set_confidential_inputs_hash(&mut self, confidential_inputs_hash: FixedBytes<32>) {
        self.confidential_inputs_hash = Some(confidential_inputs_hash);
    }

    pub fn set_confidential_inputs_hash_from_inputs(
        &mut self, 
        confidential_inputs: &Bytes
    ) {
        let ci_hash = primitives::keccak256(confidential_inputs);
        self.set_confidential_inputs_hash(ci_hash);
    }

    pub fn set_sig(&mut self, signature: Signature) {
        self.signature = Some(signature);
    }

}


#[derive(Debug, RlpEncodable, RlpDecodable, PartialEq)]
pub struct CRecordRLP {
    nonce: u64,
    gas_price: u128,
    gas: u128,
    to: Address,
    value: U256,
    input: Bytes,
    kettle_address: Address,
    confidential_inputs_hash: FixedBytes<32>,
    chain_id: u64,
    v: u8,
    r: U256,
    s: U256,
}

impl CRecordRLP {
    pub fn fields_len(&self) -> usize {
        let mut len = 0;
        len += self.nonce.length();
        len += self.gas_price.length();
        len += self.gas.length();
        len += self.to.length();
        len += self.value.length();
        len += self.input.0.length();
        len += self.kettle_address.length();
        len += self.confidential_inputs_hash.length();
        len += self.chain_id.length();
        len += self.v.length();
        len += self.r.length();
        len += self.s.length();
        len
    }
}

impl TryFrom<&ConfidentialComputeRecord> for CRecordRLP {
    type Error = eyre::Error; // todo: implement custom errors

    fn try_from(ccr: &ConfidentialComputeRecord) -> Result<Self> {
        let sig = ccr.signature
            .expect("Missing signature field");
        let (v, r, s) = signature_to_vrs(sig);
        let cinputs_hash = ccr.confidential_inputs_hash.unwrap_or(EMPTY_BYTES_HASH);

        Ok(Self {
            nonce: ccr.nonce.ok_or_else(|| eyre!("Missing nonce field"))?,
            gas_price: ccr.gas_price.ok_or_else(|| eyre!("Missing gas price field"))?,
            gas: ccr.gas.ok_or_else(|| eyre!("Missing gas field"))?,
            to: ccr.to,
            value: ccr.value,
            input: ccr.input.clone(),
            kettle_address: ccr.kettle_address.ok_or_else(|| eyre!("Missing kettle address field"))?,
            confidential_inputs_hash: cinputs_hash,
            chain_id: ccr.chain_id.ok_or_else(|| eyre!("Missing chain id field"))?,
            v, r, s
        })
    }
}

impl Into<ConfidentialComputeRecord> for CRecordRLP {
    fn into(self) -> ConfidentialComputeRecord {
        let sig = Signature::from_rs_and_parity(self.r, self.s, self.v as u64)
            .expect("Invalid signature");
        ConfidentialComputeRecord {
            nonce: Some(self.nonce),
            gas_price: Some(self.gas_price),
            gas: Some(self.gas),
            to: self.to,
            value: self.value,
            input: self.input,
            kettle_address: Some(self.kettle_address),
            chain_id: Some(self.chain_id),
            confidential_inputs_hash: Some(self.confidential_inputs_hash),
            signature: Some(sig),
            from: None, // todo: retrieve from signature and prehash
        }
    }

}

pub(crate) fn signature_to_vrs(sig: Signature) -> (u8, U256, U256) {
    let v = sig.v().recid().to_byte();
    let r = sig.r();
    let s = sig.s();
    (v, r, s)
}


#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use super::*;
    use alloy::{
        network::TransactionBuilder, 
        rpc::types::eth::TransactionRequest, 
    };


    #[test]
    fn test_ccr_rlp_encode() -> Result<()> {
        let chain_id = 0x067932;
        let kettle_address = Address::from_str("0x7d83e42b214b75bf1f3e57adc3415da573d97bff").unwrap();
        let to_add = Address::from_str("0x780675d71ebe3d3ef05fae379063071147dd3aee").unwrap();
        let input = Bytes::from_str("0x236eb5a70000000000000000000000000000000000000000000000000000000000000002000000000000000000000000000000000000000000000000000000000000006000000000000000000000000000000000000000000000000000000000000000a00000000000000000000000000000000000000000000000000000000000000001000000000000000000000000780675d71ebe3d3ef05fae379063071147dd3aee0000000000000000000000000000000000000000000000000000000000000000").unwrap();
        let tx = TransactionRequest::default()
            .to(to_add)
            .gas_limit(0x0f4240)
            .with_gas_price(0x3b9aca00)
            .with_chain_id(chain_id)
            .with_nonce(0x22)
            .with_input(input)
            .with_value(U256::from(0x2233));
        
        let cc_record = ConfidentialComputeRecord::from_tx_request(tx.clone(), kettle_address)?;
        assert_eq!(cc_record.kettle_address, Some(kettle_address));
        assert_eq!(cc_record.to, to_add);
        assert_eq!(cc_record.gas, tx.gas);
        assert_eq!(cc_record.gas_price, tx.gas_price);
        assert_eq!(cc_record.chain_id, Some(chain_id));
        assert_eq!(cc_record.nonce, tx.nonce);
        assert_eq!(cc_record.input, tx.input.input.unwrap());
        assert_eq!(cc_record.value, tx.value.unwrap());
        assert!(cc_record.confidential_inputs_hash.is_none());
        assert!(cc_record.signature.is_none());

        Ok(())
    }

    #[test]
    fn test_ccr_rlp_encode_missing_fields() -> Result<()> {
        let chain_id = 0x067932;
        let kettle_address = Address::from_str("0x7d83e42b214b75bf1f3e57adc3415da573d97bff").unwrap();
        let tx = TransactionRequest::default()
            .gas_limit(0x0f4240)
            .with_chain_id(chain_id);
        
        let cc_record = ConfidentialComputeRecord::from_tx_request(tx.clone(), kettle_address)?;
        assert_eq!(cc_record.kettle_address, Some(kettle_address));
        assert_eq!(cc_record.to, Address::ZERO);
        assert_eq!(cc_record.gas, tx.gas);
        assert_eq!(cc_record.gas_price, None);
        assert_eq!(cc_record.chain_id, Some(chain_id));
        assert_eq!(cc_record.nonce, None);
        assert_eq!(cc_record.input, Bytes::new());
        assert_eq!(cc_record.value, U256::ZERO);
        assert!(cc_record.confidential_inputs_hash.is_none());
        assert!(cc_record.signature.is_none());

        Ok(())
    }

    #[test]
    fn test_missing_vals() {
        let chain_id = 0x067932;
        let kettle_address = Address::from_str("0x7d83e42b214b75bf1f3e57adc3415da573d97bff").unwrap();

        let tx = TransactionRequest::default().gas_limit(0x0f4240);
        let cc_record_res = ConfidentialComputeRecord::from_tx_request(tx, kettle_address);
        assert!(cc_record_res.is_err());

        let tx = TransactionRequest::default()
            .with_chain_id(chain_id);
        let cc_record_res = ConfidentialComputeRecord::from_tx_request(tx, kettle_address);
        assert!(cc_record_res.is_err());
    }

}
