use crate::wrapper;
use octocrab;

struct WorkspaceInfo {
    source_owner: String,
    fork_owner: String,
    project_name: String,
    workspace_title: String,
    workspace_id: String,
    workspace_description: String
}

pub async fn create_workspace(info: WorkspaceInfo) {
    if !wrapper::already_forked(&info.source_owner, &info.fork_owner, &info.project_name).await {
        wrapper::fork_repository(&info.source_owner, &info.project_name).await
    }
    else {
        wrapper::sync_default_branch(&info.fork_owner, &info.project_name).await;
    }
    wrapper::create_branch(&info.fork_owner, &info.project_name, &info.workspace_id).await;
    wrapper::create_draft_pull_request(
        &info.source_owner,
        &info.fork_owner,
        &info.project_name,
        &info.workspace_title,
        &info.workspace_id,
        &info.workspace_description
    ).await;
}
