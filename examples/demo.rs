extern crate futures;
extern crate tokio_core;
extern crate travis;

use futures::{Future as StdFuture, Stream, future};
use futures::stream::futures_unordered;
use std::env;
use tokio_core::reactor::Core;
use travis::{Client, Credential, Future, Result, State};
//use travis::builds::ListOptions;

use travis::builds;
use travis::repos;

fn run() -> Result<()> {
    let mut core = Core::new()?;
    let travis = Client::pro(
        // authentication credentials
        env::var("GH_TOKEN").ok().map(
            |token| Credential::Github(token),
        ),
        // core for credential exchange ( if needed )
        &mut core,
    )?;

    // all pending jobs
    let work = travis
        .repos()
        .iter(
            env::var("GH_OWNER").ok().unwrap_or("softprops".into()),
            &repos::ListOptions::builder()
                .limit(100)
                .active(true)
                .build()?,
        )
        .map(|repo| {
            let slug = repo.slug;
            let running = travis
                .builds(slug.clone())
                .iter(&builds::ListOptions::builder()
                    .state(State::Started)
                    .build()
                    .unwrap())
                .fold::<_, _, Future<i64>>(
                    0,
                    |acc, _| Box::new(future::ok(acc + 1)),
                );

            let queued = travis
                .builds(slug.clone())
                .iter(&builds::ListOptions::builder()
                    .state(State::Created)
                    .build()
                    .unwrap())
                .fold::<_, _, Future<i64>>(
                    0,
                    |acc, _| Box::new(future::ok(acc + 1)),
                );
            futures_unordered(vec![
                running.and_then(
                    move |r| queued.map(move |q| (slug, r, q))
                ),
            ])
        })
        .flatten()
        .for_each(|(slug, running, queued)| {
            Ok(println!("{} ({}, {})", slug, running, queued))
        });

    // Start the event loop, driving the asynchronous code to completion.
    Ok(println!("{:#?}", core.run(work)))
}

fn main() {
    run().unwrap()
}
