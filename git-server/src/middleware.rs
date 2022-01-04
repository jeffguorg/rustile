pub mod token_extractor {
    use actix_web::{Error, FromRequest};
    use futures::future;
    use jsonwebtoken::{Algorithm, DecodingKey, Validation};
    use serde::{Deserialize, Serialize};

    pub struct JWTSecret(pub Box<Vec<u8>>);

    impl Default for JWTSecret {
        fn default() -> Self {
            Self(Default::default())
        }
    }

    #[derive(Debug, Serialize, Deserialize)]
    pub struct Token {
        sub: String,
        iat: chrono::DateTime<chrono::Utc>,

        pub command: String,
    }

    impl FromRequest for Token {
        type Error = actix_web::error::Error;

        type Future = futures::future::Ready<Result<Self, Error>>;

        type Config = JWTSecret;

        fn from_request(
            req: &actix_web::HttpRequest,
            _: &mut actix_web::dev::Payload,
        ) -> Self::Future {
            let jwtsecret = req.app_data::<JWTSecret>().unwrap();

            if let Some(header) = req.headers().get("Authorization") {
                if let Ok(token) = jsonwebtoken::decode::<Token>(
                    String::from_utf8(header.as_bytes().to_vec())
                        .unwrap()
                        .as_str(),
                    &DecodingKey::from_secret(&jwtsecret.0),
                    &Validation::new(Algorithm::HS256),
                ) {
                    return futures::future::ready(Ok(token.claims));
                }
            }
            futures::future::ready(Err(actix_web::error::ErrorUnauthorized("token needed")))
        }
    }
}
