use std::cell::RefCell;
use std::fmt;
use std::pin::Pin;
use std::rc::Rc;
use std::task::{Context, Poll};

use crate::authorities::authorities::AuthorityService;
use crate::db::pg::Pool;
use crate::http_response::Response as JsonResponse;
use crate::jwt::Claims;
use crate::result::Error as BaseError;
use crate::PublicKey;

use actix_service::{Service, Transform};
use actix_web::dev::{Payload, ServiceRequest, ServiceResponse};
use actix_web::{
    error::ErrorUnauthorized,
    error::ResponseError,
    http::{self, HeaderMap, Method},
    Error, FromRequest, HttpMessage, HttpRequest, HttpResponse,
};
use futures::future::{err, ok, Either, Future, FutureExt, Ready};
use jsonwebtoken::{decode, Algorithm, DecodingKey, Validation};
use uuid::Uuid;
use base64::DecodeError;

#[derive(Clone)]
pub struct Jwt {
    pub skip_paths: Vec<String>,
    pub authority_service: AuthorityService,
}

impl Jwt {
    pub fn new(authority_service: AuthorityService, skip_paths: Vec<String>) -> Self {
        Jwt { authority_service, skip_paths }
    }
}

impl<S, B> Transform<S> for Jwt
where
    S: Service<Request = ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S: 'static,
    S::Future: 'static,
    B: 'static,
{
    type Request = ServiceRequest;
    type Response = ServiceResponse<B>;
    type Error = Error;
    type InitError = ();
    type Transform = JwtMiddleware<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ok(JwtMiddleware {
            service: Rc::new(RefCell::new(service)),
            skip_paths: self.skip_paths.clone(),
            authority_service: self.authority_service.clone(),
        })
    }
}
#[derive(Clone)]
pub struct JwtMiddleware<S> {
    service: Rc<RefCell<S>>,
    skip_paths: Vec<String>,
    authority_service: AuthorityService,
}

impl<S, B> Service for JwtMiddleware<S>
where
    S: Service<Request = ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S: 'static,
    S::Future: 'static,
    B: 'static,
{
    type Request = ServiceRequest;
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>>>>;

    fn poll_ready(&mut self, cx: &mut Context) -> Poll<Result<(), Self::Error>> {
        self.service.poll_ready(cx)
    }

    fn call(&mut self, req: ServiceRequest) -> Self::Future {
        let mut service = self.service.clone();

        if req.method() == Method::OPTIONS {
            return Box::pin(async move { service.call(req).await }) 
        }

        for path in self.skip_paths.iter() {
            if req.path().starts_with(path) {
                return Box::pin(async move { service.call(req).await }) 
            }
        }


        let authority_service = self.authority_service.clone();

        Box::pin(async move {
            match extract_claims(&req.headers(), authority_service).await {
                Ok(claims) => {
                    req.extensions_mut().insert(claims);
                    service.call(req).await
                },
                Err(err) => Err(ErrorUnauthorized(err)),
            }
        })
    }
}

type ClaimsResult = std::result::Result<Claims, ClaimsError>;

#[derive(Serialize)]
pub struct HttpError {
    errors: Vec<String>,
}

impl HttpError {
    pub fn from_error<E: std::error::Error>(err: E) -> Self {
        let err = err.to_string();

        HttpError { errors: vec![err] }
    }
}

#[derive(Debug)]
pub enum ClaimsError {
    NoHeader,
    NoClientKey,
    FailedSignature,
    DecodeError(DecodeError),
    JwtError(jsonwebtoken::errors::Error),
    JwtParseError(http::header::ToStrError),
    Other(Box<dyn std::error::Error>),
}

impl fmt::Display for ClaimsError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use ClaimsError::*;

        match &self {
            NoHeader => write!(f, "no authoriation header found"),
            NoClientKey => write!(f, "no client key header found"),
            FailedSignature => write!(f, "failed to validate token"),
            DecodeError(err) => write!(f, "error decoding jwt: {}", err),
            JwtError(err) => write!(f, "error getting jwt: {}", err),
            JwtParseError(err) => write!(f, "error getting jwt: {}", err),
            Other(err) => write!(f, "other error: {}", err),
        }
    }
}

impl std::error::Error for ClaimsError {
    fn description(&self) -> &str {
        "ClaimsError"
    }

    fn cause(&self) -> Option<&(dyn std::error::Error)> {
        None
    }
}

impl ResponseError for ClaimsError {
    fn error_response(&self) -> HttpResponse {
        HttpResponse::Unauthorized().json("not authorized")
    }
}

impl From<DecodeError> for ClaimsError {
    fn from(err: DecodeError) -> ClaimsError {
        Self::DecodeError(err)
    }
}

pub async fn extract_claims(
    headers: &HeaderMap,
    authority_service: AuthorityService,
) -> ClaimsResult {
    let client_key = headers
        .get("client-key")
        .ok_or(ClaimsError::NoClientKey)?
        .to_str()
        .map_err(|err| ClaimsError::Other(Box::new(err)))?
        .to_string();

    let client_key = Uuid::parse_str(&client_key).map_err(|err| ClaimsError::NoHeader)?;

    let public_keys = authority_service
        .key_pairs_by_client_key(client_key)
        .await
        .map_err(|err| ClaimsError::Other(err.into()))?;

    return decode_claims(headers, public_keys);
}

pub fn decode_claims(
    headers: &HeaderMap,
    public_keys: Vec<PublicKey>,
) -> ClaimsResult {
    let token = headers
        .get("authorization")
        .ok_or(ClaimsError::NoHeader)?
        .to_str()
        .map_err(|err| ClaimsError::JwtParseError(err))?
        .replace("Bearer ", "");

    let client_key = headers
        .get("client-key")
        .ok_or(ClaimsError::NoClientKey)?
        .to_str()
        .map_err(|err| ClaimsError::Other(Box::new(err)))?
        .to_string();

    let client_key = Uuid::parse_str(&client_key).map_err(|err| ClaimsError::NoHeader)?;

    for key in public_keys.into_iter() {
        let public_key = key.decoded_public_key()
            .map_err(|err| ClaimsError::FailedSignature)?;

        if let Ok(claim) = Claims::decode(token.clone(), public_key) {
            return Ok(claim);
        }
    }

    return Err(ClaimsError::FailedSignature);
}

impl FromRequest for Claims {
    type Config = ();
    type Error = ClaimsError;
    type Future = Ready<Result<Claims, ClaimsError>>;

    fn from_request(req: &HttpRequest, _: &mut Payload) -> Self::Future {
        match req.extensions().get::<Claims>() {
            Some(claims) => return ok(claims.clone()),
            None => return err(ClaimsError::NoHeader),
        }
    }
}
