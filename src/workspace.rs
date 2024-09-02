use crate::wrapper;
use crate::wrapper::TreeCreationPart;
use iced::widget::image;
use octocrab;
use octocrab::Octocrab;
use std::collections::{BTreeMap, HashMap};
use std::path::Path;
use iced::futures::StreamExt;
use normalize_path::NormalizePath;

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
    pub workspace_full_id: String,
    pub workspace_id: String,
    pub workspace_description: String
}

pub async fn get_workspaces(crab: Octocrab, project_info: ProjectInfo, all: bool) -> Vec<WorkspaceInfo> {
    wrapper::get_pull_requests(&crab, &project_info.source_owner, &project_info.source_name, all).await.into_iter()
        .map(move |x| WorkspaceInfo {
            project: project_info.clone(),
            workspace_name: x.title.unwrap_or("Blank Title".to_string()),
            workspace_full_id: x.head.label.clone().unwrap(),
            workspace_id: x.head.label.unwrap().split(":").last().unwrap().to_string(),
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
        &info.project.source_name,
        &info.workspace_name,
        &info.workspace_full_id,
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

impl PathInfo {
    pub fn contains(&self, string: &str) -> bool {
        let lowercase_path = self.path.to_lowercase();
        let lowercase_string = string.to_lowercase();
        match &self.content {
            PathContent::Directory(directory) => {
                if lowercase_path.contains(&lowercase_string) {
                    return true;
                }
                for info in directory.contents.values() {
                    if info.contains(string) {
                        return true;
                    }
                }
                false
            }
            PathContent::File(_) => lowercase_path.contains(&lowercase_string)
        }
    }
}

/// Fills up a provided HashMap recursively to a tree-shape.
/// Takes in count that the HashMap can be non-empty.
/// By specifying the path, the url, and the path turned to a vector; this function will check
/// if the current entry it wants to put in the map is the last string entry of the vector;
/// if so, it will set its path context information such as path or url.
/// If it is not the last string entry of the vector; it will instead recursively look up
/// for its inner content until the primary condition is met to set its tree information.
/// When setting the path information, you can provide content through a treemap representing
/// provided directory contents that will be added to the current directory information at this
/// location and if this location is not a directory then it will become one.
fn fill_content(ref_sha: String, ref_path: String, ref_url: String, ref_name: String, provided_content: Option<BTreeMap<String, PathInfo>>, map: &mut BTreeMap<String, PathInfo>, remaining: &mut Vec<String>, i: usize, depth: usize) {
    if i == depth - 1 {
        let key = remaining.remove(0);
        let info = PathInfo {
            sha: ref_sha,
            path: ref_path,
            url: ref_url,
            content: if map.contains_key(&key) {
                let content = map.remove(&key).unwrap().content;
                if provided_content.is_some() {
                    if let PathContent::Directory(mut directory) = content {
                        directory.contents.extend(provided_content.unwrap());
                        PathContent::Directory(DirectoryInfo { name: ref_name, contents: directory.contents })
                    } else {
                        PathContent::Directory(DirectoryInfo { name: ref_name, contents: provided_content.unwrap() })
                    }
                }
                else {
                    content
                }
            } else {
                if provided_content.is_some() {
                    PathContent::Directory(DirectoryInfo { name: ref_name, contents: provided_content.unwrap() })
                }
                else {
                    PathContent::File(FileInfo { name: ref_name })
                }
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
        fill_content(ref_sha, ref_path, ref_url, ref_name, provided_content, &mut inner, remaining, i + 1, depth);
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

fn propagate_path(content: PathContent, built_path: Vec<String>) -> (PathContent, HashMap<String, (String, String)>) {
    match content {
        PathContent::Directory(directory) => {
            let mut map = BTreeMap::new();
            let mut refactors = HashMap::new();
            for (key, value) in directory.contents.into_iter() {
                let previous_path = value.path;
                let previous_sha = value.sha;
                let child_path = [built_path.clone(), vec![key.clone()]].concat();
                let is_file = matches!(&value.content, PathContent::File(_));
                let (propagated, inner_refactors) = propagate_path(value.content, child_path.clone());
                let refactored = PathInfo { sha: "".to_string(), path: child_path.join("/"), url: "".to_string(), content: propagated };
                let new_path = refactored.path.clone();
                map.insert(key, refactored);
                if is_file {
                    refactors.insert(previous_path, (new_path, previous_sha));
                }
                refactors.extend(inner_refactors);
            }
            (PathContent::Directory(DirectoryInfo { contents: map, ..directory }), refactors)
        }
        PathContent::File(x) => (PathContent::File(x), HashMap::new())
    }
}

fn erase_content(path: String, map: &mut BTreeMap<String, PathInfo>, remaining: &mut Vec<String>, i: usize, depth: usize, cleanup: bool) -> Option<PathInfo> {
    if i == depth - 1 {
        map.remove(&remaining.remove(0))
    }
    else {
        let key = remaining.remove(0);
        let value = map.remove(&key)?;
        let previous = value.clone();
        let PathContent::Directory(mut directory) = value.content else { panic!("Should be a directory") };
        let removed = erase_content(path, &mut directory.contents, remaining, i + 1, depth, cleanup);
        if !directory.contents.is_empty() || !cleanup {
            let directory_content = PathContent::Directory(DirectoryInfo { name: key.clone(), contents: directory.contents });
            map.insert(key, PathInfo { sha: previous.sha, path: previous.path, url: previous.url, content: directory_content });
        }
        removed
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

pub async fn get_workspace_content(crab: Octocrab, info: WorkspaceInfo) -> (BTreeMap<String, PathInfo>, Modification) {
    let object = wrapper::get_repository_content(&crab, &info.project.fork_owner, &info.project.fork_name, &info.workspace_id).await;
    let mut structure: BTreeMap<String, PathInfo> = BTreeMap::new();
    let mut modification = Modification::new();
    for part in object.tree {
        let mut vec = (&part.path).split("/").map(|s| s.to_string()).collect::<Vec<String>>();
        let name = (&vec.last().unwrap()).to_string();
        let len = &vec.len();
        fill_content(part.sha, part.path.clone(), part.url, name, None, &mut structure, &mut vec, 0usize, len.clone());
        modification.upstream.push(part.path);
    }
    // debug_content(&structure, 0);
    (structure, modification)
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
        let name = if is_inside_directory {
            if !import_location_path.is_empty() { format!("{}/{}", import_location_path, file.file_name()) } else { file.file_name() }
        } else {
            import_location_path.to_string()
        };
        map.insert(name, file.read().await);
    }
    map
}

pub fn append_workspace_content(content: &mut BTreeMap<String, PathInfo>, paths: Vec<String>) {
    for path in paths {
        let mut vec = path.split("/").map(|s| s.to_string()).collect::<Vec<String>>();
        let name = (&vec.last().unwrap()).to_string();
        let len = &vec.len();
        fill_content("".to_string(), path, "".to_string(), name, None, content, &mut vec, 0usize, len.clone())
    }
}

/// Extracts path information from an origin location (and so removes it at this location) and if
/// it is a directory it will also retain its content. Then it will append that path information to
/// refactored location after propagating the new path through all inner items of the information.
/// At the end, this function will return a map of all locations that were moved, linked to their
/// new refactored locations and the sha that was originally inside the information at the origin.
pub fn refactor_workspace_content(content: &mut BTreeMap<String, PathInfo>, origin_path: String, refactor_input: String, origin_sha: String) -> HashMap<String, (String, String)> {
    let mut origin = origin_path.split("/").map(|s| s.to_string()).collect::<Vec<String>>();
    let normalized = Path::new(&origin.clone()[..origin.len().clone() - 1].join("/"))
        .join(refactor_input)
        .normalize()
        .to_str()
        .unwrap()
        .to_string()
        .replace("\\", "/");
    let mut refactor = normalized.split("/").map(|s| s.to_string()).collect::<Vec<String>>();
    let origin_len = &origin.len();
    let to_propagate = erase_content(origin_path.clone(), content, &mut origin, 0usize, origin_len.clone(), false).unwrap();
    let is_file = matches!(&to_propagate.content, PathContent::File(_));
    let (propagation, mut trace) = propagate_path(to_propagate.content, refactor.clone());
    let erased = match propagation {
        PathContent::Directory(directory) => Some(directory.contents),
        PathContent::File(_) => None
    };
    let refactor_len = &refactor.len();
    fill_content("".to_string(), normalized.clone(), "".to_string(), refactor.last().unwrap().to_string(), erased, content, &mut refactor.clone(), 0usize, refactor_len.clone());
    if is_file {
        trace.insert(origin_path, (normalized, origin_sha));
    }
    // debug_content(content, 0usize);
    trace
}

pub fn remove_workspace_content(content: &mut BTreeMap<String, PathInfo>, path: String) {
    let mut vec = path.split("/").map(|s| s.to_string()).collect::<Vec<String>>();
    let len = &vec.len();
    erase_content(path, content, &mut vec, 0usize, len.clone(), true);
}

pub async fn get_file_content(crab: Octocrab, info: WorkspaceInfo, file_sha: String) -> Vec<u8> {
    wrapper::get_decoded_blob(&crab, &info.project.fork_owner, &info.project.fork_name, &file_sha).await.unwrap()
}

#[derive(Debug, Clone)]
pub enum FileContent {
    Bytes(Vec<u8>),
    Sha(String)
}

#[derive(Debug, Clone)]
pub enum Change {
    AssignContent(FileContent),
    EraseContent
}

#[derive(Debug, Clone)]
pub struct Modification {
    upstream: Vec<String>,
    changes: HashMap<String, Change>
}

impl Modification {
    pub fn new() -> Modification { Modification { upstream: Vec::new(), changes: HashMap::new() } }

    pub fn view(&self, path: &String) -> Option<&FileContent> {
        if let Change::AssignContent(content) = self.changes.get(path)? {
            Some(content)
        } else {
            None
        }
    }

    pub fn set(&mut self, path: String, content: FileContent) {
        self.changes.insert(path, Change::AssignContent(content));
    }

    pub fn refactor(&mut self, origin: String, refactor: String, origin_sha: String) {
        if origin != refactor {
            let content = if self.changes.contains_key(&origin) {
                let change = self.changes.remove(&origin).unwrap();
                if let Change::AssignContent(local_content) = change {
                    Some(local_content)
                } else {
                    None
                }
            } else {
                Some(FileContent::Sha(origin_sha))
            };
            if content.is_some() {
                self.erase(origin);
                self.set(refactor, content.unwrap());
            }
        }
    }

    pub fn erase(&mut self, path: String) {
        if self.upstream.contains(&path) {
            self.changes.insert(path, Change::EraseContent);
        }
    }

    pub fn reset(&mut self) {
        self.changes.clear();
    }

    pub fn present(&self) -> bool {
        !self.changes.is_empty()
    }
}

pub async fn send_contents(crab: Octocrab, info: WorkspaceInfo, modification: Modification, modification_name: String) {
    let mut tree_parts = vec![];
    for (path, change) in modification.changes {
        match change {
            Change::AssignContent(content) => {
                match content {
                    FileContent::Sha(sha) => {
                        tree_parts.push(TreeCreationPart { path, mode: "100644".to_string(), type_: "blob".to_string(), sha: Some(sha) });
                    }
                    FileContent::Bytes(bytes) => {
                        let blob = wrapper::create_blob(&crab, &info.project.fork_owner, &info.project.fork_name, bytes).await;
                        tree_parts.push(TreeCreationPart { path, mode: "100644".to_string(), type_: "blob".to_string(), sha: Some(blob.sha) });
                    }
                };
            }
            Change::EraseContent => {
                tree_parts.push(TreeCreationPart { path, mode: "100644".to_string(), type_: "blob".to_string(), sha: None });
            }
        }
    }
    let (parent_sha, tree) = wrapper::create_tree(&crab, &info.project.fork_owner, &info.project.fork_name, &info.workspace_id, tree_parts).await.unwrap();
    let commit_sha = wrapper::create_commit(&crab, &info.project.fork_owner, &info.project.fork_name, &modification_name, &parent_sha, &tree.sha).await;
    wrapper::push_commit(&crab, &info.project.fork_owner, &info.project.fork_name, &info.workspace_id, &parent_sha, &commit_sha).await;
}
