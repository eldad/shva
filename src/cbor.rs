use std::ops::{Deref, DerefMut};

use axum::{
    body::Bytes,
    extract::{FromRequest, Request},
    http::{header, HeaderValue, StatusCode},
    response::{IntoResponse, Response},
};
use http::HeaderMap;
use serde::{de::DeserializeOwned, Serialize};
use thiserror::Error;
use tracing::error;

/// CBOR Extractor / Response.
/// [RFC8949](https://datatracker.ietf.org/doc/html/rfc8949)
///
/// Request must have the `Content-Type` header set to `application/cbor`.
///
/// Uses the [`ciborium` crate](https://crates.io/crates/ciborium).
pub struct Cbor<T>(pub T);

impl<T> Deref for Cbor<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> DerefMut for Cbor<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<T> From<T> for Cbor<T> {
    fn from(inner: T) -> Self {
        Self(inner)
    }
}

fn assert_cbor_content_type(headers: &HeaderMap<HeaderValue>) -> Result<(), CborRejection> {
    let mime = headers
        .get(header::CONTENT_TYPE)
        .ok_or(CborRejection::ContentTypeMissing)?
        .to_str()
        .map_err(|_| CborRejection::ContentTypeInvalid)?
        .parse::<mime::Mime>()
        .map_err(|_| CborRejection::ContentTypeInvalid)?;

    if mime.type_() == "application" && mime.subtype() == "cbor" {
        Ok(())
    } else {
        Err(CborRejection::ContentTypeInvalid)
    }
}

impl<S, T> FromRequest<S> for Cbor<T>
where
    T: DeserializeOwned,
    S: Send + Sync,
{
    type Rejection = CborRejection;

    async fn from_request(req: Request, state: &S) -> Result<Self, Self::Rejection> {
        assert_cbor_content_type(req.headers())?;

        let bytes = Bytes::from_request(req, state).await?;
        let value: T = ciborium::de::from_reader(bytes.as_ref())?;
        let response = Ok(Cbor(value));

        if let Err(err) = &response {
            error!(error = %err, "error extracting cbor data")
        }

        response
    }
}

// https://github.com/hyperium/mime/issues/140
const MIME_APPLICATION_CBOR: &str = "application/cbor";

impl<T> IntoResponse for Cbor<T>
where
    T: Serialize,
{
    fn into_response(self) -> Response {
        let mut buf: Vec<u8> = Vec::new();
        let result = ciborium::ser::into_writer(&self.0, &mut buf);

        if let Err(err) = result {
            error!(error = %err, "Error rendering response as CBOR");
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                [(
                    header::CONTENT_TYPE,
                    HeaderValue::from_static(mime::TEXT_PLAIN_UTF_8.as_ref()),
                )],
                "Error rendering response as CBOR",
            )
                .into_response();
        }

        (
            [(header::CONTENT_TYPE, HeaderValue::from_static(MIME_APPLICATION_CBOR))],
            buf,
        )
            .into_response()
    }
}

#[derive(Debug, Error)]
pub enum CborRejection {
    #[error("content type is missing")]
    ContentTypeMissing,
    #[error("content type is not application/cbor")]
    ContentTypeInvalid,
    #[error("failed to buffer request body: {0}")]
    BytesRejection(#[from] axum::extract::rejection::BytesRejection),
    #[error("failed to deserialize: {0}")]
    CiboriumIOError(#[from] ciborium::de::Error<std::io::Error>),
}

impl CborRejection {
    fn status_code(&self) -> StatusCode {
        match self {
            Self::ContentTypeMissing | Self::ContentTypeInvalid => StatusCode::UNSUPPORTED_MEDIA_TYPE,
            Self::BytesRejection(_) => StatusCode::PAYLOAD_TOO_LARGE,
            Self::CiboriumIOError(err) => match err {
                // CBOR can be parsed, but the result does not align with the entity
                ciborium::de::Error::Semantic(_, _) => StatusCode::UNPROCESSABLE_ENTITY,
                // All other errors are client errors (syntax, I/O)
                _ => StatusCode::BAD_REQUEST,
            },
        }
    }
}

impl IntoResponse for CborRejection {
    fn into_response(self) -> Response {
        self.status_code().into_response()
    }
}
