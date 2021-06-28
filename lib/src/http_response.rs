use crate::result::{Error, Result};
use actix_web::HttpResponse;
use serde::Serialize;

#[derive(Serialize)]
pub struct Response<T>{
    success: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    payload: Option<T>,
    #[serde(skip_serializing_if = "Option::is_none")]
    errors: Option<Vec<String>>,
}

impl<T: Serialize> Response<T> {
    pub fn from_result(result: Result<T>) -> Response<T> {
        match result {
            Ok(payload) => Response::payload(payload),
            Err(err) => Response::error(err),
        }
    }

    pub fn payload(payload: T) -> Self {
        Self {
            success: true,
            payload: Some(payload),
            errors: None,
        }
    }

    pub fn error(err: Error) -> Self {
        let errors = vec![err.to_string()];

        Self {
            success: false,
            payload: None,
            errors: Some(errors),
        }
    }

    pub fn errors(errs: Vec<Error>) -> Self {
        let errors = errs.into_iter().map(|err| err.to_string()).collect();

        Self {
            success: false,
            payload: None,
            errors: Some(errors),
        }
    }

    pub fn json(&self) -> HttpResponse {
        HttpResponse::Ok()
            .json(self)
    }
}
