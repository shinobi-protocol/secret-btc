use cosmwasm_std::Api;
use sfps_lib::light_block::Ed25519Verifier;

pub struct ApiEd25519Verifier<'a, A: Api> {
    pub api: &'a A,
}

impl<'a, A: Api> Ed25519Verifier for ApiEd25519Verifier<'a, A> {
    fn verify_batch(
        &mut self,
        messages: &[&[u8]],
        signatures: &[&[u8]],
        public_keys: &[&[u8]],
    ) -> Result<(), sfps_lib::light_block::Error> {
        #[cfg(feature = "full_signatures_test")]
        let (messages, signatures, public_keys) = (
            &repeat_element(messages.last().unwrap(), 80),
            &repeat_element(signatures.last().unwrap(), 80),
            &repeat_element(public_keys.last().unwrap(), 80),
        );

        if self
            .api
            .ed25519_batch_verify(messages, signatures, public_keys)
            .map_err(|_| sfps_lib::light_block::Error::VerifyBatchFailed {})?
        {
            Ok(())
        } else {
            Err(sfps_lib::light_block::Error::VerifyBatchFailed {})
        }
    }
}

#[cfg(feature = "full_signatures_test")]
fn repeat_element<T: Clone>(element: &T, time: usize) -> Vec<T> {
    let mut vec = Vec::with_capacity(time);
    for _ in 0..time {
        vec.push(element.clone())
    }
    vec
}
