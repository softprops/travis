//! interfaces for interacting with travis jobs

use super::{Client, Error, Future, Owner};
use futures::{Future as StdFuture, IntoFuture};
use futures::future;
use hyper::client::Connect;

#[derive(Debug, Deserialize)]
struct JobsWrapper {
    jobs: Vec<Job>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Job {
    pub id: usize,
    // standard rep fields
    pub number: Option<String>,
    pub state: Option<String>,
    pub started_at: Option<String>,
    pub finished_at: Option<String>,
    //pub build:
    pub queue: Option<String>,
    //pub repository
    // pub commit
    pub owner: Option<Owner>,
    //pub stage
}

pub struct Jobs<'a, C>
where
    C: Clone + Connect,
{
    pub(crate) travis: &'a Client<C>,
    pub(crate) build_id: usize,
}

impl<'a, C> Jobs<'a, C>
where
    C: Clone + Connect,
{
    pub fn list(&self) -> Future<Vec<Job>> {
        Box::new(
            self.travis
                .get(
                    format!(
                        "{host}/build/{build_id}/jobs",
                        host = self.travis.host,
                        build_id = self.build_id
                    ).parse()
                        .map_err(Error::from)
                        .into_future(),
                )
                .and_then(|wrapper: JobsWrapper| future::ok(wrapper.jobs)),
        )
    }
}
