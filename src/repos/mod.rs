//! interfaces for interacting with travis repositories

use super::{Branch, Client, Error, Future, Owner};
use futures::{Future as StdFuture, IntoFuture};
use futures::future;
use hyper::client::Connect;
use std::borrow::Cow;

#[derive(Debug, Deserialize)]
struct RepositoriesWrapper {
    pub repositories: Vec<Repository>,
}

#[derive(Debug, Deserialize)]
pub struct Repository {
    pub id: usize,
    pub name: String,
    pub slug: String,
    pub description: Option<String>,
    pub github_language: Option<String>,
    pub active: bool,
    pub private: bool,
    pub owner: Owner,
    #[serde(rename = "@permissions")]
    pub permissions: RepoPermissions,
    pub default_branch: Option<Branch>,
    pub starred: bool,
}

#[derive(Debug, Deserialize)]
pub struct RepoPermissions {
    pub read: bool,
    pub admin: bool,
    pub activate: bool,
    pub deactivate: bool,
    pub star: bool,
    pub unstar: bool,
    pub create_cron: bool,
    pub create_env_var: bool,
    pub create_key_pair: bool,
    pub delete_key_pair: bool,
    pub create_request: bool,
}

pub struct Repos<'a, C>
where
    C: Clone + Connect,
{
    pub(crate) travis: &'a Client<C>,
}

impl<'a, C> Repos<'a, C>
where
    C: Clone + Connect,
{
    /// get a list of repos for the a given owner (user or org)
    pub fn repos<'b, O>(&self, owner: O) -> Future<Vec<Repository>>
    where
        O: Into<Cow<'b, str>>,
    {
        Box::new(
            self.travis
                .get(
                    format!(
                        "{host}/owner/{owner}/repos",
                        host = self.travis.host,
                        owner = owner.into()
                    ).parse()
                        .map_err(Error::from)
                        .into_future(),
                )
                .and_then(|wrapper: RepositoriesWrapper| {
                    future::ok(wrapper.repositories)
                }),
        )
    }
}
