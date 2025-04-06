use jsonwebtoken::{decode, encode, Algorithm, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use chrono::{Duration, Utc};
use actix_web::{HttpRequest, dev::{Service, ServiceRequest, ServiceResponse, Transform}, Error};
use futures_util::future::{ok, Ready, FutureExt};
use crate::errors::ServiceError;

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,    // user id
    pub exp: usize,     // expiration timestamp
    pub iat: usize,     // issued at timestamp
    pub iss: String,    // issuer
}

// Configuración JWT
const JWT_ALGORITHM: Algorithm = Algorithm::HS256;
const TOKEN_EXPIRATION_HOURS: i64 = 24;
const JWT_ISSUER: &str = "notes-app";

pub struct CheckLogin;

impl<S, B> Transform<S, ServiceRequest> for CheckLogin
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Transform = CheckLoginMiddleware<S>;
    type InitError = ();
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ok(CheckLoginMiddleware { service })
    }
}

pub struct CheckLoginMiddleware<S> {
    service: S,
}

impl<S, B> Service<ServiceRequest> for CheckLoginMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = futures_util::future::LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    fn poll_ready(&self, ctx: &mut core::task::Context<'_>) -> core::task::Poll<Result<(), Self::Error>> {
        self.service.poll_ready(ctx)
    }

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let token = req.headers()
            .get("Authorization")
            .and_then(|h| h.to_str().ok())
            .and_then(|s| s.split("Bearer ").nth(1));

        if token.is_none() {
            return futures::future::ready(Err(Error::from(ServiceError::unauthorized("Missing token")))).boxed_local();
        }

        let fut = self.service.call(req);
        async move {
            fut.await
        }.boxed_local()
    }
}

pub fn create_jwt(user_id: &str) -> Result<String, ServiceError> {
    let now = Utc::now();
    let expiration = now
        .checked_add_signed(Duration::hours(TOKEN_EXPIRATION_HOURS))
        .ok_or_else(|| ServiceError::internal_server_error("Invalid timestamp calculation"))?;
    
    let claims = Claims {
        sub: user_id.to_owned(),
        exp: expiration.timestamp() as usize,
        iat: now.timestamp() as usize,
        iss: JWT_ISSUER.to_string(),
    };
    
    let secret = std::env::var("JWT_SECRET")
        .map_err(|_| ServiceError::internal_server_error("JWT_SECRET not set in environment"))?;
    
    encode(
        &Header::new(JWT_ALGORITHM),
        &claims,
        &EncodingKey::from_secret(secret.as_ref())
    ).map_err(|e| ServiceError::internal_server_error(&format!("JWT encoding failed: {}", e)))
}

pub fn get_user_id_from_token(req: &HttpRequest) -> Result<String, ServiceError> {
    let token = req.headers()
        .get("Authorization")
        .ok_or_else(|| ServiceError::unauthorized("Authorization header missing"))?
        .to_str()
        .map_err(|_| ServiceError::unauthorized("Invalid authorization header"))?
        .split("Bearer ")
        .nth(1)
        .ok_or_else(|| ServiceError::unauthorized("Invalid bearer token format"))?;

    validate_jwt(token)
}

pub fn validate_jwt(token: &str) -> Result<String, ServiceError> {
    let secret = std::env::var("JWT_SECRET")
        .map_err(|_| ServiceError::internal_server_error("JWT_SECRET not set in environment"))?;

    let validation = Validation {
        iss: Some(JWT_ISSUER.to_string()),
        algorithms: vec![JWT_ALGORITHM],
        ..Validation::new(JWT_ALGORITHM)
    };

    let token_data = decode::<Claims>(
        token,
        &DecodingKey::from_secret(secret.as_ref()),
        &validation,
    ).map_err(|e| ServiceError::unauthorized(&format!("Invalid token: {}", e)))?;

    Ok(token_data.claims.sub)
}