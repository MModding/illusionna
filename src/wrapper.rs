use crate::osc;
use either::Either;
use http::header::ACCEPT;
use http::Uri;
use octocrab::auth::{Continue, DeviceCodes, OAuth};
use octocrab::models::repos::Branch;
use octocrab::models::Repository;
use octocrab::params::repos::Reference;
use octocrab::{Error, Octocrab, OctocrabBuilder};
use secrecy::{Secret, SecretString};
use serde_json::json;
use std::time::Duration;
use iced::widget::image;
use reqwest::Url;

pub async fn oauth_process() -> octocrab::Result<Octocrab> {
    let crab = Octocrab::builder()
        .base_uri("https://github.com").unwrap()
        .add_header(ACCEPT, "application/json".to_string())
        .build()?;
    let codes = start_authorization(&crab).await?;
    webbrowser::open(&codes.verification_uri).expect("...");
    cli_clipboard::set_contents(String::from(&codes.user_code)).expect("...");
    let oauth = wait_confirm(&crab, codes).await?;
    OctocrabBuilder::oauth(OctocrabBuilder::new(), oauth).build()
}

pub async fn start_authorization(crab: &Octocrab) -> octocrab::Result<DeviceCodes> {
    let client_id: Secret<String> = SecretString::new(osc::GITHUB_ID.to_string());
    Ok(crab.authenticate_as_device(&client_id, ["repo"]).await?)
}

pub async fn wait_confirm(crab: &Octocrab, codes: DeviceCodes) -> octocrab::Result<OAuth> {
    let mut interval = Duration::from_secs(codes.interval);
    let mut clock = tokio::time::interval(interval);
    let oauth = loop {
        clock.tick().await;
        match codes.poll_once(crab, &SecretString::new(osc::GITHUB_ID.to_string())).await? {
            Either::Left(auth) => break auth,
            Either::Right(cont) => match cont {
                Continue::SlowDown => {
                    interval += Duration::from_secs(5);
                    clock = tokio::time::interval(interval);
                    clock.tick().await;
                }
                _ => {}
            },
        }
    };
    Ok(oauth)
}

pub async fn get_image(url: Url) -> Result<image::Handle, reqwest::Error> {
    Ok(image::Handle::from_bytes(reqwest::get(url).await?.bytes().await?))
}

pub struct UserDisplay {
    name: String,
    icon: Uri,
    bio: String,
}

pub async fn get_user(crab: &Octocrab) {
    // println!("{}", crab.get_page().await.unwrap().url)
    crab.current().user().await.unwrap().url;
    /* crab.users()
    return UserDisplay {
        name: crab.current().user().await.unwrap();
    }; */
}

pub async fn get_forked_repositories(crab: &Octocrab) -> Vec<Repository> {
    crab.current()
        .list_repos_for_authenticated_user()
        .send()
        .await
        .unwrap()
        .items
        .into_iter()
        .filter(|x| x.fork.unwrap())
        .collect::<Vec<Repository>>()
}

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
    Ok(vec)
}

pub async fn already_forked(crab: &Octocrab, source_owner: &str, fork_owner: &str, project_name: &str) -> bool {
    let forks = crab.repos(source_owner, project_name).list_forks().send().await.unwrap().items;
    for fork in forks {
        return fork.full_name.unwrap().split("/").collect::<Vec<&str>>()[0] == fork_owner;
    }
    false
}

pub async fn fork_repository(crab: &Octocrab, source_owner: &str, project_name: &str) {
    crab.repos(source_owner, project_name).create_fork().send().await.unwrap();
}

pub async fn get_default_branch(crab: &Octocrab, owner: &str, project_name: &str) -> Option<Branch> {
    let default_branch = crab.repos(owner, project_name).get().await.unwrap().default_branch?;
    let branches = crab.repos(owner, project_name).list_branches().send().await.unwrap().items;
    for branch in branches {
        if branch.name == default_branch {
            return Some(branch);
        }
    }
    None
}

pub async fn sync_default_branch(crab: &Octocrab, owner: &str, project_name: &str) -> () {
    let route = format!(
        "/repos/{owner}/{repo}/merge-upstream",
        owner = owner,
        repo = project_name,
    );
    crab.post(
        route,
        Some(&json!({
            "owner": owner,
            "repo": project_name,
            "branch": get_default_branch(crab, owner, project_name).await.unwrap().name
        })),
    ).await.expect("Syncing did not work correctly")
}

pub async fn create_branch(crab: &Octocrab, owner: &str, project_name: &str, workspace_id: &str) {
    let branch = get_default_branch(crab, owner, project_name).await.unwrap();
    crab.repos(owner, project_name).create_ref(&Reference::Branch(workspace_id.to_string()), branch.commit.sha).await.unwrap();
}

pub async fn create_draft_pull_request(crab: &Octocrab, source_owner: &str, fork_owner: &str, project_name: &str, workspace_title: &str, workspace_id: &str, workspace_description: &str) {
    octocrab::instance()
        .pulls(source_owner, project_name)
        .create(
            workspace_title,
            fork_owner.to_string() + ":" + workspace_id,
            get_default_branch(crab, source_owner, fork_owner).await.unwrap().name
        )
        .body(workspace_description)
        .draft(true)
        .send().await.unwrap();
}
