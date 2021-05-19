use crate::User;
use jsonwebtoken::{decode, Algorithm, Validation, DecodingKey, EncodingKey, Header, encode, errors};
use serde::{Deserialize, Serialize};
use std::time;
use std::ops::Add;
use crate::result::Result;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Claims {
    pub email: Option<String>,
    pub exp: usize,
    pub grants: Vec<String>,
}

impl Claims {
    pub fn encode(&self, encoding_key: &Vec<u8>) -> Result<String> {
        let encoding_key = EncodingKey::from_rsa_pem(encoding_key).unwrap();

        let result = encode(
            &Header::new(Algorithm::RS256),
            self,
            &encoding_key,
        )?;

        Ok(result)
    }

    pub fn decode(token: String, decoding_key: Vec<u8>) -> Result<Claims> {
        let decoding_key = DecodingKey::from_rsa_pem(&&decoding_key).unwrap();
        let result = decode::<Claims>(&token, &decoding_key, &Validation::new(Algorithm::RS256))?;

        Ok(result.claims)
    }
}

pub fn exp(duration: time::Duration) -> usize {
    time::SystemTime::now()
        .duration_since(time::UNIX_EPOCH)
        .expect("unable to get the current time")
        .add(duration)
        .as_secs() as usize
}

pub fn mk_token(claims: &Claims, encoding_key: &EncodingKey) -> Result<String, errors::Error> {
    let result = encode(
        &Header::new(Algorithm::RS256),
        claims,
        encoding_key,
    )?;

    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    use jsonwebtoken::{decode, DecodingKey, Validation};
    use openssl::rsa::Rsa;
    use std::ops::Add;
    use std::time;

    #[test]
    fn test_mk_token() {
        let rsa = Rsa::generate(4096).unwrap();

        let private_der = rsa.private_key_to_der().unwrap();
        let public_der = rsa.public_key_to_der_pkcs1().unwrap();

        let encoding_key = EncodingKey::from_rsa_der(&private_der);
        let decoding_key = DecodingKey::from_rsa_der(&public_der);

        let exp = time::SystemTime::now()
            .duration_since(time::UNIX_EPOCH)
            .expect("unable to get the current time")
            .add(time::Duration::from_secs(60 * 20))
            .as_secs();

        let test_claims = Claims {
            email: Some("a@b.c".to_string()),
            exp: exp as usize,
            grants: vec![],
        };

        let token = match encode(&Header::new(Algorithm::RS256), &test_claims, &encoding_key) {
            Ok(t) => t,
            Err(_) => {
                panic!();
            }
        };

        let token_data =
            match decode::<Claims>(&token, &decoding_key, &Validation::new(Algorithm::RS256)) {
                Ok(t) => t,
                Err(e) => {
                    eprintln!("{}", e);
                    panic!("Couldn't even decode")
                }
            };

        println!("{}", token_data.claims.email)
    }
}
