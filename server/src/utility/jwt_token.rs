use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, Algorithm, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use std::env::var;
use uuid::Uuid;

use crate::models::project_info::ProjectInfo;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Claims {
    pub nbf: u64,
    pub iat: u64,
    pub exp: u64,
    pub iss: String,
    pub aud: String,
    pub project_id: Uuid,
}

#[derive(Debug)]
pub enum JwtError {
    InvalidToken,
    ExpiredToken,
}

pub fn generate_token(project_info: &ProjectInfo) -> String {
    let jwt_secret =
        var("JWT_SECRET").expect("Couldn't find JWT SECRET from environment variable.");
    let jwt_issuer =
        var("JWT_ISSUER").expect("Couldn't find JWT_ISSUER from environment variable.");
    let jwt_audience =
        var("JWT_AUDIENCE").expect("Couldn't find JWT_AUDIENCE from environment variable.");

    let current_time = Utc::now();

    let claims = Claims {
        nbf: current_time.timestamp() as u64,
        iat: current_time.timestamp() as u64,
        exp: (current_time + Duration::minutes(180)).timestamp() as u64,
        iss: jwt_issuer,
        aud: jwt_audience,
        project_id: project_info.project_id,
    };

    let token_str = encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(jwt_secret.as_ref()),
    );

    token_str.unwrap()
}

fn extract_claims_from_token(token: &str) -> Result<Claims, JwtError> {
    let jwt_secret =
        var("JWT_SECRET").expect("Couldn't find JWT SECRET from environment variable.");

    let token_msg = decode::<Claims>(
        &token,
        &DecodingKey::from_secret(jwt_secret.as_ref()),
        &Validation::new(Algorithm::HS256),
    );

    match token_msg {
        Ok(token) => {
            let claims = token.claims;

            Ok(claims)
        }
        Err(error) => {
            println!(
                "Error occurred while trying to retrieve claims from the jwt token: {}",
                error
            );

            Err(JwtError::InvalidToken)
        }
    }
}

pub fn validate_token(token: &str) -> Result<Claims, JwtError> {
    println!("jwt token : {token}");
    let jwt_audience =
        var("JWT_AUDIENCE").expect("Couldn't find JWT_AUDIENCE from environment variable.");

    let claims = extract_claims_from_token(token);

    match claims {
        Ok(claims) => {
            let current_time = Utc::now().timestamp() as u64;

            println!("claims: {:#?}", claims);

            if claims.iat > current_time && claims.nbf > current_time {
                return Err(JwtError::InvalidToken);
            }

            if claims.exp <= current_time {
                return Err(JwtError::ExpiredToken);
            }

            if claims.aud != jwt_audience {
                return Err(JwtError::InvalidToken);
            }

            Ok(claims)
        }
        Err(error) => {
            println!("JwtError: {:?}", error);
            Err(error)
        }
    }
}

pub fn _regenerate_token(_token: String) -> String {
    "".to_owned()
}
