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
use tokio_postgres::NoTls;
use anyhow::anyhow;

mod embedded {
    refinery::embed_migrations!();
}

pub fn verify_migration_versioning() -> anyhow::Result<()> {
    let runner = embedded::migrations::runner();
    let migrations = runner.get_migrations();

    if let Some(migration) = migrations.iter().find(|migration| format!("{}", migration.prefix()) == "U") {
        return Err(anyhow!("Unversioned migrations are prohibited: `{}`", migration.name()));
    }

    // Use a BTreeMap to get the keys sorted already when checking for discontinuous versions.
    let versions: BTreeMap<_, usize> = migrations.iter().fold(BTreeMap::new(),
        |mut map, migration| {
            *map.entry(migration.version()).or_insert(0) += 1;
            map
        }
    );

    let duplicates: Vec<_> = versions.iter().filter(|&(_, &frequency)| frequency > 1).map(|(k, _)| k).collect();
    if !duplicates.is_empty() {
        return Err(anyhow!("Non-unique versions of migrations are prohibited. Duplicate versions: {:?}", duplicates))
    }

    let (discontinuous, _) = versions.keys().fold((Vec::new(), None), |(mut gaps, last), &version| {
        if let Some(last) = last {
            if last + 1 != version {
                if last + 1 == version - 1 {
                    gaps.push(format!("Missing V{}", last + 1));
                } else {
                    gaps.push(format!("Gap V{}->V{}", last, version));
                }
            }
        }
        (gaps, Some(version))
    });
    if !discontinuous.is_empty() {
        return Err(anyhow!("Discontinuous versions of migrations are prohibited: `{:?}`", discontinuous))
    }

    Ok(())
}

pub(crate) async fn refinery_migrate(postgres_connection_string: &str, dryrun: bool) -> anyhow::Result<()> {

    let (mut client, connection) =
        tokio_postgres::connect(postgres_connection_string, NoTls).await?;

    tokio::spawn(async move {
        connection.await.unwrap();
    });

    let runner = embedded::migrations::runner();

    println!("Applied migrations:");
    let mut applied_migrations = runner
        .get_applied_migrations_async(&mut client)
        .await?
        .into_iter()
        .map(|migration| (migration.name().to_owned(), migration.applied_on().cloned()))
        .collect::<HashMap<String, _>>();

    let migrations = runner.get_migrations();
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
    }

    Ok(())
}
