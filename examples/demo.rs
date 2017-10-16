extern crate futures;
extern crate tokio_core;
extern crate travis;

use futures::Stream;
use std::env;
use tokio_core::reactor::Core;
use travis::{Client, Credential, Result, State};
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
            println!("{:#?}", repo.slug);
            travis.builds(repo.slug).iter(
                &builds::ListOptions::builder()
                    .state(State::Started)
                    .build()
                    .unwrap(),
            )
        })
        .flatten()
        .for_each(|build| Ok(println!("{:#?}", build)));


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
