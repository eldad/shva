/*
 * MIT License
 *
 * Copyright (c) 2022 Eldad Zack
 *
 * Permission is hereby granted, free of charge, to any person obtaining a copy
 * of this software and associated documentation files (the "Software"), to deal
 * in the Software without restriction, including without limitation the rights
 * to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
 * copies of the Software, and to permit persons to whom the Software is
 * furnished to do so, subject to the following conditions:
 *
 * The above copyright notice and this permission notice shall be included in all
 * copies or substantial portions of the Software.
 *
 * THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
 * IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
 * FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
 * AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
 * LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
 * OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
 * SOFTWARE.
 *
 */

use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};
use tracing::error;

use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("doomed to fail")]
    Doomed,
    #[error("just plain unlucky")]
    Unlucky,
    #[error("unforseen consequences")]
    Unforseen,
    #[error("too little too late")]
    TooLittleTooLate,
    #[error("oops")]
    Oops,
    #[error("strange! this should never happen...")]
    ShouldNeverHappen,
    #[error("bytes rejection")]
    BytesRejection(#[from] axum::extract::rejection::BytesRejection),
    #[error("Ciborium IO error")]
    CiboriumIOError(#[from] ciborium::de::Error<std::io::Error>),
    #[error(transparent)]
    GenericError(#[from] anyhow::Error),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let status_code = match &self {
            AppError::Doomed => StatusCode::INTERNAL_SERVER_ERROR,
            AppError::Unlucky => StatusCode::MISDIRECTED_REQUEST,
            AppError::TooLittleTooLate => StatusCode::EXPECTATION_FAILED,
            AppError::Oops => StatusCode::IM_A_TEAPOT,
            AppError::GenericError(err) => {
                error!(error = %err, "Internal error: `{}`", err);
                StatusCode::INTERNAL_SERVER_ERROR
            }
            _ => StatusCode::NOT_IMPLEMENTED,
        };

        let body = match status_code {
            StatusCode::INTERNAL_SERVER_ERROR => "Internal server error".to_string(),
            _ => format!("{:?}", &self),
        };

        (status_code, body).into_response()
    }
}
