extern crate futures;
extern crate tokio_core;
extern crate travis;

use futures::{Future as StdFuture, Stream, future};
use futures::stream::futures_unordered;
use std::env;
use tokio_core::reactor::Core;
use travis::{Client, Credential, Error, Future, Result, State};
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
                .fold(
                    0,
                    |acc, _| Box::new(future::ok(acc + 1)) as Future<usize>,
                );

            let queued = travis
                .builds(slug.clone())
                .iter(&builds::ListOptions::builder()
                    .state(State::Created)
                    .build()
                    .unwrap())
                .fold(
                    0,
                    |acc, _| Box::new(future::ok(acc + 1)) as Future<usize>,
                );
            futures_unordered(vec![
                running.and_then(
                    move |r| queued.map(move |q| (slug, r, q))
                ),
            ]).map_err(Error::from)
        })
        .flatten()
        .for_each(|(slug, running, queued)| {
            Ok(println!("{} ({}, {})", slug, running, queued))
        });


    // all of the builds
    /*let work = travis
        .builds("softprops/codeowners")
        .iter(&ListOptions::builder()
            .limit(1)
            .include(vec!["jobs".into()])
            .build()?)
        .for_each(|build| Ok(println!("{:#?}", build)));*/

    // Start the event loop, driving the asynchronous code to completion.
    Ok(println!("{:#?}", core.run(work)))
}

fn main() {
    run().unwrap()
}
