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

use tracing::error;

use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};

#[derive(Debug)]
pub enum AppError {
    Doomed,
    Unlucky,
    Unforseen,
    TooLittleTooLate,
    Oops,
    ShouldNeverHappen,
    GenericError(String, String),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let status_code = match &self {
            AppError::Doomed => StatusCode::INTERNAL_SERVER_ERROR,
            AppError::Unlucky => StatusCode::MISDIRECTED_REQUEST,
            AppError::TooLittleTooLate => StatusCode::EXPECTATION_FAILED,
            AppError::Oops => StatusCode::IM_A_TEAPOT,
            AppError::GenericError(type_, message) => {
                error!("Internal error [{}]: `{}`", type_, message);
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

macro_rules! apperror_from {
    ($err:path) => {
        impl From<$err> for AppError {
            fn from(e: $err) -> Self {
                AppError::GenericError(stringify!($err).to_string(), e.to_string())
            }
        }
    };
}

apperror_from!(anyhow::Error);
