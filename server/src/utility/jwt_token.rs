use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, Algorithm, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use std::env::var;
use uuid::Uuid;

use crate::models::project_info::ProjectInfo;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Claims {
    pub iat: u64,
    pub exp: u64,
    pub issuer: String,
    pub issue: String,
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

    let current_time = Utc::now();

    let claims = Claims {
        iat: current_time.timestamp() as u64,
        exp: (current_time + Duration::minutes(180)).timestamp() as u64,
        issuer: "home_file_server".to_owned(),
        issue: "home_file_server".to_owned(),
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
            let current_time = Utc::now().timestamp() as u64;

            println!("claims: {:#?}", claims);

            if claims.iat > current_time {
                return Err(JwtError::InvalidToken);
            }

            if claims.exp <= current_time {
                return Err(JwtError::ExpiredToken);
            }

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

    let claims = extract_claims_from_token(token);

    match claims {
        Ok(claims) => Ok(claims),
        Err(error) => {
            println!("JwtError: {:?}", error);
            Err(error)
        }
    }
}

pub fn _regenerate_token(_token: String) -> String {
    "".to_owned()
}
