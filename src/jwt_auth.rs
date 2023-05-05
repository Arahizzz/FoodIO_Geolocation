use std::sync::Arc;
use axum::extract::State;
use axum::http::{header, Request, StatusCode};
use axum::Json;
use axum::middleware::Next;
use axum::response::IntoResponse;
use jsonwebtoken::{decode, DecodingKey, Validation};
use serde::{Serialize, Deserialize};
use tracing::error;

#[derive(Debug, Serialize)]
pub struct ErrorResponse {
    pub status: &'static str,
    pub message: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TokenClaims {
    pub sub: String,
    pub iat: usize,
}

static CONFIG: once_cell::sync::Lazy<JwtConfig> = once_cell::sync::Lazy::new(|| JwtConfig::init());

pub async fn auth<B>(
    mut req: Request<B>,
    next: Next<B>,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorResponse>)> {
    let token = req.headers()
        .get(header::AUTHORIZATION)
        .and_then(|auth_header| auth_header.to_str().ok())
        .and_then(|auth_value| {
            if auth_value.starts_with("Bearer ") {
                Some(auth_value[7..].to_owned())
            } else {
                None
            }
        });

    let token = token.ok_or_else(|| {
        let json_error = ErrorResponse {
            status: "fail",
            message: "You are not logged in, please provide token".to_string(),
        };
        (StatusCode::UNAUTHORIZED, Json(json_error))
    })?;

    let claims = decode::<TokenClaims>(
        &token,
        &DecodingKey::from_secret(CONFIG.jwt_secret.as_ref()),
        &Validation::default(),
    )
        .map_err(|e| {
            let json_error = ErrorResponse {
                status: "fail",
                message: "Invalid token".to_string(),
            };
            (StatusCode::UNAUTHORIZED, Json(json_error))
        })?
        .claims;

    req.extensions_mut().insert(UserId(claims.sub));
    Ok(next.run(req).await)
}

pub struct UserId(pub String);

pub struct JwtConfig {
    pub jwt_secret: String,
}

impl JwtConfig {
    pub fn init() -> JwtConfig {
        let jwt_secret = std::env::var("JWT_SECRET").unwrap_or("SECRET".to_string());
        JwtConfig {
            jwt_secret,
        }
    }
}
