mod github;
mod bitbucket;

pub use github::GITHUB_QUERY;
pub use github::GITHUB_DELETE_COMMENT;
pub use github::GITHUB_ADD_COMMENT;
pub use github::GithubPullRequestPayload;
pub use github::GithubPayload;
pub use github::GithubErrorPayload;

pub use bitbucket::BitbucketChangesPayload;
pub use bitbucket::BitbucketActivitiesPayload;
pub use bitbucket::BitbucketPagePayload;

pub const SLOGAN: &'static str = "COPYRIGHT is missing";