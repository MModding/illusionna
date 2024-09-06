use base64::{DecodeError, Engine};
use either::Either;
use http::header::ACCEPT;
use iced::widget::image;
use octocrab::auth::{Continue, DeviceCodes, OAuth};
use octocrab::models::pulls::PullRequest;
use octocrab::models::repos::{Branch, CommitObject, Object};
use octocrab::models::Repository;
use octocrab::params::repos::Reference;
use octocrab::params::State;
use octocrab::{Octocrab, OctocrabBuilder};
use reqwest::Url;
use secrecy::{ExposeSecret, SecretString};
use serde::{Deserialize, Serialize};
use std::convert::Into;
use std::string::ToString;
use std::time::Duration;

pub (crate) const ILLUSIONNA_GITHUB_APP: &str = env!("ILLUSIONNA_GITHUB_APP");

#[derive(Clone, Serialize, Deserialize)]
pub struct OAuthData {
    pub access_token: String,
    pub token_type: String,
    pub scope: Vec<String>,
    pub expires_in: Option<usize>,
    pub refresh_token: Option<String>,
    pub refresh_token_expires_in: Option<usize>,
}

impl From<OAuth> for OAuthData {
    fn from(oauth: OAuth) -> Self {
        OAuthData {
            access_token: oauth.access_token.expose_secret().to_string(),
            token_type: oauth.token_type.to_string(),
            scope: oauth.scope,
            expires_in: oauth.expires_in,
            refresh_token: oauth.refresh_token.map(|rtk| rtk.expose_secret().to_string()),
            refresh_token_expires_in: oauth.refresh_token_expires_in,
        }
    }
}

impl Into<OAuth> for OAuthData {
    fn into(self) -> OAuth {
        OAuth {
            access_token: SecretString::from(self.access_token),
            token_type: self.token_type,
            scope: self.scope,
            expires_in: self.expires_in,
            refresh_token: self.refresh_token.map(|rtk| SecretString::from(rtk)),
            refresh_token_expires_in: self.refresh_token_expires_in,
        }
    }
}

pub fn get_stored_token() -> Option<OAuth> {
    let username = whoami::username();
    let entry = keyring::Entry::new("illusionna-token-storage", &username).ok()?;
    let result: Option<OAuthData> = serde_json::from_slice(&entry.get_secret().ok()?).ok();
    result.map(|data| data.into())
}

pub fn set_stored_token(oauth: OAuth) -> octocrab::Result<OAuth> {
    let username = whoami::username();
    let entry = keyring::Entry::new("illusionna-token-storage", &username).unwrap();
    entry.set_secret(&serde_json::to_vec(&OAuthData::from(oauth.clone())).unwrap()).unwrap();
    Ok(oauth)
}

pub async fn embedded_oauth_process() -> octocrab::Result<Octocrab> {
    let oauth: OAuth = match get_stored_token() {
        Some(oauth) => oauth,
        None => set_stored_token(oauth_process().await.unwrap()).unwrap()
    };
    let crab = OctocrabBuilder::oauth(OctocrabBuilder::new(), oauth).build().ok();
    match crab {
        Some(x) => Ok(x),
        None => OctocrabBuilder::oauth(OctocrabBuilder::new(), set_stored_token(oauth_process().await.unwrap()).unwrap()).build()
    }
}

pub async fn oauth_process() -> octocrab::Result<OAuth> {
    let crab = Octocrab::builder()
        .base_uri("https://github.com").unwrap()
        .add_header(ACCEPT, "application/json".to_string())
        .build()?;
    let codes = start_authorization(&crab).await.unwrap();
    open::that(&codes.verification_uri).expect("...");
    cli_clipboard::set_contents(String::from(&codes.user_code)).expect("...");
    wait_confirm(&crab, codes).await
}

pub async fn start_authorization(crab: &Octocrab) -> octocrab::Result<DeviceCodes> {
    Ok(crab.authenticate_as_device(&SecretString::new(ILLUSIONNA_GITHUB_APP.to_string()), ["repo"]).await.unwrap())
}

pub async fn wait_confirm(crab: &Octocrab, codes: DeviceCodes) -> octocrab::Result<OAuth> {
    let mut interval = Duration::from_secs(codes.interval);
    let mut clock = tokio::time::interval(interval);
    let oauth = loop {
        clock.tick().await;
        match codes.poll_once(crab, &SecretString::new(ILLUSIONNA_GITHUB_APP.to_string())).await.unwrap() {
            Either::Left(oauth) => break oauth,
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

#[derive(Debug, Clone)]
pub struct AccountInfo {
    pub name: String,
    pub avatar: image::Handle,
    pub count: usize,
    pub profile: Url
}

pub async fn get_account_info(crab: Octocrab, count: usize) -> AccountInfo {
    let author = crab.current().user().await.unwrap();
    AccountInfo {
        name: author.login,
        avatar: get_image(author.avatar_url).await.unwrap(),
        count,
        profile: author.html_url
    }
}

pub async fn get_forked_repositories(crab: &Octocrab) -> Vec<Repository> {
    crab.current()
        .list_repos_for_authenticated_user()
        .per_page(100)
        .send()
        .await
        .unwrap()
        .items
        .into_iter()
        .filter(|x| x.fork.unwrap())
        .collect::<Vec<Repository>>()
}

pub async fn repository_exists(crab: Octocrab, author: String, project: String) -> bool {
    crab.repos(author, project).get().await.is_ok()
}

pub async fn fork_repository(crab: Octocrab, source_owner: &str, project_name: &str) -> Repository {
    crab.repos(source_owner, project_name).create_fork().send().await.unwrap()
}

pub async fn get_pull_requests(crab: &Octocrab, owner: &str, project_name: &str, all: bool) -> Vec<PullRequest> {
    let name = crab.current().user().await.unwrap().login;
    match crab.pulls(owner, project_name).list().state(if all {State::All} else {State::Open}).per_page(100).send().await {
        Ok(pulls) => {
            pulls.items.into_iter().filter(|pull| {
                match &pull.user {
                    Some(author) => author.login == name,
                    None => false
                }
            }).collect::<Vec<PullRequest>>()
        }
        Err(_) => vec![]
    }
}

pub async fn get_default_branch(crab: &Octocrab, owner: &str, project_name: &str) -> Option<Branch> {
    let default_branch = crab.repos(owner, project_name).get().await.unwrap().default_branch?;
    let branches = crab.repos(owner, project_name).list_branches().per_page(100).send().await.unwrap().items;
    for branch in branches {
        if branch.name == default_branch {
            return Some(branch);
        }
    }
    None
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncResult {
    pub message: String,
    pub merge_type: String,
    pub base_branch: String
}

pub async fn sync_default_branch(crab: &Octocrab, owner: &str, project_name: &str) -> SyncResult {
    let route = format!("/repos/{}/{}/merge-upstream", owner, project_name);
    crab.post(
        route,
        Some(&serde_json::json!(
            { "owner": owner, "repo": project_name, "branch": get_default_branch(crab, owner, project_name).await.unwrap().name }
        ))
    ).await.expect("Syncing did not work correctly")
}

pub async fn create_branch(crab: &Octocrab, owner: &str, project_name: &str, workspace_id: &str) {
    let branch = get_default_branch(crab, owner, project_name).await.unwrap();
    crab.repos(owner, project_name).create_ref(&Reference::Branch(workspace_id.to_string()), branch.commit.sha).await.unwrap();
}

pub async fn create_empty_commit(crab: &Octocrab, owner: &str, project_name: &str, workspace_id: &str) -> Option<(String, String)> {
    match crab.repos(owner, project_name).get_ref(&Reference::Branch(workspace_id.to_string())).await.unwrap().object {
        Object::Commit { sha, .. } => {
            let usable_sha = sha.clone();
            let commit = crab.repos(owner, project_name).list_commits().sha(sha).per_page(1).send().await.unwrap().items.last()?.clone();
            Some((
                usable_sha.clone(),
                crab.repos(owner, project_name)
                    .create_git_commit_object(format!("Initialize {}", workspace_id), commit.commit.tree.sha)
                    .parents(vec![usable_sha])
                    .send()
                    .await
                    .unwrap()
                    .sha
            ))
        }
        _ => None
    }
}

pub async fn push_commit(crab: &Octocrab, owner: &str, project_name: &str, workspace_id: &str, branch_sha: &str, commit: &str) -> Option<CommitObject> {
    let route = format!("/repos/{}/{}/git/refs/heads/{}", owner, project_name, workspace_id);
    crab.post(route, Some(&serde_json::json!({ "ref": branch_sha, "sha": commit, "force": true }))).await.ok()
}

pub async fn is_private(crab: &Octocrab, source_owner: &str, source_name: &str) -> bool {
    crab.repos(source_owner, source_name).get().await.unwrap().private.unwrap()
}

pub async fn create_draft_pull_request(crab: &Octocrab, source_owner: &str, source_name: &str, workspace_title: &str, workspace_full_id: &str, workspace_description: &str) {
    let draft = !is_private(crab, source_owner, source_name).await;
    crab.pulls(source_owner, source_name)
        .create(
            workspace_title,
            workspace_full_id,
            get_default_branch(crab, source_owner, source_name).await.unwrap().name
        )
        .body(workspace_description)
        .draft(draft)
        .send().await.unwrap();
}

#[derive(Debug, Clone, Deserialize)]
pub struct TreeObject {
    pub sha: String,
    pub url: String,
    pub tree: Vec<TreePart>
}

#[derive(Debug, Clone, Deserialize)]
pub struct TreePart {
    pub sha: String,
    pub url: String,
    pub path: String
}

pub async fn get_repository_content(crab: &Octocrab, owner: &str, project_name: &str, branch: &str) -> TreeObject {
    let route = format!("/repos/{}/{}/git/trees/{}", owner, project_name, branch);
    crab.get(route, Some(&serde_json::json!({ "recursive": true }))).await.unwrap()
}

#[derive(Debug, Clone, Deserialize)]
pub struct BlobObject {
    pub content: String
}

pub async fn get_decoded_blob(crab: &Octocrab, owner: &str, project_name: &str, file_sha: &str) -> Result<Vec<u8>, DecodeError> {
    let route = format!("/repos/{}/{}/git/blobs/{}", owner, project_name, file_sha);
    let blob: BlobObject = crab.get(route, Some(&serde_json::json!({}))).await.unwrap();
    let content = blob.content.as_bytes().to_vec().into_iter().filter(|b| !b" \n\t\r\x0b\x0c".contains(b)).collect::<Vec<u8>>();
    base64::prelude::BASE64_STANDARD.decode(content)
}

#[derive(Debug, Clone, Deserialize)]
pub struct BlobCreationResult {
    pub url: String,
    pub sha: String
}

pub async fn create_blob(crab: &Octocrab, owner: &str, project_name: &str, content: Vec<u8>) -> BlobCreationResult {
    let route = format!("/repos/{}/{}/git/blobs", owner, project_name);
    crab.post(route, Some(&serde_json::json!({
        "content": base64::prelude::BASE64_STANDARD.encode(content),
        "encoding": "base64"
    }))).await.unwrap()
}

#[derive(Debug, Clone, Serialize)]
pub struct TreeCreationPart {
    pub path: String,
    pub mode: String,
    #[serde(rename = "type")]
    pub type_: String,
    pub sha: Option<String>
}

pub async fn create_tree(crab: &Octocrab, owner: &str, project_name: &str, workspace_id: &str, blobs: Vec<TreeCreationPart>) -> Option<(String, TreeObject)> {
    match crab.repos(owner, project_name).get_ref(&Reference::Branch(workspace_id.to_string())).await.unwrap().object {
        Object::Commit { sha, .. } => {
            let page = crab.repos(owner, project_name).list_commits().sha(sha).per_page(1).send().await.unwrap();
            let commit = page.items.last()?;
            let new_sha = &commit.sha;
            let tree_base = &commit.commit.tree.sha;
            let route = format!("/repos/{}/{}/git/trees", owner, project_name);
            Some((new_sha.to_string(), crab.post(route, Some(&serde_json::json!({ "base_tree": tree_base.to_string(), "tree": blobs }))).await.unwrap()))
        }
        _ => None
    }
}

pub async fn create_commit(crab: &Octocrab, owner: &str, project_name: &str, modification_name: &str, parent_sha: &str, tree_sha: &str) -> String {
    crab.repos(owner, project_name)
        .create_git_commit_object(modification_name, tree_sha)
        .parents(vec![parent_sha.to_string()])
        .send()
        .await
        .unwrap()
        .sha
}
