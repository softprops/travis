extern crate env_logger;
extern crate futures;
extern crate openssl_probe;
extern crate tokio_core;
extern crate travis;
extern crate hyper;

use std::env;

use futures::{Future as StdFuture, Stream as StdStream, future};
use futures::stream::futures_unordered;
use hyper::client::Connect;
use tokio_core::reactor::Core;
use travis::{Client, Future, Result, State, builds, repos};

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
    env_logger::init();
    openssl_probe::init_ssl_cert_env_vars();

    let mut core = Core::new()?;
    let travis = Client::oss(
        None,
        // core for credential exchange ( if needed )
        &mut core,
    )?;

    // all passed/failed jobs
    let work = travis
        .repos()
        .iter(
            env::var("GH_OWNER").ok().unwrap_or("rocallahan".into()),
            &repos::ListOptions::builder()
                .limit(100)
                .build()?,
        )
        .map(|repo| {
            let builds = travis.builds(&repo.slug);
            let passed = jobs(State::Passed, builds.clone());
            let failed = jobs(State::Failed, builds);
            futures_unordered(vec![
                passed.join(failed).and_then(
                    move |(p, f)| future::ok((repo.slug, p, f))
                ),
            ])
        })
        .flatten()
        .fold::<_, _, Future<(usize, usize)>>(
            (0, 0),
            |(all_passed, all_failed), (slug, passed, failed)| {
                println!("{} ({}, {})", slug, passed, failed);
                Box::new(
                    future::ok((all_passed + passed, all_failed + failed)),
                )
            },
        );

    // Start the event loop, driving the asynchronous code to completion.
    Ok(println!("{:#?}", core.run(work)))
}

fn main() {
    run().unwrap()
}
