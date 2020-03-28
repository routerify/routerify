use crate::prelude::*;
use http::StatusCode;
use hyper::{header, Body, Response};
use serde::Serialize;

pub struct JsonResponse<T = ()>
where
    T: Serialize,
{
    inner: Inner<T>,
}

enum Inner<T> {
    Success(SuccessData<T>),
    Error(ErrorData),
}

#[derive(Serialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
struct SuccessData<T> {
    status: Status,
    code: u16,
    data: T,
}

#[derive(Serialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
struct ErrorData {
    status: Status,
    code: u16,
    message: String,
}

#[derive(Serialize, Debug, Clone)]
enum Status {
    #[serde(rename = "Success")]
    SUCCESS,
    #[serde(rename = "Failed")]
    FAILED,
}

impl<T: Serialize> JsonResponse<T> {
    pub fn with_success(code: StatusCode, data: T) -> Self {
        JsonResponse {
            inner: Inner::Success(SuccessData {
                status: Status::SUCCESS,
                code: code.as_u16(),
                data,
            }),
        }
    }
}

impl JsonResponse<()> {
    pub fn with_error<M: Into<String>>(code: StatusCode, message: M) -> Self {
        JsonResponse {
            inner: Inner::Error(ErrorData {
                status: Status::FAILED,
                code: code.as_u16(),
                message: message.into(),
            }),
        }
    }

    pub fn with_error_code(code: StatusCode) -> Self {
        Self::with_error(code, code.canonical_reason().unwrap().to_owned())
    }
}

impl<T: Serialize> JsonResponse<T> {
    pub fn into_response(self) -> crate::Result<Response<Body>> {
        let code;
        let body;

        match self.inner {
            Inner::Success(success_data) => {
                code = success_data.code;
                body = Body::from(
                    serde_json::to_vec(&success_data)
                        .context("JsonResponse: Failed to convert success data to JSON")?,
                );
            }
            Inner::Error(err_data) => {
                code = err_data.code;
                body = Body::from(
                    serde_json::to_vec(&err_data).context("JsonResponse: Failed to convert error data to JSON")?,
                );
            }
        }

        Ok(Response::builder()
            .status(StatusCode::from_u16(code).unwrap())
            .header(header::CONTENT_TYPE, "application/json")
            .body(body)
            .context("JsonResponse: Failed to create a response")?)
    }
}
