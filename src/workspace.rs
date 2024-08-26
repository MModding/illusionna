use crate::wrapper;
use iced::widget::image;
use octocrab;
use octocrab::Octocrab;
use std::collections::{BTreeMap, HashMap};

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

#[derive(Debug, Clone)]
pub struct PathInfo {
    pub sha: String,
    pub path: String,
    pub url: String,
    pub content: PathContent
}

#[derive(Debug, Clone)]
pub enum PathContent {
    File(FileInfo),
    Directory(DirectoryInfo)
}

#[derive(Debug, Clone)]
pub struct FileInfo {
    pub name: String
}

#[derive(Debug, Clone)]
pub struct DirectoryInfo {
    pub name: String,
    pub contents: BTreeMap<String, PathInfo>
}

/// Fills up a provided HashMap recursively to a tree-shape.
/// Takes in count that the HashMap can be non-empty.
/// By specifying the path, the url, and the path turned to a vector; this function will check
/// if the current entry it wants to put in the map is the last string entry of the vector;
/// if so, it will set its path context information such as path or url.
/// If it is not the last string entry of the vector; it will instead recursively look up
/// for its inner content until the primary condition is met to set its tree information.
fn fill_content(ref_sha: String, ref_path: String, ref_url: String, ref_name: String, map: &mut BTreeMap<String, PathInfo>, remaining: &mut Vec<String>, i: usize, depth: usize) {
    if i == depth - 1 {
        let key = remaining.remove(0);
        let info = PathInfo {
            sha: ref_sha,
            path: ref_path,
            url: ref_url,
            content: if map.contains_key(&key) {
                map.remove(&key).unwrap().content
            } else {
                PathContent::File(FileInfo { name: ref_name })
            }
        };
        map.insert(key, info);
    }
    else {
        let key = remaining.remove(0);
        let mut previous: Option<PathInfo> = None;
        let mut inner: BTreeMap<String, PathInfo> = if map.contains_key(&key) {
            let value = map.remove(&key).unwrap();
            previous = Some(value.clone());
            match value.content {
                PathContent::File(_) => BTreeMap::new(),
                PathContent::Directory(info) => info.contents
            }
        } else {
            BTreeMap::new()
        };
        fill_content(ref_sha, ref_path, ref_url, ref_name, &mut inner, remaining, i + 1, depth);
        let directory_content = PathContent::Directory(DirectoryInfo { name: key.clone(), contents: inner });
        let info = if previous.is_some() {
            let previous_info = previous.unwrap();
            PathInfo {
                sha: previous_info.sha,
                path: previous_info.path,
                url: previous_info.url,
                content: directory_content
            }
        } else {
            PathInfo {
                sha: "".to_string(),
                path: "".to_string(),
                url: "".to_string(),
                content: directory_content
            }
        };
        map.insert(key, info);
    }
}

pub (crate) fn debug_content(structure: &BTreeMap<String, PathInfo>, indentation: usize) {
    for (key, value) in structure {
        println!("{}{}: (", " ".repeat(indentation), key);
        println!("{}  path: {}", " ".repeat(indentation), value.path);
        println!("{}  url: {}", " ".repeat(indentation), value.url);
        match &value.content {
            PathContent::File(info) => {
                println!("{}  name: {}", " ".repeat(indentation), info.name);
                println!("{})", " ".repeat(indentation));
            }
            PathContent::Directory(info) => {
                println!("{}  name: {}", " ".repeat(indentation), info.name);
                println!("{}  contents: (", " ".repeat(indentation));
                debug_content(&info.contents, indentation + 4);
                println!("{}  )", " ".repeat(indentation));
                println!("{})", " ".repeat(indentation));
            }
        }
    }
}

pub async fn get_workspace_content(crab: Octocrab, info: WorkspaceInfo) -> BTreeMap<String, PathInfo> {
    let object = wrapper::get_repository_content(&crab, &info.project.fork_owner, &info.project.fork_name, &info.workspace_id.split(":").last().unwrap().to_string()).await;
    let mut structure: BTreeMap<String, PathInfo> = BTreeMap::new();
    for part in object.tree {
        let mut vec = part.path.split("/").map(|s| s.to_string()).collect::<Vec<String>>();
        let name = (&vec.last().unwrap()).to_string();
        let len = &vec.len();
        fill_content(part.sha, part.path, part.url, name, &mut structure, &mut vec, 0usize, len.clone())
    }
    // debug_content(&structure, 0);
    structure
}

pub async fn import_files(is_inside_directory: bool, import_location_path: String) -> HashMap<String, Vec<u8>> {
    let dialog = rfd::AsyncFileDialog::new();
    let files: Vec<rfd::FileHandle> = if is_inside_directory {
        dialog.pick_files().await.unwrap_or(vec![])
    } else {
        let file = dialog.pick_file().await;
        if file.is_some() { vec![file.unwrap()] } else { vec![] }
    };
    let mut map = HashMap::new();
    for file in files {
        map.insert(if is_inside_directory { format!("{}/{}", import_location_path, file.file_name()) } else { import_location_path.to_string() }, file.read().await);
    }
    map
}

pub fn append_workspace_content(content: &mut BTreeMap<String, PathInfo>, paths: Vec<String>) {
    for path in paths {
        let mut vec = path.split("/").map(|s| s.to_string()).collect::<Vec<String>>();
        let name = (&vec.last().unwrap()).to_string();
        let len = &vec.len();
        fill_content("".to_string(), path, "".to_string(), name, content, &mut vec, 0usize, len.clone())
    }
}

pub async fn view_workspace_file(crab: Octocrab, info: WorkspaceInfo, file_sha: String) -> Vec<u8> {
    wrapper::get_decoded_blob(&crab, &info.project.fork_owner, &info.project.fork_name, &file_sha).await.unwrap()
}
