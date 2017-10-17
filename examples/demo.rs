extern crate futures;
extern crate tokio_core;
extern crate travis;
extern crate hyper;

use futures::{Future as StdFuture, Stream as StdStream, future};
use futures::stream::futures_unordered;
use hyper::client::Connect;
use std::env;
use tokio_core::reactor::Core;
use travis::{Client, Credential, Future, Result, State, builds, repos};

fn jobs<C>(state: State, builds: builds::Builds<C>) -> Future<usize>
where
    C: Clone + Connect,
{
    Box::new(
        builds
            .iter(&builds::ListOptions::builder()
                .state(state.clone())
                .include(vec!["build.jobs".into()])
                .build()
                .unwrap())
            .fold::<_, _, Future<usize>>(0, move |acc, build| {
                Box::new(future::ok(
                    acc +
                        build
                            .jobs
                            .iter()
                            .filter(|job| Some(state.clone()) == job.state)
                            .count(),
                ))
            }),
    )
}

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
            let builds = travis.builds(repo.slug.as_ref());
            let started = jobs(State::Started, builds.clone());
            let created = jobs(State::Created, builds);
            futures_unordered(vec![
                started.and_then(
                    move |s| created.map(move |c| (repo.slug, s, c))
                ),
            ])
        })
        .flatten()
        .for_each(|(slug, started, created)| {
            Ok(println!("{} ({}, {})", slug, started, created))
        });

    // Start the event loop, driving the asynchronous code to completion.
    Ok(println!("{:#?}", core.run(work)))
}

fn main() {
    run().unwrap()
}
