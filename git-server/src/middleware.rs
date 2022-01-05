pub mod token_extractor {
    use actix_web::{Error, FromRequest};
    use jsonwebtoken::{Algorithm, DecodingKey, Validation};
    use log::*;
    use serde::{Deserialize, Serialize};

    #[derive(Debug)]
    pub struct JWTSecret(pub String);

    impl Default for JWTSecret {
        fn default() -> Self {
            Self(Default::default())
        }
    }

    lazy_static::lazy_static! {
        static ref AUTH_CAPTURE: regex::Regex = regex::Regex::new("(?P<kind>[[:alpha:]]*) (?P<cred>.*)").unwrap();
    }

    #[derive(Debug, Serialize, Deserialize)]
    pub struct Token {
        sub: String,
        iat: i64,
        exp: i64,

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
            debug!("extracting token info");

            let jwtsecret = req.app_data::<JWTSecret>().unwrap();
            debug!("jwt secret ok: {:?}", jwtsecret);

            if let Some(header) = req.headers().get("Authorization") {
                match AUTH_CAPTURE.captures(
                    String::from_utf8(header.as_bytes().to_vec())
                        .unwrap()
                        .as_str(),
                ) {
                    Some(captures) => {
                        if captures.name("kind").unwrap().as_str().to_lowercase() == "token" {
                            match jsonwebtoken::decode::<Token>(
                                captures.name("cred").unwrap().as_str(),
                                &DecodingKey::from_secret(jwtsecret.0.as_bytes()),
                                &Validation::new(Algorithm::HS256),
                            ) {
                                Ok(token) => return futures::future::ready(Ok(token.claims)),
                                Err(err) => debug!("failed to decode token: {}", err),
                            }
                        } else {
                            debug!("is not token auth");
                        }
                    }
                    None => debug!("not valid auth"),
                }
            }

            debug!("auth needed");
            futures::future::ready(Err(actix_web::error::ErrorUnauthorized("auth needed")))
        }
    }
}
