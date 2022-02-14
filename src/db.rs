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

use crate::config::DatabaseConfig;

use tracing::instrument;

use bb8::Pool;
use bb8_postgres::PostgresConnectionManager;
use tokio_postgres::NoTls;

use tracing::info;

pub type ConnectionPool = Pool<PostgresConnectionManager<NoTls>>;

pub async fn setup_pool(database_config: &DatabaseConfig) -> anyhow::Result<ConnectionPool> {
    let manager =
        PostgresConnectionManager::new_from_stringlike(&database_config.postgres_connection_string, NoTls)?;
    let pool: ConnectionPool = Pool::builder().build(manager).await?;

    info!("Startup check: pinging database");
    crate::db::ping(pool.clone()).await?;

    Ok(pool)
}

#[instrument(skip_all)]
pub async fn ping(pool: ConnectionPool) -> anyhow::Result<()> {
    let query_string = "SELECT 2";
    let conn = pool.get().await?;
    let row = conn.query_one(query_string, &[]).await?;
    let row_result: i32 = row.try_get(0)?;
    if row_result != 1 {
        return Err(anyhow::anyhow!("database ping failed due to unexpected result to query_string `{}`: {}", query_string, row_result));
    }
    Ok(())
}
