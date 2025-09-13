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

use std::collections::HashMap;

use axum::{
    http::{Response, StatusCode},
    response::IntoResponse,
};
use hyper::Request;
use tower_http::validate_request::ValidateRequest;

#[derive(Clone)]
pub struct ApiKeyAuth {
    apikeys: HashMap<String, String>,
}

const APIKEY_HEADER: &str = "x-auth-api-key";

impl ApiKeyAuth {
    pub fn from_apikeys(apikeys: HashMap<String, String>) -> Self {
        Self { apikeys }
    }
}

#[derive(Debug, Clone)]
pub struct UserId(pub String);

impl<B> ValidateRequest<B> for ApiKeyAuth {
    type ResponseBody = axum::body::Body;

    fn validate(&mut self, request: &mut Request<B>) -> Result<(), Response<Self::ResponseBody>> {
        let user_id = request
            .headers()
            .get(APIKEY_HEADER)
            .and_then(|key| key.to_str().ok())
            .and_then(|key| self.apikeys.get(key))
            .map(|user_id| UserId(user_id.clone()));
        match user_id {
            Some(user_id) => {
                request.extensions_mut().insert(user_id);
                Ok(())
            }
            None => Err(StatusCode::UNAUTHORIZED.into_response()),
        }
    }
}
