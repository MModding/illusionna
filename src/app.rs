use crate::workspace::{ProjectInfo, WorkspaceInfo};
use crate::wrapper::AccountInfo;
use crate::{workspace, wrapper};
use iced::alignment::{Horizontal, Vertical};
use iced::widget::button::Status;
use iced::widget::image::FilterMethod;
use iced::widget::{button, image, scrollable, text, Button, Column, Container, Image, Row, Text, TextInput};
use iced::window::icon;
use iced::{window, Alignment, Background, Border, Color, Degrees, Element, Length, Radians, Renderer, Rotation, Shadow, Subscription, Task, Theme};
use octocrab::Octocrab;
use reqwest::Url;

const ICON: &[u8] = include_bytes!("../resources/icon.png").as_slice();

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
    workspace_creation_title_text: String,
    workspace_creation_id_text: String,
    workspace_creation_description_text: String,
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
    OpenAccountProfile(Url),
    OpenSelectedProject,
    AppendCreatedProject(ProjectInfo),
    ReceiveWorkspaceInfos(Vec<WorkspaceInfo>),
    DisplayProjectList,
    CreateNewWorkspace,
    ProcessNewWorkspace
}

pub fn sidebar_button(theme: &Theme, status: Status) -> button::Style {
    let color;
    if status == Status::Hovered || status == Status::Pressed {
        color = Color::from_rgb8(72, 68, 255)
    }
    else {
        if theme.extended_palette().is_dark {
            color = Color::from_rgb8(46, 45, 62);
        } else {
            color = Color::WHITE;
        }
    }
    button::Style {
        background: Some(Background::Color(color)),
        text_color: theme.palette().text,
        border: Border::default(),
        shadow: Shadow::default()
    }
}

pub fn large_button(theme: &Theme, status: Status) -> button::Style {
    button::Style { border: Border::default().rounded(10), ..sidebar_button(theme, status) }
}

pub fn small_button(theme: &Theme, _: Status) -> button::Style {
    button::Style {
        background: Some(Background::Color(Color::from_rgb8(72, 68, 255))),
        text_color: theme.palette().text,
        border: Border::default().rounded(3),
        shadow: Shadow::default()
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
                workspace_creation_title_text: "".to_string(),
                workspace_creation_id_text: "".to_string(),
                workspace_creation_description_text: "".to_string()
            },
            icon_task
        )
    }

    pub fn ticker(&self) -> Subscription<Interaction> {
        window::frames().map(|x| Interaction::Tick)
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
                Task::perform(wrapper::oauth_process(), |result| {
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
            Interaction::OpenAccountProfile(url) => {
                webbrowser::open(url.as_str()).unwrap();
                Task::none()
            }
            Interaction::OpenSelectedProject => {
                self.display = Display::WorkspaceSelection;
                let crab = self.get_crab().clone();
                let project = self.selected_project.clone().unwrap();
                Task::perform(workspace::get_workspaces(crab.clone(), project.clone()), |workspaces| {
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
            Interaction::DisplayProjectList => {
                self.display = Display::ProjectSelection;
                self.workspaces = None;
                Task::none()
            }
            Interaction::CreateNewWorkspace => {
                self.display = Display::WorkspaceCreation;
                Task::none()
            }
            Interaction::ProcessNewWorkspace => {
                Task::none()
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
                                        .push(Image::new(&selected_project.source_owner_icon).width(Length::Fixed(64f32)).height(64f32))
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
                                .push(Image::new(&info.avatar).width(Length::Fixed(48f32)).height(48f32))
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
                                .on_press(Interaction::OpenAccountProfile(info.clone().profile));
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
                        .push(Text::new(id).size(12))
                        .spacing(6)
                ).padding(6).width(Length::Fixed(320f32)).height(Length::Fixed(96f32))
            ).style(large_button)
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
                        row.push(display_workspace(first_info.workspace_title.to_string(), first_info.workspace_id.to_string()));
                        if i + 1 < workspaces.len() {
                            let second_info = workspaces.get(i + 1).unwrap();
                            row.push(display_workspace(second_info.workspace_title.to_string(), second_info.workspace_id.to_string()))
                        }
                        column.push(Row::new().extend(row.into_iter().map(|x| x.into())).spacing(6));
                    }
                    if workspaces.len() % 2 == 1 {
                        let last = workspaces.last().unwrap();
                        column.push(
                            Row::new()
                                .push(display_workspace(last.workspace_title.to_string(), last.workspace_id.to_string()))
                        );
                    }
                    Column::new()
                        .extend(column.into_iter().map(|x| x.into()))
                        .width(Length::Fill).align_x(Horizontal::Center)
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
                            .push(Image::new(selected_project.source_owner_icon).width(Length::Fixed(64f32)).height(64f32))
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
                Row::new()
                    .push(
                        Column::new()
                            .push(Button::new(Text::new("Return back to Projects List")).style(small_button).on_press(Interaction::DisplayProjectList))
                    )
                    .push(
                        Column::new()
                            .push(Button::new(Text::new("Create New Workspace")).style(small_button).on_press(Interaction::CreateNewWorkspace))
                            .width(Length::Fill)
                            .align_x(Horizontal::Right)
                    )
                    .height(Length::Fill)
                    .align_y(Vertical::Bottom)
                    .padding(10)
            )
            .into()
    }

    fn workspace_creation(&self) -> Element<'_, Interaction, Theme, Renderer> {
        Text::new("").into()
    }

    fn workspace_content(&self) -> Element<'_, Interaction, Theme, Renderer> {
        Text::new("").into()
    }
}
