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

use std::collections::{BTreeMap, HashMap};

use anyhow::anyhow;
use tokio_postgres::NoTls;
use tracing::error;

mod embedded {
    refinery::embed_migrations!();
}

pub fn verify_migration_versioning() -> anyhow::Result<()> {
    let runner = embedded::migrations::runner();
    let migrations = runner.get_migrations();

    let mut is_error = false;

    let unversioned: Vec<_> = migrations
        .iter()
        .filter(|migration| format!("{}", migration.prefix()) != "V")
        .map(|migration| format!("U{}", migration.version()))
        .collect();
    if !unversioned.is_empty() {
        is_error = true;
        error!("Unversioned migrations are prohibited: `{:?}`", unversioned);
    }

    // Use a BTreeMap to get the keys sorted already when checking for discontinuous versions.
    let versions: BTreeMap<_, usize> = migrations.iter().fold(BTreeMap::new(), |mut map, migration| {
        *map.entry(migration.version()).or_insert(0) += 1;
        map
    });

    let duplicates: Vec<_> = versions
        .iter()
        .filter(|&(_, &frequency)| frequency > 1)
        .map(|(k, _)| k)
        .collect();
    if !duplicates.is_empty() {
        is_error = true;
        error!(
            "Non-unique versions of migrations are prohibited. Duplicate versions: {:?}",
            duplicates
        );
    }

    let (discontinuous, _) = versions.keys().fold((Vec::new(), None), |(mut gaps, last), &version| {
        if let Some(last) = last {
            if last + 1 == version - 1 {
                gaps.push(format!("Missing V{}", last + 1));
            } else if last + 1 != version {
                gaps.push(format!("Gap V{}->V{}", last, version));
            }
        }
        (gaps, Some(version))
    });
    if !discontinuous.is_empty() {
        is_error = true;
        error!(
            "Discontinuous versions of migrations are prohibited: `{:?}`",
            discontinuous
        );
    }

    match is_error {
        true => Err(anyhow!("migration versioning violations detected")),
        false => Ok(()),
    }
}

pub(crate) async fn refinery_migrate(postgres_connection_string: &str, dryrun: bool) -> anyhow::Result<()> {
    if !dryrun {
        verify_migration_versioning()?;
    }

    let (mut client, connection) = tokio_postgres::connect(postgres_connection_string, NoTls).await?;

    tokio::spawn(async move {
        connection.await.unwrap();
    });

    let runner = embedded::migrations::runner();

    let mut applied_migrations = runner
        .get_applied_migrations_async(&mut client)
        .await?
        .into_iter()
        .map(|migration| (migration.name().to_owned(), migration.applied_on().cloned()))
        .collect::<HashMap<String, _>>();

    println!("Applied migrations:");

    let mut migrations: Vec<&refinery::Migration> = runner.get_migrations().iter().collect();
    migrations.sort_by_key(|a| a.version());
    migrations.iter().for_each(|migration| {
        let applied_on = applied_migrations
            .remove(migration.name())
            .flatten()
            .map(|applied_on| format!("{}", applied_on))
            .unwrap_or_else(|| String::from("-"));

        println!(
            "{} | {} | {} | {:#016x} ",
            migration.version(),
            applied_on,
            migration.name(),
            migration.checksum()
        );
    });

    if !dryrun {
        println!("Running migrations");
        runner.run_async(&mut client).await?;
        println!("Success!");
    } else {
        verify_migration_versioning()?;
    }

    Ok(())
}
