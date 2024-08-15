use crate::workspace::ProjectInfo;
use crate::wrapper::AccountInfo;
use crate::{workspace, wrapper};
use iced::alignment::{Horizontal, Vertical};
use iced::widget::button::Status;
use iced::widget::image::FilterMethod;
use iced::widget::{button, image, scrollable, text, Button, Column, Container, Image, Row, Text, TextInput};
use iced::window::icon;
use iced::{window, Alignment, Background, Border, Color, Element, Length, Renderer, Shadow, Task, Theme};
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
    ProjectCreation,
    ProjectSelection,
    WorkspaceCreation,
    WorkspaceSelection,
    ModificationCreation,
    WorkingModification
}

#[derive(Debug, Clone)]
pub enum ReferenceValidation {
    Valid,
    Invalid(String),
    Unspecified
}

#[derive(Debug, Clone)]
pub struct IllusionnaApp {
    crab: CrabState,
    display: Display,
    projects: Option<Vec<ProjectInfo>>,
    selected_project: Option<ProjectInfo>,
    project_creation_text: String,
    project_creation_validation: ReferenceValidation,
    account: Option<AccountInfo>
}

#[derive(Debug, Clone)]
pub enum Interaction {
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
    OpenCreatedProject(ProjectInfo)
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
                crab: CrabState::Absent,
                display: Display::GithubConnexion,
                projects: None,
                selected_project: None,
                project_creation_text: "".to_string(),
                project_creation_validation: ReferenceValidation::Unspecified,
                account: None
            },
            icon_task
        )
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
                            Interaction::OpenCreatedProject(result)
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
                Task::none()
            }
            Interaction::OpenCreatedProject(project) => {
                self.selected_project = Some(project);
                self.display = Display::WorkspaceSelection;
                Task::none()
            }
        }
    }

    pub fn view(&self) -> Element<'_, Interaction, Theme, Renderer> {
        match &self.display {
            Display::GithubConnexion => self.github_connection(),
            Display::ProjectSelection => self.project_selection(),
            Display::ProjectCreation => self.project_creation(),
            Display::WorkspaceSelection => self.workspace_selection(),
            Display::WorkspaceCreation => self.workspace_creation(),
            Display::ModificationCreation => self.modification_creation(),
            Display::WorkingModification => self.working_modification()
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
                        .height(64f32)
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
                Container::new(Image::new(image::Handle::from_bytes(ICON)).width(Length::Fixed(48f32)).height(48f32))
                    .width(Length::Fill)
                    .height(Length::Fill)
                    .align_x(Horizontal::Center)
                    .align_y(Vertical::Center)
                    .into()
            }
        }
    }

    fn project_creation(&self) -> Element<'_, Interaction, Theme, Renderer> {
        Text::new("").into()
    }

    fn workspace_selection(&self) -> Element<'_, Interaction, Theme, Renderer> {
        Text::new("").into()
    }

    fn workspace_creation(&self) -> Element<'_, Interaction, Theme, Renderer> {
        Text::new("").into()
    }

    fn modification_creation(&self) -> Element<'_, Interaction, Theme, Renderer> {
        Text::new("").into()
    }

    fn working_modification(&self) -> Element<'_, Interaction, Theme, Renderer> {
        Text::new("").into()
    }
}
