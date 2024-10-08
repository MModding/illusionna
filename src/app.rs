use crate::workspace::{FileContent, Modification, PathContent, PathInfo, ProjectInfo, WorkspaceInfo};
use crate::wrapper::AccountInfo;
use crate::{workspace, wrapper};
use iced::alignment::{Horizontal, Vertical};
use iced::theme::Palette;
use iced::widget::image::{FilterMethod, Viewer};
use iced::widget::{button, image, markdown, scrollable, svg, text, Button, Checkbox, Column, Container, Image, Row, Scrollable, Svg, Text, TextInput};
use iced::window::icon;
use iced::{widget, window, Alignment, Background, Border, Color, Degrees, Element, Length, Padding, Radians, Renderer, Rotation, Shadow, Subscription, Task, Theme};
use octocrab::Octocrab;
use reqwest::Url;
use std::collections::{BTreeMap, HashMap};

// Illusionna Icons
const ICON: &[u8] = include_bytes!("../resources/icon.png").as_slice();

// Bootstrap Icons
const REPOSITORY: &[u8] = include_bytes!("../resources/repository.svg").as_slice();
const APPEND: &[u8] = include_bytes!("../resources/append.svg").as_slice();
const REPLACE: &[u8] = include_bytes!("../resources/replace.svg").as_slice();
const RENAME: &[u8] = include_bytes!("../resources/rename.svg").as_slice();
const REMOVE: &[u8] = include_bytes!("../resources/remove.svg").as_slice();
const FOLDER: &[u8] = include_bytes!("../resources/folder.svg").as_slice();
const COLLAPSE: &[u8] = include_bytes!("../resources/collapse.svg").as_slice();
const EXPAND: &[u8] = include_bytes!("../resources/expand.svg").as_slice();

#[derive(Debug, Clone)]
enum CrabState {
    Absent,
    Present(Octocrab)
}

#[derive(Debug, Clone)]
pub enum Display {
    GithubConnexion,
    ProjectSelection,
    WorkspaceSelection,
    WorkspaceCreation,
    WorkspaceContent
}

#[derive(Debug, Clone)]
pub enum ReferenceValidation {
    Valid,
    Invalid(String),
    Unspecified
}

#[derive(Debug, Clone)]
pub struct IllusionnaApp {
    rotator: u16,
    crab: CrabState,
    display: Display,
    projects: Option<Vec<ProjectInfo>>,
    selected_project: Option<ProjectInfo>,
    project_creation_text: String,
    project_creation_validation: ReferenceValidation,
    account: Option<AccountInfo>,
    workspaces: Option<Vec<WorkspaceInfo>>,
    show_closed: bool,
    workspace_creation_name_text: String,
    workspace_creation_id_text: String,
    workspace_creation_description_text: String,
    selected_workspace: Option<WorkspaceInfo>,
    workspace_content: Option<BTreeMap<String, PathInfo>>,
    workspace_content_filter: String,
    collapsed_directories: Vec<String>,
    viewed_file_path: Option<String>,
    viewed_file_name: Option<String>,
    viewed_file_content: Option<Vec<u8>>,
    refactors: HashMap<String, String>,
    modification: Modification,
    modification_name: String,
}

#[derive(Debug, Clone)]
pub enum Interaction {
    Tick,
    StartDeviceFlow,
    CompleteDeviceFlow(Octocrab),
    ReceiveProjectInfos(Vec<ProjectInfo>),
    ReceiveAccountInfo(AccountInfo),
    SelectProjectInfo(String),
    ProcessProjectReference(String),
    ValidateProjectReference(ReferenceValidation),
    CreateProject,
    OpenLink(Url),
    OpenSelectedProject,
    AppendCreatedProject(ProjectInfo),
    ReceiveWorkspaceInfos(Vec<WorkspaceInfo>),
    DisplayProjectsList,
    ToggleClosedWorkspaces(bool),
    CreateNewWorkspace,
    WorkspaceNameInput(String),
    WorkspaceIdInput(String),
    WorkspaceDescriptionInput(String),
    ProcessNewWorkspace,
    DisplayWorkspacesList,
    AddNewWorkspace(WorkspaceInfo),
    OpenWorkspace(String),
    ReceiveWorkspaceContent(BTreeMap<String, PathInfo>, Modification, bool),
    FilterWorkspaceContent(String),
    CollapseDirectory(String),
    ExpandDirectory(String),
    ViewFile(String, String),
    ProcessViewingContent(Vec<u8>),
    SelectFiles(bool, String),
    SetFiles(HashMap<String, Vec<u8>>),
    RefactorFiles(String),
    RefactorInput(String, String),
    PushRefactor(String, String),
    EraseFiles(String),
    ModificationNameInput(String),
    SendChanges,
    ResetChanges
}

pub fn sidebar_button(theme: &Theme, status: button::Status) -> button::Style {
    let color;
    if status == button::Status::Hovered || status == button::Status::Pressed {
        color = Color::from_rgb8(72, 68, 255)
    }
    else {
        if theme.extended_palette().is_dark {
            color = Color::from_rgb8(46, 45, 62);
        } else {
            color = Color::from_rgb8(184, 182, 216);
        }
    }
    button::Style {
        background: Some(Background::Color(color)),
        text_color: theme.palette().text,
        border: Border::default(),
        shadow: Shadow::default()
    }
}

pub fn large_button(theme: &Theme, status: button::Status) -> button::Style {
    button::Style { border: Border::default().rounded(10), ..sidebar_button(theme, status) }
}

pub fn small_button(theme: &Theme, status: button::Status) -> button::Style {
    button::Style { border: Border::default().rounded(3), ..large_button(theme, status) }
}

pub fn default_svg(theme: &Theme, _: svg::Status) -> svg::Style {
    svg::Style {
        color: if theme.extended_palette().is_dark { Some(Color::WHITE) } else { Some(Color::BLACK) }
    }
}

pub fn advanced_svg(color: Color, theme: &Theme, status: svg::Status) -> svg::Style {
    if status == svg::Status::Hovered {
        svg::Style { color: Some(color) }
    }
    else {
        default_svg(theme, status)
    }
}

impl IllusionnaApp {

    pub fn new() -> (Self, Task<Interaction>) {
        let icon_png = icon::from_file_data(ICON, None).unwrap();
        let icon_task = window::get_latest().and_then(move |id| window::change_icon(id, icon_png.clone()));
        (
            IllusionnaApp {
                rotator: 0u16,
                crab: CrabState::Absent,
                display: Display::GithubConnexion,
                projects: None,
                selected_project: None,
                project_creation_text: "".to_string(),
                project_creation_validation: ReferenceValidation::Unspecified,
                account: None,
                workspaces: None,
                show_closed: false,
                workspace_creation_name_text: "".to_string(),
                workspace_creation_id_text: "".to_string(),
                workspace_creation_description_text: "".to_string(),
                selected_workspace: None,
                workspace_content: None,
                workspace_content_filter: "".to_string(),
                collapsed_directories: vec![],
                viewed_file_path: None,
                viewed_file_name: None,
                viewed_file_content: None,
                refactors: HashMap::new(),
                modification: Modification::new(),
                modification_name: "".to_string()
            },
            icon_task
        )
    }

    pub fn ticker(&self) -> Subscription<Interaction> {
        window::frames().map(|_| Interaction::Tick)
    }

    pub fn get_crab(&self) -> &Octocrab {
        match &self.crab {
            CrabState::Absent => panic!("Crab is Absent"),
            CrabState::Present(crab) => crab
        }
    }

    pub fn title(&self) -> String {
        String::from("Illusionna")
    }

    pub fn update(&mut self, message: Interaction) -> Task<Interaction> {
        match message {
            Interaction::Tick => {
                self.rotator = (self.rotator + 1) % 360;
                Task::none()
            }
            Interaction::StartDeviceFlow => {
                Task::perform(wrapper::embedded_oauth_process(), |result| {
                    return Interaction::CompleteDeviceFlow(result.unwrap());
                })
            }
            Interaction::CompleteDeviceFlow(crab) => {
                self.crab = CrabState::Present(crab);
                self.display = Display::ProjectSelection;
                let usable_crab = self.get_crab().clone();
                Task::perform(workspace::get_projects(usable_crab.clone()), |projects| {
                    return Interaction::ReceiveProjectInfos(projects)
                })
            }
            Interaction::ReceiveProjectInfos(projects) => {
                let count = projects.len();
                self.projects = Some(projects);
                let crab = self.get_crab().clone();
                Task::perform(wrapper::get_account_info(crab, count), |account| {
                    return Interaction::ReceiveAccountInfo(account)
                })
            }
            Interaction::ReceiveAccountInfo(account) => {
                self.account = Some(account);
                Task::none()
            }
            Interaction::SelectProjectInfo(fork_name) => {
                for x in self.projects.clone().unwrap() {
                    if x.fork_name == fork_name {
                        self.selected_project = Some(x);
                        return Task::none();
                    }
                }
                Task::none()
            }
            Interaction::ProcessProjectReference(reference) => {
                self.project_creation_text = reference.clone();
                if reference.is_empty() {
                    self.project_creation_validation = ReferenceValidation::Unspecified;
                    return Task::none();
                }
                if reference.contains("/") {
                    let split = reference.split("/").collect::<Vec<&str>>();
                    if split.len() == 2 {
                        let author = split[0];
                        let project = split[1];
                        for x in self.projects.clone().unwrap() {
                            if x.source_name == project {
                                self.project_creation_validation = ReferenceValidation::Invalid("Project already exists.".to_string());
                                return Task::none();
                            }
                        }
                        let crab = self.get_crab().clone();
                        return Task::perform(wrapper::repository_exists(crab.clone(), author.to_string(), project.to_string()), move |result| {
                            if result {
                                Interaction::ValidateProjectReference(ReferenceValidation::Valid)
                            }
                            else {
                                Interaction::ValidateProjectReference(ReferenceValidation::Invalid("Project does not exist.".to_string()))
                            }
                        });
                    }
                }
                self.project_creation_validation = ReferenceValidation::Invalid("Invalid Project Reference.".to_string());
                Task::none()
            }
            Interaction::ValidateProjectReference(validation) => {
                self.project_creation_validation = validation;
                Task::none()
            }
            Interaction::CreateProject => {
                match self.project_creation_validation {
                    ReferenceValidation::Valid => {
                        let split = self.project_creation_text.split("/").collect::<Vec<&str>>().clone();
                        let author = split[0].to_string();
                        let project = split[1].to_string();
                        let crab = self.get_crab().clone();
                        Task::perform(workspace::create_project(crab.clone(), author.clone(), project.clone()), move |result| {
                            Interaction::AppendCreatedProject(result)
                        })
                    }
                    _ => Task::none()
                }
            }
            Interaction::OpenLink(url) => {
                open::that(url.as_str()).unwrap();
                Task::none()
            }
            Interaction::OpenSelectedProject => {
                self.display = Display::WorkspaceSelection;
                let crab = self.get_crab().clone();
                let project = self.selected_project.clone().unwrap();
                Task::perform(workspace::get_workspaces(crab.clone(), project.clone(), false), |workspaces| {
                    Interaction::ReceiveWorkspaceInfos(workspaces)
                })
            }
            Interaction::AppendCreatedProject(project) => {
                let mut projects = self.projects.clone().unwrap();
                projects.insert(0, project.clone());
                self.projects = Some(projects);
                self.selected_project = Some(project);
                Task::none()
            }
            Interaction::ReceiveWorkspaceInfos(workspaces) => {
                self.workspaces = Some(workspaces);
                Task::none()
            }
            Interaction::DisplayProjectsList => {
                self.display = Display::ProjectSelection;
                self.workspaces = None;
                Task::none()
            }
            Interaction::ToggleClosedWorkspaces(toggle) => {
                self.workspaces = None;
                self.show_closed = toggle;
                let crab = self.get_crab().clone();
                let project = self.selected_project.clone().unwrap();
                Task::perform(workspace::get_workspaces(crab.clone(), project.clone(), toggle), |workspaces| {
                    Interaction::ReceiveWorkspaceInfos(workspaces)
                })
            }
            Interaction::CreateNewWorkspace => {
                self.display = Display::WorkspaceCreation;
                Task::none()
            }
            Interaction::WorkspaceNameInput(input) => {
                self.workspace_creation_name_text = input;
                Task::none()
            }
            Interaction::WorkspaceIdInput(input) => {
                let filtered_input = input.chars()
                    .filter(|char| matches!(*char, 'a'..='z') | matches!(*char, '0'..='9') | char.eq(&'-') | char.eq(&'/'))
                    .collect::<String>();
                self.workspace_creation_id_text = filtered_input;
                Task::none()
            }
            Interaction::WorkspaceDescriptionInput(input) => {
                self.workspace_creation_description_text = input;
                Task::none()
            }
            Interaction::ProcessNewWorkspace => {
                if !self.workspace_creation_name_text.is_empty() && !self.workspace_creation_id_text.is_empty() {
                    let crab = self.get_crab().clone();
                    let selected_project = self.selected_project.clone().unwrap();
                    let fork_owner = selected_project.fork_owner.clone();
                    let workspace = WorkspaceInfo {
                        project: selected_project,
                        workspace_name: self.workspace_creation_name_text.clone(),
                        workspace_full_id: format!("{}:{}", fork_owner, &self.workspace_creation_id_text),
                        workspace_id: self.workspace_creation_id_text.clone(),
                        workspace_description: format!("{}\n\nPowered by [Illusionna](https://mmodding.com/illusionna).", self.workspace_creation_description_text.clone()),
                    };
                    return Task::perform(workspace::create_workspace(crab, workspace.clone()), move |_| {
                        Interaction::AddNewWorkspace((&workspace).clone())
                    });
                }
                Task::none()
            }
            Interaction::DisplayWorkspacesList => {
                self.workspace_creation_name_text = "".to_string();
                self.workspace_creation_id_text = "".to_string();
                self.workspace_creation_description_text = "".to_string();
                self.workspace_content = None;
                self.workspace_content_filter = "".to_string();
                self.collapsed_directories.clear();
                self.viewed_file_name = None;
                self.viewed_file_path = None;
                self.viewed_file_content = None;
                self.modification.reset();
                self.display = Display::WorkspaceSelection;
                Task::none()
            }
            Interaction::AddNewWorkspace(workspace) => {
                let mut workspaces = self.workspaces.clone().unwrap();
                workspaces.insert(0, workspace.clone());
                self.workspaces = Some(workspaces);
                Task::done(Interaction::DisplayWorkspacesList)
            }
            Interaction::OpenWorkspace(workspace_full_id) => {
                let crab = self.get_crab().clone();
                for x in self.workspaces.clone().unwrap() {
                    if x.workspace_full_id == workspace_full_id {
                        self.selected_workspace = Some(x.clone());
                        return Task::perform(workspace::get_workspace_content(crab.clone(), x), |(x, y, z)| Interaction::ReceiveWorkspaceContent(x, y, z));
                    }
                }
                Task::none()
            }
            Interaction::ReceiveWorkspaceContent(content, modification, private) => {
                self.workspace_content = Some(content);
                self.modification = modification;
                self.display = Display::WorkspaceContent;
                Task::none()
            }
            Interaction::FilterWorkspaceContent(input) => {
                self.workspace_content_filter = input;
                Task::none()
            }
            Interaction::CollapseDirectory(path) => {
                let mut vec = self.collapsed_directories.clone();
                vec.push(path);
                self.collapsed_directories = vec;
                Task::none()
            }
            Interaction::ExpandDirectory(path) => {
                let mut vec = self.collapsed_directories.clone();
                vec.retain(|x| x != &path);
                self.collapsed_directories = vec;
                Task::none()
            }
            Interaction::ViewFile(sha, path) => {
                self.viewed_file_path = Some(path.clone());
                self.viewed_file_name = Some(path.split("/").last().unwrap().to_string());
                self.viewed_file_content = None;
                let content = self.modification.view(&path);
                if content.is_some() {
                    match content.unwrap() {
                        FileContent::Bytes(bytes) => {
                            return Task::done(Interaction::ProcessViewingContent(bytes.clone()));
                        }
                        _ => {}
                    }
                }
                let crab = self.get_crab().clone();
                let workspace = self.selected_workspace.clone().unwrap();
                Task::perform(workspace::get_file_content(crab, workspace, sha), Interaction::ProcessViewingContent)
            }
            Interaction::ProcessViewingContent(bytes) => {
                self.viewed_file_content = Some(bytes);
                Task::none()
            }
            Interaction::SelectFiles(is_inside_dir, import_location_path) => {
                Task::perform(workspace::import_files(is_inside_dir, import_location_path), Interaction::SetFiles)
            }
            Interaction::SetFiles(files) => {
                let Some(mut content) = self.workspace_content.clone() else { panic!() };
                workspace::append_workspace_content(&mut content, files.keys().map(|x| x.to_string()).collect());
                self.workspace_content = Some(content);
                for file in files {
                    self.modification.set(file.0, FileContent::Bytes(file.1));
                }
                Task::none()
            }
            Interaction::RefactorFiles(path) => {
                self.refactors.insert(path.clone(), path.split("/").last().unwrap().to_string());
                Task::none()
            }
            Interaction::RefactorInput(path, input) => {
                if !input.starts_with("/") && !input.contains("//") {
                    self.refactors.insert(path, input);
                }
                Task::none()
            }
            Interaction::PushRefactor(path, sha) => {
                let check = self.refactors.get(&path).unwrap();
                if !check.ends_with("/") && !check.ends_with(".") {
                    let Some(mut content) = self.workspace_content.clone() else { panic!() };
                    let refactors = workspace::refactor_workspace_content(&mut content, path.clone(), self.refactors.remove(&path).unwrap(), sha);
                    self.workspace_content = Some(content);
                    for (origin, (refactor, origin_sha)) in refactors {
                        self.modification.refactor(origin, refactor, origin_sha);
                    }
                }
                Task::none()
            }
            Interaction::EraseFiles(path) => {
                let Some(mut content) = self.workspace_content.clone() else { panic!() };
                workspace::remove_workspace_content(&mut content, path.clone());
                self.workspace_content = Some(content);
                self.modification.erase(path);
                Task::none()
            }
            Interaction::ModificationNameInput(input) => {
                self.modification_name = input;
                Task::none()
            }
            Interaction::SendChanges => {
                if !self.modification_name.is_empty() {
                    let crab = self.get_crab().clone();
                    let workspace = self.selected_workspace.clone().unwrap();
                    let modification = self.modification.clone(); // I do not like that at all.
                    self.modification.reset();
                    let modification_name = self.modification_name.clone();
                    Task::perform(workspace::send_contents(crab, workspace, modification, modification_name), |_| Interaction::ResetChanges)
                }
                else {
                    Task::none()
                }
            }
            Interaction::ResetChanges => {
                self.refactors.clear();
                self.modification.reset();
                self.modification_name = "".to_string();
                let crab = self.get_crab();
                let workspace = self.selected_workspace.clone().unwrap();
                Task::perform(workspace::get_workspace_content(crab.clone(), workspace), |(x, y, z)| Interaction::ReceiveWorkspaceContent(x, y, z))
            }
        }
    }

    pub fn view(&self) -> Element<'_, Interaction, Theme, Renderer> {
        match &self.display {
            Display::GithubConnexion => self.github_connection(),
            Display::ProjectSelection => self.project_selection(),
            Display::WorkspaceCreation => self.workspace_creation(),
            Display::WorkspaceSelection => self.workspace_selection(),
            Display::WorkspaceContent => self.workspace_content()
        }
    }

    fn github_connection(&self) -> Element<'_, Interaction, Theme, Renderer> {
        let illusionna_title = Image::new(image::Handle::from_bytes(include_bytes!("../resources/title.png").as_slice()))
            .filter_method(FilterMethod::Nearest)
            .width(Length::Fixed(426f32))
            .height(Length::Fixed(240f32));
        let device_auth_text = text("GitHub Authentication");
        let device_auth_button = Button::new("Login to GitHub via Device Flow")
            .style(small_button)
            .on_press(Interaction::StartDeviceFlow);
        let column = Column::new().push(illusionna_title).push(device_auth_text).push(device_auth_button).align_x(Alignment::Center).spacing(10);
        Container::new(column).center_x(Length::Fill).center_y(Length::Fill).into()
    }

    fn project_selection(&self) -> Element<'_, Interaction, Theme, Renderer> {
        let projects: Option<Column<Interaction>> = match &self.projects {
            Some(infos) => {
                Some(Column::new().extend(infos.into_iter().map(|project| {
                    let content: Column<Interaction> = Column::new()
                        .push(text(&project.fork_name).size(16))
                        .push(
                            Row::new()
                                .push(Image::new(&project.source_owner_icon).width(Length::Fixed(24f32)).height(24f32))
                                .push(text(format!("{} - {}", &project.source_owner, &project.source_name)).size(12))
                                .align_y(Vertical::Center)
                                .spacing(10)
                        )
                        .spacing(10);
                    Button::new(content)
                        .width(Length::Fixed(256f32))
                        .height(Length::Fixed(64f32))
                        .style(sidebar_button)
                        .on_press(Interaction::SelectProjectInfo(project.clone().fork_name))
                        .into()
                })))
            }
            None => None
        };
        match projects {
            Some(values) => {
                let scroll = scrollable(values).anchor_left();
                let project_info: Column<Interaction> = match &self.selected_project {
                    Some(selected_project) => {
                        Column::new()
                            .push(
                                Button::new(
                                    Row::new()
                                        .push(Image::new(&selected_project.source_owner_icon).width(Length::Fixed(64f32)).height(Length::Fixed(64f32)))
                                        .push(
                                            Column::new()
                                                .push(Text::new(format!("{} - {}", &selected_project.source_owner, &selected_project.fork_name)).size(28))
                                                .push(Text::new(&selected_project.source_name).size(16))
                                                .spacing(5)
                                        )
                                        .align_y(Vertical::Center)
                                        .spacing(10)
                                ).style(large_button).on_press(Interaction::OpenSelectedProject)
                            )
                            .push("Annotated Description:")
                            .push(Text::new(&selected_project.fork_description))
                            .push("Upstream Description:")
                            .push(Text::new(&selected_project.source_description))
                            .height(Length::Fixed(400f32))
                            .spacing(10)
                            .padding(10)
                    }
                    None => Column::new()
                };
                let creation_and_account = Row::new()
                    .push(Container::new(
                        Column::new()
                            .push(
                                TextInput::new("Project: Author/Project", &self.project_creation_text)
                                    .on_input(Interaction::ProcessProjectReference)
                                    .on_submit(Interaction::CreateProject)
                            )
                            .push(
                                Text::new(match &self.project_creation_validation {
                                    ReferenceValidation::Valid => "Project found!",
                                    ReferenceValidation::Invalid(reference) => reference,
                                    ReferenceValidation::Unspecified => "No specified project."
                                })
                            )
                            .spacing(3)
                    ).padding(6).center_x(Length::Fill).align_bottom(Length::Fill))
                    .push(Container::new(match &self.account {
                        Some(info) => {
                            let content: Row<Interaction> = Row::new()
                                .push(Image::new(&info.avatar).width(Length::Fixed(48f32)).height(Length::Fixed(48f32)))
                                .push(
                                    Column::new()
                                        .push(text(&info.name).size(20))
                                        .push(text(format!("{} Compatible Projects", &info.count)).size(10))
                                        .spacing(6)
                                )
                                .align_y(Vertical::Center)
                                .spacing(12);
                            let button = Button::new(content)
                                .width(Length::Fixed(256f32))
                                .padding(10)
                                .style(large_button)
                                .on_press(Interaction::OpenLink(info.clone().profile));
                            Column::new().push(button)
                        }
                        None => Column::new()
                    }).align_right(Length::Fill).align_bottom(Length::Fill));
                Row::new()
                    .push(scroll)
                    .push(Column::new().push(project_info).push(creation_and_account)).into()
            }
            None => {
                Container::new(
                    Image::new(image::Handle::from_bytes(ICON))
                        .width(Length::Fixed(48f32))
                        .height(Length::Fixed(48f32))
                        .rotation(Rotation::Floating(Radians::from(Degrees(self.rotator as f32))))
                ).center(Length::Fill).into()
            }
        }
    }

    fn workspace_selection(&self) -> Element<'_, Interaction, Theme, Renderer> {
        fn display_workspace(title: String, id: String) -> Button<'static, Interaction> {
            Button::new(
                Container::new(
                    Column::new()
                        .push(Text::new(title).size(16))
                        .push(Text::new(id.clone()).size(12))
                        .spacing(6)
                ).padding(6).width(Length::Fixed(320f32)).height(Length::Fixed(96f32))
            ).style(large_button).on_press(Interaction::OpenWorkspace(id))
        }
        let selected_project = self.selected_project.clone().unwrap();
        let workspaces_widget = match &self.workspaces {
            Some(workspaces) => {
                if !workspaces.is_empty() {
                    let iterations = workspaces.len().div_euclid(2);
                    let mut column = vec![];
                    for i in 0..iterations {
                        let mut row = vec![];
                        let first_info = workspaces.get(i).unwrap();
                        row.push(display_workspace(first_info.workspace_name.to_string(), first_info.workspace_full_id.to_string()));
                        if i + 1 < workspaces.len() {
                            let second_info = workspaces.get(i + 1).unwrap();
                            row.push(display_workspace(second_info.workspace_name.to_string(), second_info.workspace_full_id.to_string()))
                        }
                        column.push(Row::new().extend(row.into_iter().map(|x| x.into())).spacing(6));
                    }
                    if workspaces.len() % 2 == 1 {
                        let last = workspaces.last().unwrap();
                        column.push(
                            Row::new()
                                .push(display_workspace(last.workspace_name.to_string(), last.workspace_full_id.to_string()))
                        );
                    }
                    Column::new()
                        .extend(column.into_iter().map(|x| x.into()))
                        .spacing(6).width(Length::Fill).align_x(Horizontal::Center)
                }
                else {
                    Column::new()
                }
            }
            None => Column::new().push(
                Container::new(
                    Image::new(image::Handle::from_bytes(ICON))
                        .width(Length::Fixed(32f32))
                        .height(Length::Fixed(32f32))
                        .rotation(Rotation::Floating(Radians::from(Degrees(self.rotator as f32))))
                ).center_x(Length::Fill).center_y(288f32)
            )
        };
        Column::new()
            .push(
                Row::new()
                    .push(
                        Row::new()
                            .push(Image::new(selected_project.source_owner_icon).width(Length::Fixed(64f32)).height(Length::Fixed(64f32)))
                            .push(Text::new(selected_project.source_owner).size(28))
                            .spacing(10)
                            .align_y(Vertical::Center)
                    )
                    .push(
                        Text::new(format!("{} ({})", selected_project.fork_name, selected_project.source_name)).size(28).width(Length::Fill).align_x(Horizontal::Right)
                    )
                    .padding(25)
                    .align_y(Vertical::Center)
            )
            .push(scrollable(workspaces_widget).height(Length::Fixed(288f32)))
            .push(
                Container::new(
                    Row::new()
                        .push(
                            Column::new()
                                .push(Button::new(Text::new("Return back to Projects List")).style(small_button).on_press(Interaction::DisplayProjectsList))
                                .width(Length::Fill)
                                .align_x(Horizontal::Left)
                        )
                        .push(
                            Column::new()
                                .push(Checkbox::new("Display Closed Workspaces", self.show_closed).on_toggle(Interaction::ToggleClosedWorkspaces))
                                .width(Length::Fill)
                                .align_x(Horizontal::Center)
                        )
                        .push(
                            Column::new()
                                .push(Button::new(Text::new("Create New Workspace")).style(small_button).on_press(Interaction::CreateNewWorkspace))
                                .width(Length::Fill)
                                .align_x(Horizontal::Right)
                        )
                        .align_y(Vertical::Center)
                ).align_bottom(Length::Fill).padding(10)
            )
            .into()
    }

    fn workspace_creation(&self) -> Element<'_, Interaction, Theme, Renderer> {
        let selected_project = self.selected_project.clone().unwrap();
        Column::new()
            .push(
                Row::new()
                    .push(Image::new(selected_project.source_owner_icon).width(Length::Fixed(64f32)).height(Length::Fixed(64f32)))
                    .push(
                        Column::new()
                            .push(Text::new("Workspace Creation").size(24))
                            .push(Text::new(format!("{}/{}", selected_project.source_owner, selected_project.source_name)))
                    )
                    .spacing(10)
                    .align_y(Vertical::Center)
            )
            .push(
                Column::new()
                    .push(Text::new("Workspace Name"))
                    .push(TextInput::new("Enter a title", &self.workspace_creation_name_text).width(Length::Fixed(250f32)).on_input(Interaction::WorkspaceNameInput))
                    .spacing(6)
            )
            .push(
                Column::new()
                    .push(Text::new("Workspace Id"))
                    .push(TextInput::new("Only [a-z] & [0-9] & \"-\" & \"/\"", &self.workspace_creation_id_text).width(Length::Fixed(250f32)).on_input(Interaction::WorkspaceIdInput))
                    .spacing(6)
            )
            .push(
                Column::new()
                    .push(Text::new("Workspace Description"))
                    .push(TextInput::new("Optionally write a description", &self.workspace_creation_description_text).width(Length::Fixed(250f32)).on_input(Interaction::WorkspaceDescriptionInput))
                    .spacing(6)
            )
            .push(
                Button::new(Text::new("Create Workspace"))
                    .style(small_button)
                    .on_press(Interaction::ProcessNewWorkspace)
            )
            .push(
                Button::new(Text::new("Back to Workspaces List"))
                    .style(small_button)
                    .on_press(Interaction::DisplayWorkspacesList)
            )
            .padding(25)
            .spacing(25)
            .width(Length::Fill)
            .align_x(Horizontal::Center)
            .into()
    }

    fn display_content<'a>(&self, structure: &'a BTreeMap<String, PathInfo>, indentation: f32, vec: &mut Vec<Element<'a, Interaction>>) {
        for (_, value) in structure {
            let is_dir = matches!(&value.content, PathContent::Directory(_));
            let modifier = Button::new(
                Svg::new(svg::Handle::from_memory(if is_dir { APPEND } else { REPLACE })).width(Length::Fixed(16f32))
                    .style(|t, s| advanced_svg(Color::from_rgb8(0, 125, 125), t, s))
            ).style(small_button).on_press(Interaction::SelectFiles(is_dir, value.path.clone()));
            let rename = Button::new(
                Svg::new(svg::Handle::from_memory(RENAME)).width(Length::Fixed(16f32))
                    .style(|t, s| advanced_svg(Color::from_rgb8(0, 255, 0), t, s))
            ).style(small_button).on_press(Interaction::RefactorFiles(value.path.clone()));
            let remove = Button::new(
                Svg::new(svg::Handle::from_memory(REMOVE)).width(Length::Fixed(16f32))
                    .style(|t, s| advanced_svg(Color::from_rgb8(255, 0, 0), t, s))
            ).style(small_button).on_press(Interaction::EraseFiles(value.path.clone()));
            let operations = Container::new(
                Row::new()
                    .push(modifier).push(rename).push(remove)
                    .spacing(2.5)
                    .align_y(Vertical::Center)
            ).padding(Padding::new(0.0).right(2.5)).center_y(Length::Fill).align_right(Length::Fill);
            match &value.content {
                PathContent::File(info) => {
                    let file = if !self.refactors.contains_key(&value.path) {
                        widget::hover(
                            Button::new(Text::new(&info.name))
                                .padding(Padding::new(3.0).left(9.0))
                                .width(Length::Fill)
                                .style(small_button)
                                .on_press(Interaction::ViewFile((&value.sha).to_string(), (&value).path.to_string())),
                            operations
                        )
                    } else {
                        TextInput::new("Rename File", self.refactors.get(&value.path).unwrap())
                            .on_input(|input| Interaction::RefactorInput(value.path.to_string(), input))
                            .on_submit(Interaction::PushRefactor(value.path.to_string(), value.sha.to_string()))
                            .into()
                    };
                    if value.contains(&self.workspace_content_filter) {
                        vec.push(Container::new(file).width(Length::Fixed(350f32)).padding(Padding::new(2.5).left(2.5 + indentation)).into());
                    }
                }
                PathContent::Directory(info) => {
                    let cds = self.collapsed_directories.clone();
                    let directory = if !self.refactors.contains_key(&value.path) {
                        let management_button = if cds.contains(&value.path) {
                            Button::new(Svg::new(svg::Handle::from_memory(EXPAND)).width(Length::Fixed(16f32)).style(default_svg))
                                .padding(3)
                                .style(small_button)
                                .on_press(Interaction::ExpandDirectory(value.path.to_string()))
                        } else {
                            Button::new(Svg::new(svg::Handle::from_memory(COLLAPSE)).width(Length::Fixed(16f32)).style(default_svg))
                                .padding(3)
                                .style(small_button)
                                .on_press(Interaction::CollapseDirectory(value.path.to_string()))
                        };
                        widget::hover(
                            Button::new(
                                Row::new()
                                    .push(management_button)
                                    .push(Svg::new(svg::Handle::from_memory(FOLDER)).width(Length::Fixed(16f32)).style(default_svg))
                                    .push(Text::new(&info.name))
                                    .spacing(5)
                                    .align_y(Vertical::Center)
                            ).width(Length::Fill).padding(3).style(small_button),
                            operations
                        )
                    } else {
                        TextInput::new("Rename Directory", self.refactors.get(&value.path).unwrap())
                            .on_input(|input| Interaction::RefactorInput(value.path.to_string(), input))
                            .on_submit(Interaction::PushRefactor(value.path.to_string(), value.sha.to_string()))
                            .into()
                    };
                    if value.contains(&self.workspace_content_filter) {
                        vec.push(Container::new(directory).width(Length::Fixed(350f32)).padding(Padding::new(2.5).left(2.5 + indentation)).into());
                        if !cds.contains(&value.path) {
                            let mut inner = vec![];
                            self.display_content(&info.contents, indentation + 15.0, &mut inner);
                            vec.push(Column::new().extend(inner).into());
                        }
                    }
                }
            }
        }
    }

    fn workspace_content(&self) -> Element<'_, Interaction, Theme, Renderer> {
        match &self.workspace_content {
            Some(workspace_content) => {
                let append = Button::new(
                    Svg::new(svg::Handle::from_memory(APPEND)).width(Length::Fixed(16f32))
                        .style(|t, s| advanced_svg(Color::from_rgb8(0, 125, 125), t, s))
                ).style(small_button).on_press(Interaction::SelectFiles(true, "".to_string()));
                let workspace = self.selected_workspace.clone().unwrap();
                let repository = widget::hover(
                    Button::new(
                        Row::new()
                            .push(Svg::new(svg::Handle::from_memory(REPOSITORY)).width(Length::Fixed(16f32)).style(default_svg))
                            .push(Text::new(format!("{}/{}", workspace.project.source_owner, workspace.project.source_name)))
                            .push(Text::new(workspace.workspace_full_id).size(9))
                            .spacing(5)
                            .align_y(Vertical::Center)
                    ).width(Length::Fill).padding(6).style(small_button),
                    Container::new(append).padding(Padding::new(0.0).right(2.5)).center_y(Length::Fill).align_right(Length::Fill)
                );
                let mut root = vec![];
                self.display_content(workspace_content, 15.0, &mut root);
                let scroll = Scrollable::new(
                    Column::new()
                        .push(Container::new(repository).width(Length::Fixed(350f32)).padding(2.5))
                        .extend(root).padding(10)).width(Length::Fixed(374f32)
                );
                let mut content: Vec<Element<Interaction, Theme, Renderer>> = vec![];
                let o_view = self.viewed_file_content.clone();
                if matches!(o_view, Some(_)) {
                    let name = self.viewed_file_name.clone().unwrap();
                    if name.ends_with("png") || name.ends_with("jpg") || name.ends_with("jpeg") {
                        content.push(
                            Container::new(
                                Viewer::new(image::Handle::from_bytes(o_view.unwrap()))
                                    .width(Length::Fill)
                                    .height(Length::Fill)
                                    .filter_method(FilterMethod::Nearest)
                                    .scale_step(0.5)
                            ).center(Length::FillPortion(9)).padding(Padding::new(15f32).bottom(5)).into()
                        )
                    }
                    else if name.ends_with("md") {
                        let value = String::from_utf8(o_view.unwrap()).unwrap();
                        let items = markdown::parse(&value).map(|x| &*Box::leak(Box::new(x)));
                        let display: Element<Interaction, Theme, Renderer> = widget::markdown(items, markdown::Settings::default(), markdown::Style::from_palette(Palette::DARK)).map(Interaction::OpenLink);
                        content.push(
                            Container::new(Scrollable::new(display).spacing(10))
                                .width(Length::Fill)
                                .height(Length::FillPortion(9))
                                .padding(Padding::new(15f32).bottom(5))
                                .into()
                        );
                    }
                }
                let bottom_bar = if self.modification.present() {
                    Container::new(
                        Row::new()
                            .push(TextInput::new("Modification Name", &self.modification_name).on_input(Interaction::ModificationNameInput))
                            .push(Button::new("Send Changes").style(small_button).on_press(Interaction::SendChanges))
                            .push(Button::new("Cancel Changes").style(small_button).on_press(Interaction::ResetChanges))
                            .spacing(10)
                            .align_y(Vertical::Center)
                    )
                } else {
                    let mut unmodified: Vec<Element<Interaction, Theme, Renderer>> = vec![];
                    unmodified.push(
                        Button::new("Return back to Workspaces List")
                            .style(small_button)
                            .on_press(Interaction::DisplayWorkspacesList)
                            .into()
                    );
                    Container::new(Row::new().extend(unmodified).spacing(10).align_y(Vertical::Center)).align_right(Length::Fill)
                };
                content.push(
                    Container::new(bottom_bar)
                        .align_bottom(Length::FillPortion(1))
                        .padding(Padding::new(10f32).top(0f32))
                        .into()
                );
                Row::new()
                    .push(
                        Column::new()
                            .push(
                                TextInput::new("Filter Elements", &self.workspace_content_filter)
                                    .width(Length::Fixed(374f32))
                                    .on_input(Interaction::FilterWorkspaceContent)
                            )
                            .push(scroll)
                            .spacing(3)
                    )
                    .push(
                        Column::new()
                            .extend(content)
                            .height(Length::Fill)
                    )
                    .into()
            }
            None => Row::new().into()
        }
    }
}
