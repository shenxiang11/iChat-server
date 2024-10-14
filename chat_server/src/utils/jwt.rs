use jwt_simple::claims::Claims;
use jwt_simple::common::VerificationOptions;
use jwt_simple::prelude::{Duration, Ed25519KeyPair, Ed25519PublicKey, EdDSAKeyPairLike, EdDSAPublicKeyLike};
use serde::{Deserialize, Serialize};

pub struct EncodingKey(Ed25519KeyPair);

pub struct DecodingKey(Ed25519PublicKey);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IdClaims {
    pub user_id: i64,
}

impl EncodingKey {
    pub fn load(pem: &str) -> Result<Self, jwt_simple::Error> {
        Ok(Self(Ed25519KeyPair::from_pem(pem)?))
    }

    pub fn sign(&self, user_id: i64) -> Result<String, jwt_simple::Error> {
        let claims = Claims::with_custom_claims(IdClaims { user_id }, Duration::from_secs(60 * 60 * 24 * 7));
        let claims = claims.with_issuer("ichat_server").with_audience("ichat_web");

        self.0.sign(claims)
    }
}

impl DecodingKey {
    pub fn load(pem: &str) -> Result<Self, jwt_simple::Error> {
        Ok(Self(Ed25519PublicKey::from_pem(pem)?))
    }

    pub fn verify(&self, token: &str) -> Result<i64, jwt_simple::Error> {
        let opts = VerificationOptions {
            allowed_issuers: Some(["ichat_server".to_string()].into_iter().collect()),
            allowed_audiences: Some(["ichat_web".to_string()].into_iter().collect()),
            ..Default::default()
        };

        let claims = self.0.verify_token::<IdClaims>(token, Some(opts))?;

        Ok(claims.custom.user_id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_jwt_sign_and_verify_should_work() {
        let encoding_key = EncodingKey::load(include_str!("../../fixtures/encoding.pem")).unwrap();
        let decoding_key = DecodingKey::load(include_str!("../../fixtures/decoding.pem")).unwrap();

        let token = encoding_key.sign(1).unwrap();
        let user_id = decoding_key.verify(&token).unwrap();

        assert_eq!(user_id, 1);
    }
}
