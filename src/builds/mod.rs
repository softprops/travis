//! interfaces for interacting with travis builds

use super::{Branch, Client, Error, Future, Owner};
use futures::{Future as StdFuture, IntoFuture, Stream};
use futures::future;
use futures::stream;
use hyper::client::Connect;
use jobs::Job;
use url::form_urlencoded::Serializer;

#[derive(Debug, Deserialize, Clone)]
struct Pagination {
    count: usize,
    first: Page,
    next: Option<Page>,
}

#[derive(Debug, Deserialize, Clone)]
struct Page {
    #[serde(rename = "@href")]
    href: String,
}

#[derive(Debug, Deserialize, Clone)]
struct Wrapper {
    builds: Vec<Build>,
    #[serde(rename = "@pagination")]
    pagination: Pagination,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Build {
    pub id: usize,
    pub number: String,
    pub state: String,
    pub duration: usize,
    pub event_type: String,
    pub previous_state: Option<String>,
    pub pull_request_title: Option<String>,
    pub pull_request_number: Option<usize>,
    pub started_at: String,
    pub finished_at: Option<String>,
    // repository
    pub branch: Branch,
    // commit
    pub jobs: Vec<Job>,
    // stages
    pub created_by: Owner,
}

/// list options
#[derive(Builder, Debug)]
#[builder(setter(into), default)]
pub struct ListOptions {
    pub include: Vec<String>,
    pub limit: i32,
    /// id, started_at, finished_at, append :desc to any attribute to reverse order.
    pub sort_by: String,
}

impl ListOptions {
    pub fn builder() -> ListOptionsBuilder {
        ListOptionsBuilder::default()
    }

    fn into_query_string(&self) -> String {
        Serializer::new(String::new())
            .extend_pairs(vec![
                ("include", &self.include.join(",")),
                ("limit", &self.limit.to_string()),
                ("sort_by", &self.sort_by),
            ])
            .finish()
    }
}

impl Default for ListOptions {
    fn default() -> Self {
        ListOptions {
            include: Default::default(),
            limit: 25,
            sort_by: "started_at".into(),
        }
    }
}

#[derive(Clone)]
pub struct Builds<C>
where
    C: Clone + Connect,
{
    pub(crate) travis: Client<C>,
    pub(crate) slug: String,
}

impl<C> Builds<C>
where
    C: Clone + Connect,
{
    pub fn list(&self, options: &ListOptions) -> Future<Vec<Build>> {
        Box::new(
            self.travis
                .get(
                    format!(
                        "{host}/repo/{slug}/builds?{query}",
                        host = self.travis.host,
                        slug = self.slug,
                        query = options.into_query_string()
                    ).parse()
                        .map_err(Error::from)
                        .into_future(),
                )
                .and_then(|wrapper: Wrapper| future::ok(wrapper.builds)),
        )
    }

    pub fn iter(
        &self,
        options: &ListOptions,
    ) -> Box<stream::Stream<Item = Build, Error = super::Error>> {
        let query = Serializer::new(String::new())
            .extend_pairs(vec![
                ("limit", &options.limit.to_string()),
                ("sort_by", &options.sort_by),
            ])
            .finish();
        let first = self.travis
            .get::<Wrapper>(
                format!(
                    "{host}/repo/{slug}/builds?{query}",
                    host = self.travis.host,
                    slug = self.slug,
                    query = query
                ).parse()
                    .map_err(Error::from)
                    .into_future(),
            )
            .map(|mut wrapper: Wrapper| {
                let mut builds = wrapper.builds;
                builds.reverse();
                wrapper.builds = builds;
                wrapper
            });
        // needed to move "self" into the closure below
        let clone = self.clone();
        Box::new(
            first
                .map(move |wrapper| {
                    stream::unfold(wrapper, move |mut state| match state.builds.pop() {
                        Some(build) => Some(Box::new(future::ok((build, state))) as
                            Future<(Build, Wrapper)>),
                        _ => {
                            state.pagination.next.clone().map(|path| {
                                let f = clone
                                    .travis
                                    .get::<Wrapper>(
                                        format!(
                                            "{host}{path}",
                                            host = clone.travis.host,
                                            path = path.href
                                        ).parse()
                                            .map_err(Error::from)
                                            .into_future(),
                                    )
                                    .map(|mut next| {
                                        let mut builds = next.builds;
                                        builds.reverse();
                                        next.builds = builds;
                                        (next.builds.pop().unwrap(), next)
                                    });
                                Box::new(f) as Future<(Build, Wrapper)>
                            })
                        }
                    })
                })
                .into_stream()
                .flatten(),
        )
    }
}
