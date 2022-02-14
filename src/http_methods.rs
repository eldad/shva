/*
 * MIT License
 *
 * Copyright (c) 2021 Eldad Zack
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

use tracing::instrument;
use tracing::event;
use tracing::Level;

use rand;

use crate::apperror::AppError;

pub async fn default() -> String {
    "OK".to_string()
}

pub async fn error() -> Result<String, AppError> {
    Err(AppError::Doomed)
}

#[instrument]
fn generate_random_error() -> AppError {
    let r: u8 = rand::random();
    match r % 5 {
        0 => AppError::Unforseen,
        1 => AppError::Unlucky,
        2 => AppError::Doomed,
        3 => AppError::TooLittleTooLate,
        4 => AppError::Oops,
        _ => AppError::ShouldNeverHappen,
    }
}

pub async fn random_error() -> Result<String, AppError> {
    event!(Level::INFO, "generating random error");
    Err(generate_random_error())
}
