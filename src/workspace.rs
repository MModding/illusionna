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

pub async fn create_project(crab: Octocrab, author: String, project: String) -> ProjectInfo {
    let repository = wrapper::fork_repository(crab, Box::leak(author.into_boxed_str()), Box::leak(project.into_boxed_str())).await;
    let owner = repository.owner.unwrap();
    let parent = repository.parent.unwrap();
    let parent_owner = parent.owner.unwrap();
    ProjectInfo {
        source_owner: parent_owner.login,
        source_owner_icon: wrapper::get_image(parent_owner.avatar_url).await.unwrap(),
        source_name: parent.name,
        source_description: parent.description.unwrap_or("Blank Description".to_string()),
        fork_owner: owner.login,
        fork_name: repository.name,
        fork_description: repository.description.unwrap_or("Blank Description".to_string())
    }
}

#[derive(Debug, Clone)]
pub struct WorkspaceInfo {
    pub project: ProjectInfo,
    pub workspace_name: String,
    pub workspace_id: String,
    pub workspace_description: String
}

pub async fn get_workspaces(crab: Octocrab, project_info: ProjectInfo, all: bool) -> Vec<WorkspaceInfo> {
    wrapper::get_pull_requests(&crab, &project_info.source_owner, &project_info.source_name, all).await.into_iter()
        .map(move |x| WorkspaceInfo {
            project: project_info.clone(),
            workspace_name: x.title.unwrap_or("Blank Title".to_string()),
            workspace_id: x.head.label.unwrap(),
            workspace_description: x.body.unwrap_or("Blank Description".to_string())
        })
        .collect::<Vec<WorkspaceInfo>>()
}

pub async fn create_workspace(crab: Octocrab, info: WorkspaceInfo) {
    wrapper::sync_default_branch(&crab, &info.project.fork_owner, &info.project.fork_name).await;
    wrapper::create_branch(&crab, &info.project.fork_owner, &info.project.fork_name, &info.workspace_id).await;
    let (branch, commit) = wrapper::create_empty_commit(&crab, &info.project.fork_owner, &info.project.fork_name, &info.workspace_id).await.unwrap();
    wrapper::push_commit(&crab, &info.project.fork_owner, &info.project.fork_name, &info.workspace_id, &branch, &commit).await;
    wrapper::create_draft_pull_request(
        &crab,
        &info.project.source_owner,
        &info.project.fork_owner,
        &info.project.source_name,
        &info.workspace_name,
        &info.workspace_id,
        &info.workspace_description
    ).await;
}
