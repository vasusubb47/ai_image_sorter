use actix_web::{dev::ServiceRequest, Error, HttpMessage};
use actix_web_httpauth::extractors::{
    bearer::{self, BearerAuth},
    AuthenticationError,
};

use crate::utility::jwt_token::validate_token;

pub async fn jwt_validator(
    req: ServiceRequest,
    credentials: BearerAuth,
) -> Result<ServiceRequest, (Error, ServiceRequest)> {
    let jwt_token = credentials.token();
    match validate_token(jwt_token) {
        Ok(token) => {
            req.extensions_mut().insert(token);
            Ok(req)
        }
        Err(_) => {
            let config = req
                .app_data::<bearer::Config>()
                .cloned()
                .unwrap_or_default()
                .scope("");
            Err((AuthenticationError::from(config).into(), req))
        }
    }
}
