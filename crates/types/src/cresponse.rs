use alloy::{
    rpc::types::eth::Transaction,
    primitives::Bytes, 
};
use serde::{Deserialize, Serialize};
use super::ConfidentialComputeRecord;


#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConfidentialCallResponse {
    #[serde(flatten)]
    pub transaction: Transaction,
    pub confidential_compute_result: Bytes,
    pub request_record: ConfidentialComputeRecord,
}

impl TryFrom<Transaction> for ConfidentialCallResponse {
    type Error = eyre::Error;

    fn try_from(tx: Transaction) -> Result<Self, Self::Error> {
        let confidential_compute_result = serde_json::from_value(
            tx.other.get("confidentialComputeResult")
                .ok_or(eyre::eyre!("Missing confidentialComputeResult"))?.clone()
            )?;
        let confidential_compute_record = serde_json::from_value(
            tx.other.get("requestRecord")
                .ok_or(eyre::eyre!("Missing requestRecord"))?.clone()
            )?;

        Ok(Self {
            transaction: tx,
            confidential_compute_result,
            request_record: confidential_compute_record,
        })
    }
}

#[cfg(test)]
mod tests {
    use alloy::primitives::{FixedBytes, U256, Address};
    use std::str::FromStr;
    use super::*;
    use super::super::crecord::signature_to_vrs;

    #[test]
    fn test_parse_response() {
        let response_str = r#"{"blockHash":null,"blockNumber":null,"chainId":"0x1008c45","confidentialComputeResult":"0x0000000000000000000000000000000000000000000000000000000001ccb310","from":"0x19e7e376e7c213b7e7e7e46cc70a5dd086daff2a","gas":"0xf4240","gasPrice":"0x8c9aca00","hash":"0x82f636c7bd91f9895f896b044e33528a2d116c65eea4c8e18c30c4577ae20ce2","input":"0x0000000000000000000000000000000000000000000000000000000001ccb310","nonce":"0x45","r":"0x85242d1876ce1d6a655fd485346628f3df18a051be0f8efa4bfa40b9e85a3dfe","requestRecord":{"chainId":"0x1008c45","confidentialInputsHash":"0xc5d2460186f7233c927e7db2dcc703c0e500b653ca82273b7bfad8045d85a470","gas":"0xf4240","gasPrice":"0x8c9aca00","hash":"0x3d753c496bb9053c7da2cdbbe170614d3e9408ee12ba521c72c2b21e151b7ab9","input":"0x50723553000000000000000000000000000000000000000000000000000000000000002000000000000000000000000000000000000000000000000000000000000000074554485553445400","kettleAddress":"0x03493869959c866713c33669ca118e774a30a0e5","maxFeePerGas":null,"maxPriorityFeePerGas":null,"nonce":"0x45","r":"0xc1c5071f78c6f6b6380ebc4957dd4f6c74bdf5be742ad0d62d2d75f510e33660","s":"0x5de5c97f9c5ee5c5dad3bb0d591e581f48cd947e998d32500bb73de24dd7a6f9","to":"0xc803334c79650708daf3a3462ac4b48296b1352a","type":"0x42","v":"0x0","value":"0x0"},"s":"0x4f0880f42d42b1de17f97c33749d60a46bd1f493c6547f08ac2bed0c6d111861","to":"0xc803334c79650708daf3a3462ac4b48296b1352a","transactionIndex":null,"type":"0x50","v":"0x1","value":"0x0"}"#;
        let response_tx: Transaction = serde_json::from_str(response_str).unwrap();
        let response_cc: ConfidentialCallResponse = response_tx.clone().try_into().unwrap();

        assert_eq!(response_cc.transaction, response_tx);
        assert_eq!(response_cc.confidential_compute_result, Bytes::from_str("0x0000000000000000000000000000000000000000000000000000000001ccb310").unwrap());
        assert_eq!(response_cc.request_record.chain_id, Some(0x1008c45));
        assert_eq!(response_cc.request_record.gas, Some(0xf4240));
        assert_eq!(response_cc.request_record.gas_price, Some(0x8c9aca00));
        assert_eq!(response_cc.request_record.nonce, Some(0x45));
        assert_eq!(response_cc.request_record.input, Bytes::from_str("0x50723553000000000000000000000000000000000000000000000000000000000000002000000000000000000000000000000000000000000000000000000000000000074554485553445400").unwrap());
        assert_eq!(response_cc.request_record.kettle_address, Address::from_str("0x03493869959c866713c33669ca118e774a30a0e5").unwrap());
        assert_eq!(response_cc.request_record.to, Address::from_str("0xc803334c79650708daf3a3462ac4b48296b1352a").unwrap());
        assert_eq!(response_cc.request_record.confidential_inputs_hash, Some(FixedBytes::from_str("0xc5d2460186f7233c927e7db2dcc703c0e500b653ca82273b7bfad8045d85a470").unwrap()));
        
        let sig = response_cc.request_record.signature.expect("No signature");
        let (v, r, s) = signature_to_vrs(sig);
        assert_eq!(v, 0_u8);
        assert_eq!(r, U256::from_str("0xc1c5071f78c6f6b6380ebc4957dd4f6c74bdf5be742ad0d62d2d75f510e33660").unwrap());
        assert_eq!(s, U256::from_str("0x5de5c97f9c5ee5c5dad3bb0d591e581f48cd947e998d32500bb73de24dd7a6f9").unwrap());    
    }
}