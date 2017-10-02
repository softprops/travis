extern crate futures;
extern crate tokio_core;
extern crate travis;

use futures::Stream;
use std::env;
use tokio_core::reactor::Core;
use travis::{Client, Credential, Result};
use travis::builds::ListOptions;

fn run() -> Result<()> {
    let mut core = Core::new()?;
    let travis = Client::oss(
        // authentication credentials
        env::var("GH_TOKEN").ok().map(
            |token| Credential::Github(token),
        ),
        // core for credential exchange ( if needed )
        &mut core,
    )?;

    // all of the builds
    let work = travis
        .builds("softprops/codeowners")
        .iter(&ListOptions::builder()
            .limit(1)
            .include(vec!["jobs".into()])
            .build()?)
        .for_each(|build| Ok(println!("{:#?}", build)));

    // Start the event loop, driving the asynchronous code to completion.
    Ok(println!("{:#?}", core.run(work)))
}

fn main() {
    run().unwrap()
}
