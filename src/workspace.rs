use iced::widget::image;
use crate::wrapper;
use octocrab;
use octocrab::Octocrab;

#[derive(Debug, Clone)]
pub struct ProjectInfo {
    pub source_owner: String,
    pub source_owner_icon: image::Handle,
    pub source_name: String,
    pub source_description: String,
    pub fork_owner: String,
    pub fork_name: String,
    pub fork_description: String
}

pub async fn get_projects(crab: Octocrab) -> Vec<ProjectInfo> {
    let repositories = wrapper::get_forked_repositories(&crab).await;
    let mut projects: Vec<ProjectInfo> = Vec::new();
    for repository in repositories {
        let source_repository = &crab.repos(&repository.owner.clone().unwrap().login, &repository.name).get().await.unwrap().parent.unwrap();
        let source_owner = source_repository.owner.clone().unwrap();
        projects.push(
            ProjectInfo {
                source_owner: source_owner.login,
                source_owner_icon: wrapper::get_image(source_owner.avatar_url).await.unwrap(),
                source_name: source_repository.name.clone(),
                source_description: source_repository.description.clone().unwrap_or("Blank Description".to_string()),
                fork_owner: repository.owner.unwrap().login,
                fork_name: repository.name,
                fork_description: repository.description.unwrap_or("Blank Description".to_string())
            }
        )
    }
    projects
}

#[derive(Debug, Clone)]
pub struct WorkspaceInfo {
    source_owner: String,
    fork_owner: String,
    project_name: String,
    workspace_title: String,
    workspace_id: String,
    workspace_description: String
}

pub async fn create_workspace(crab: &Octocrab, info: WorkspaceInfo) {
    if !wrapper::already_forked(crab, &info.source_owner, &info.fork_owner, &info.project_name).await {
        wrapper::fork_repository(crab, &info.source_owner, &info.project_name).await
    }
    else {
        wrapper::sync_default_branch(crab, &info.fork_owner, &info.project_name).await;
    }
    wrapper::create_branch(crab, &info.fork_owner, &info.project_name, &info.workspace_id).await;
    wrapper::create_draft_pull_request(
        crab,
        &info.source_owner,
        &info.fork_owner,
        &info.project_name,
        &info.workspace_title,
        &info.workspace_id,
        &info.workspace_description
    ).await;
}
