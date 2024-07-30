use octocrab::Error;
use octocrab::models::repos::Branch;
use octocrab::params::repos::Reference;

pub fn github_login() {}

pub struct PullRequestDisplay {
    title: String,
    body: String,
    author_avatar_url: String
}

pub async fn get_pulls(owner: &str, repository: &str) -> Result<Vec<PullRequestDisplay>, Error> {
    let pull_requests = octocrab::instance().pulls(owner, repository).list().send().await?;
    let mut vec = Vec::new();
    for pull_request in pull_requests {
        let title = pull_request.title.unwrap_or("Empty Title".to_string());
        let body = pull_request.body.unwrap_or("Empty Body".to_string());
        let author_avatar_url = pull_request.user.unwrap().avatar_url;
        vec.push(PullRequestDisplay { title, body, author_avatar_url: String::from(author_avatar_url) });
    }
    return Ok(vec);
}

pub async fn already_forked(source_owner: &str, target_owner: &str, project_name: &str) -> bool {
    let forks = octocrab::instance().repos(source_owner, project_name).list_forks().send().await.unwrap().items;
    for fork in forks {
        return fork.full_name.unwrap().split("/").collect::<Vec<&str>>()[0] == target_owner;
    }
    return false;
}

pub async fn fork_repository(source_owner: &str, project_name: &str) {
    octocrab::instance().repos(source_owner, project_name).create_fork().send().await.unwrap();
}

pub async fn get_default_branch(owner: &str, project_name: &str) -> Option<Branch> {
    let default_branch = octocrab::instance().repos(owner, project_name).get().await.unwrap().default_branch.unwrap();
    let branches = octocrab::instance().repos(owner, project_name).list_branches().send().await.unwrap().items;
    for branch in branches {
        if branch.name == default_branch {
            return Some(branch);
        }
    }
    return None;
}

pub async fn create_branch(owner: &str, project_name: &str, workspace_id: &str) {
    let branch = get_default_branch(owner, project_name).await.unwrap();
    octocrab::instance().repos(owner, project_name).create_ref(&Reference::Branch(workspace_id.to_string()), branch.commit.sha).await.unwrap();
}
