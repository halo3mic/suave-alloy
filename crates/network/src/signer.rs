use async_trait::async_trait;
use std::{
    collections::BTreeMap,
    sync::Arc,
};
use alloy::{
    signers::{Result as SignerResult, Error as SignerError},
    network::{TxSigner, NetworkSigner},
    primitives::{Address, Signature},
};
use suave_alloy_types::ConfidentialComputeRequest;
use crate::SuaveNetwork;


#[derive(Clone)]
pub struct SuaveSigner {
    default_signer: Address,
    signers: BTreeMap<Address, Arc<dyn TxSigner<Signature> + Send + Sync>>,
}

impl SuaveSigner {
    pub fn new<S>(signer: S) -> Self
    where
        S: TxSigner<Signature> + Send + Sync + 'static,
    {
        let signer = Arc::new(signer);
        let mut this = Self {
            default_signer: signer.address(),
            signers: BTreeMap::new(),
        };
        this.register_signer(signer);
        this
    }

    pub fn register_signer(&mut self, signer: Arc<dyn TxSigner<Signature> + Send + Sync>) {
        self.signers.insert(signer.address(), signer);
    }

    pub async fn sign_transaction(&self, tx: &mut ConfidentialComputeRequest) -> SignerResult<ConfidentialComputeRequest> {
        self.sign_transaction_from(self.default_signer, tx).await
    }

    async fn sign_transaction_from(
        &self,
        sender: Address,
        tx: &mut ConfidentialComputeRequest,
    ) -> SignerResult<ConfidentialComputeRequest> {
        self.signers.get(&sender)
            .ok_or(SignerError::other("unknown signer"))?
            .sign_transaction(tx).await.map(|sig| {
                tx.confidential_compute_record.set_sig(sig);
                tx.confidential_compute_record.from = Some(sender);
                tx.clone()
            })
    }
}

#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
impl NetworkSigner<SuaveNetwork> for SuaveSigner {

    fn default_signer(&self) -> Address {
        self.default_signer
    }

    fn is_signer_for(&self,address: &Address) -> bool {
        self.signers.contains_key(address)
    }

    fn signers(&self) -> impl Iterator<Item = Address> {
        self.signers.keys().cloned()
    }

    async fn sign_transaction_from(
        &self,
        sender: Address,
        mut tx: ConfidentialComputeRequest,
    ) -> SignerResult<ConfidentialComputeRequest> {
        self.sign_transaction_from(sender, &mut tx).await
    }

}

impl std::fmt::Debug for SuaveSigner {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let signers_add = self.signers.keys().collect::<Vec<_>>();
        f.debug_struct("SuaveSigner")
            .field("default_signer", &self.default_signer)
            .field("signers", &signers_add)
            .finish()
    }
}

impl<S> From<S> for SuaveSigner
where
    S: TxSigner<Signature> + Send + Sync + 'static,
{
    fn from(signer: S) -> Self {
        Self::new(signer)
    }
}
