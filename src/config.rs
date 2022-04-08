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

use anyhow::Result;
use std::collections::HashMap;

use serde::Deserialize;
use std::fs;

#[derive(Deserialize, Debug)]
pub struct Config {
    pub service: ServiceConfig,
    pub database: DatabaseConfig,
    pub apikeys: HashMap<String, String>,
}

#[derive(Deserialize, Debug)]
pub struct ServiceConfig {
    pub bind_address: String,
    pub max_concurrent_connections: Option<usize>,
    pub request_timeout_milliseconds: u64,
}

#[derive(Deserialize, Debug)]
pub struct DatabaseConfig {
    pub postgres_connection_string: String,
    pub connection_timeout_secs: Option<u64>,
}

impl Config {
    pub fn read(filename: &str) -> Result<Self> {
        Ok(toml::from_str(&fs::read_to_string(filename)?)?)
    }

    pub fn read_default() -> Result<Self> {
        let default_config_path = format!("{}.toml", env!("CARGO_PKG_NAME"));
        Config::read(&default_config_path)
    }
}
