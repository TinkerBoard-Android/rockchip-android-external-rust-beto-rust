// Copyright 2023 Google LLC
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use crypto_provider::ed25519::{
    InvalidBytes, InvalidSignature, Signature as _, SignatureError, KEY_LENGTH, KEY_PAIR_LENGTH,
    SIGNATURE_LENGTH,
};
use openssl::pkey::{Id, PKey, Private};
use openssl::sign::{Signer, Verifier};

pub struct Ed25519;

impl crypto_provider::ed25519::Ed25519Provider for Ed25519 {
    type KeyPair = KeyPair;
    type PublicKey = PublicKey;
    type Signature = Signature;
}

pub struct KeyPair(PKey<Private>);

impl crypto_provider::ed25519::KeyPair for KeyPair {
    type PublicKey = PublicKey;
    type Signature = Signature;

    fn to_bytes(&self) -> [u8; KEY_PAIR_LENGTH] {
        let private_key = self.0.raw_private_key().unwrap();
        let mut public_key = self.0.raw_public_key().unwrap();
        let mut result = private_key;
        result.append(&mut public_key);
        result.try_into().unwrap()
    }

    fn from_bytes(bytes: [u8; KEY_PAIR_LENGTH]) -> Result<Self, InvalidBytes> {
        PKey::private_key_from_raw_bytes(&bytes[..KEY_LENGTH], Id::ED25519)
            .map(Self)
            .map_err(|_| InvalidBytes)
    }

    fn sign(&self, msg: &[u8]) -> Self::Signature {
        let mut signer =
            Signer::new_without_digest(&self.0).expect("should be able to create a signer");
        let sig_bytes: [u8; SIGNATURE_LENGTH] = signer
            .sign_oneshot_to_vec(msg)
            .expect("singing should succeed")
            .try_into()
            .expect("signature should be a valid size");

        Self::Signature::from_bytes(&sig_bytes).expect("this should never fail")
    }

    fn generate() -> Self {
        let key = PKey::generate_ed25519().expect("key generation should not fail");
        Self(key)
    }

    fn public(&self) -> Self::PublicKey {
        PublicKey(
            self.0
                .raw_public_key()
                .expect("should be able to get a pubic key from a keypair"),
        )
    }
}

pub struct Signature([u8; SIGNATURE_LENGTH]);

impl crypto_provider::ed25519::Signature for Signature {
    fn from_bytes(bytes: &[u8]) -> Result<Self, InvalidSignature> {
        bytes.try_into().map(Self).map_err(|_| InvalidSignature)
    }

    fn to_bytes(&self) -> [u8; SIGNATURE_LENGTH] {
        self.0
    }
}

pub struct PublicKey(Vec<u8>);

impl crypto_provider::ed25519::PublicKey for PublicKey {
    type Signature = Signature;

    fn from_bytes(bytes: [u8; KEY_LENGTH]) -> Result<Self, InvalidBytes>
    where
        Self: Sized,
    {
        Ok(PublicKey(bytes.to_vec()))
    }

    fn to_bytes(&self) -> [u8; KEY_LENGTH] {
        //Should be length 32
        self.0.as_slice().try_into().unwrap()
    }

    fn verify_strict(
        &self,
        message: &[u8],
        signature: &Self::Signature,
    ) -> Result<(), SignatureError> {
        let public_key = PKey::public_key_from_raw_bytes(self.0.as_slice(), Id::ED25519)
            .expect("should be a valid public key");

        let mut verifier = Verifier::new_without_digest(public_key.as_ref())
            .expect("should be able to init verifier");
        let result = verifier
            .verify_oneshot(signature.0.as_slice(), message)
            .expect("should be able to call verify");
        if result {
            Ok(())
        } else {
            Err(SignatureError)
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::ed25519::Ed25519;
    use crypto_provider::ed25519::testing::{run_rfc_test_vectors, run_wycheproof_test_vectors};

    #[test]
    fn wycheproof_test_ed25519_openssl() {
        run_wycheproof_test_vectors::<Ed25519>()
    }

    #[test]
    fn rfc_test_ed25519_openssl() {
        run_rfc_test_vectors::<Ed25519>()
    }
}
