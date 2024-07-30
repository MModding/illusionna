use crate::wrapper;
use octocrab;

struct WorkspaceInfo {
    source_owner: String,
    target_owner: String,
    project_name: String,
    workspace_title: String,
    workspace_id: String,
    workspace_description: String
}

pub async fn create_workspace(info: WorkspaceInfo) {
    if !wrapper::already_forked(&info.source_owner, &info.target_owner, &info.project_name).await {
        wrapper::fork_repository(&info.source_owner, &info.project_name).await
    }
    wrapper::create_branch(&info.target_owner, &info.project_name, &info.workspace_id).await
    // TODO: Open Draft PR
}
