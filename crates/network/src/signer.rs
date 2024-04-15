use alloy::{
    network::{TxSigner, NetworkSigner, Network},
    signers::Result as SignerResult,
    primitives::Signature,
};
use async_trait::async_trait;
use std::sync::Arc;

use suave_alloy_types::ConfidentialComputeRequest;


#[derive(Clone)]
pub struct SuaveSigner(Arc<dyn TxSigner<Signature> + Send + Sync>);

impl SuaveSigner {
    pub fn new<S>(signer: S) -> Self
    where
        S: TxSigner<Signature> + Send + Sync + 'static,
    {
        Self(Arc::new(signer))
    }

    async fn sign_transaction(
        &self,
        tx: &mut ConfidentialComputeRequest,
    ) -> SignerResult<ConfidentialComputeRequest> {
        self.0.sign_transaction(tx).await.map(|sig| {
            tx.confidential_compute_record.set_sig(sig);
            tx.clone()
        })
    }
}

#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
impl<N> NetworkSigner<N> for SuaveSigner
where
    N: Network<UnsignedTx = ConfidentialComputeRequest, TxEnvelope = ConfidentialComputeRequest>,
{
    async fn sign_transaction(
        &self,
        tx: ConfidentialComputeRequest,
    ) -> SignerResult<ConfidentialComputeRequest> {
        let mut tx = tx;
        self.sign_transaction(&mut tx).await
    }
}

impl std::fmt::Debug for SuaveSigner {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("SuaveSigner").finish()
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
