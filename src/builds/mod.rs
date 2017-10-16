//! interfaces for interacting with travis builds

use {Branch, Client, Error, Future, Owner, Pagination, State};
use futures::{Future as StdFuture, IntoFuture, Stream};
use futures::future;
use futures::stream;
use hyper::client::Connect;
use jobs::Job;
use url::form_urlencoded::Serializer;

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
    pub state: State,
    pub duration: usize,
    pub event_type: String,
    pub previous_state: Option<State>,
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
    include: Vec<String>,
    limit: i32,
    /// id, started_at, finished_at, append :desc to any attribute to reverse order.
    sort_by: String,
    created_by: Option<String>,
    event_type: Option<String>,
    previous_state: Option<State>,
    state: Option<State>,
}

impl ListOptions {
    pub fn builder() -> ListOptionsBuilder {
        ListOptionsBuilder::default()
    }

    fn into_query_string(&self) -> String {
        let mut params = vec![
            ("include", self.include.join(",")),
            ("limit", self.limit.to_string()),
            ("sort_by", self.sort_by.clone()),
        ];
        if let &Some(ref created_by) = &self.created_by {
            params.push(("created_by", created_by.clone()));
        }
        if let &Some(ref event_type) = &self.event_type {
            params.push(("event_type", event_type.clone()));
        }
        if let &Some(ref previous_state) = &self.previous_state {
            params.push(("previous_state", previous_state.to_string()));
        }
        if let &Some(ref state) = &self.state {
            params.push(("state", state.to_string()));
        }
        Serializer::new(String::new()).extend_pairs(params).finish()
    }
}

impl Default for ListOptions {
    fn default() -> Self {
        ListOptions {
            include: Default::default(),
            limit: 25,
            sort_by: "started_at".into(),
            created_by: Default::default(),
            event_type: Default::default(),
            previous_state: Default::default(),
            state: Default::default(),
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
        let first = self.travis
            .get::<Wrapper>(
                format!(
                    "{host}/repo/{slug}/builds?{query}",
                    host = self.travis.host,
                    slug = self.slug,
                    query = options.into_query_string()
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
                    stream::unfold::<_, _, Future<(Build, Wrapper)>, _>(
                        wrapper,
                        move |mut state| match state.builds.pop() {
                            Some(build) => Some(
                                Box::new(future::ok((build, state))) as
                                    Future<(Build, Wrapper)>,
                            ),
                            _ => {
                                state.pagination.next.clone().map(|path| {
                                    Box::new(clone
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
                                        })) as Future<(Build, Wrapper)>
                                })
                            }
                        },
                    )
                })
                .into_stream()
                .flatten(),
        )
    }
}
