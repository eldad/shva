use std::ops::{Deref, DerefMut};

use async_trait::async_trait;
use axum::{
    body::{Bytes, HttpBody},
    extract::{FromRequest, RequestParts},
    http::{header, HeaderValue, StatusCode},
    response::{IntoResponse, Response},
    BoxError,
};
use serde::{de::DeserializeOwned, Serialize};
use tracing::error;
use thiserror::Error;

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

fn assert_cbor_content_type<B>(req: &RequestParts<B>) -> Result<(), CborRejection> {
    let mime = req
        .headers()
        .get(header::CONTENT_TYPE)
        .ok_or(CborRejection::ContentTypeMissing)?
        .to_str().map_err(|_| CborRejection::ContentTypeInvalid)?
        .parse::<mime::Mime>().map_err(|_| CborRejection::ContentTypeInvalid)?;

    if mime.type_() == "application" && mime.subtype() == "cbor" {
        Ok(())
    } else {
        Err(CborRejection::ContentTypeInvalid)
    }
}

#[async_trait]
impl<B, T> FromRequest<B> for Cbor<T>
where
    T: DeserializeOwned,
    B: HttpBody + Send,
    B::Data: Send,
    B::Error: Into<BoxError>,
{
    type Rejection = CborRejection;

    async fn from_request(req: &mut RequestParts<B>) -> Result<Self, Self::Rejection> {
        assert_cbor_content_type(req)?;
        let bytes = Bytes::from_request(req).await?;
        let value: T = ciborium::de::from_reader(bytes.as_ref())?;
        Ok(Cbor(value))
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
    #[error("failed to buffer request body")]
    BytesRejection(#[from] axum::extract::rejection::BytesRejection),
    #[error("Ciborium IO error")]
    CiboriumIOError(#[from] ciborium::de::Error<std::io::Error>),
}

impl CborRejection {
    fn status_code(&self) -> StatusCode {
        match self {
            Self::ContentTypeMissing|Self::ContentTypeInvalid => StatusCode::UNSUPPORTED_MEDIA_TYPE,
            Self::CiboriumIOError(_) => StatusCode::BAD_REQUEST,
            Self::BytesRejection(_) => StatusCode::PAYLOAD_TOO_LARGE,
        }
    }
}

impl IntoResponse for CborRejection {
    fn into_response(self) -> Response {
        self.status_code().into_response()
    }
}