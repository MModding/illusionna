use crate::workspace::ProjectInfo;
use crate::{workspace, wrapper};
use iced::alignment::Vertical;
use iced::widget::image::FilterMethod;
use iced::widget::{button, image, scrollable, text, Button, Column, Container, Image, Row, Text};
use iced::window::icon;
use iced::{window, Alignment, Background, Border, Color, Element, Length, Renderer, Shadow, Task, Theme};
use iced::widget::button::{Catalog, Status};
use iced::widget::container::Style;
use octocrab::Octocrab;

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
pub struct IllusionnaApp {
    crab: CrabState,
    display: Display,
    projects: Option<Vec<ProjectInfo>>
}

#[derive(Debug, Clone)]
pub enum IllusionnaAppMessage {
    StartDeviceFlow,
    CompleteDeviceFlow(Octocrab),
    ReceiveProjectInfos(Vec<ProjectInfo>)
}

pub fn sidebar_style(theme: &Theme, status: Status) -> button::Style {
    let color;
    if theme.extended_palette().is_dark {
        color = Color::from_rgb8(46, 45, 62);
    }
    else {
        color = Color::WHITE;
    }
    button::Style {
        background: Some(Background::Color(color)),
        text_color: theme.palette().text,
        border: Border::default(),
        shadow: Shadow::default()
    }
}

pub fn button_style(theme: &Theme, status: Status) -> button::Style {
    button::Style {
        background: Some(Background::Color(Color::from_rgb8(72, 68, 255))),
        text_color: theme.palette().text,
        border: Border::default().rounded(3),
        shadow: Shadow::default()
    }
}

impl IllusionnaApp {

    pub fn new() -> (Self, Task<IllusionnaAppMessage>) {
        let icon_png = icon::from_file_data(include_bytes!("../resources/icon.png").as_slice(), None).unwrap();
        let icon_task = window::get_latest().and_then(move |id| window::change_icon(id, icon_png.clone()));
        (IllusionnaApp { crab: CrabState::Absent, display: Display::GithubConnexion, projects: None }, icon_task)
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

    pub fn update(&mut self, message: IllusionnaAppMessage) -> Task<IllusionnaAppMessage> {
        match message {
            IllusionnaAppMessage::StartDeviceFlow => {
                Task::perform(wrapper::oauth_process(), |result| {
                    return IllusionnaAppMessage::CompleteDeviceFlow(result.unwrap());
                })
            },
            IllusionnaAppMessage::CompleteDeviceFlow(crab) => {
                self.crab = CrabState::Present(crab);
                self.display = Display::ProjectSelection;
                let usable_crab = self.get_crab().clone();
                Task::perform(workspace::get_projects(usable_crab.clone()), |projects| {
                    return IllusionnaAppMessage::ReceiveProjectInfos(projects)
                })
            }
            IllusionnaAppMessage::ReceiveProjectInfos(projects) => {
                self.projects = Some(projects);
                Task::none()
            }
        }
    }

    pub fn view(&self) -> Element<'_, IllusionnaAppMessage, Theme, Renderer> {
        match &self.display {
            Display::GithubConnexion => {
                let illusionna_title = Image::new(image::Handle::from_bytes(include_bytes!("../resources/title.png").as_slice()))
                    .filter_method(FilterMethod::Nearest)
                    .width(Length::Fixed(426f32))
                    .height(Length::Fixed(240f32));
                let device_auth_text = text("GitHub Authentication");
                let device_auth_button = Button::new("Login to GitHub via Device Flow")
                    .style(button_style)
                    .on_press(IllusionnaAppMessage::StartDeviceFlow);
                let column = Column::new().push(illusionna_title).push(device_auth_text).push(device_auth_button).align_x(Alignment::Center).spacing(10);
                Container::new(column).center_x(Length::Fill).center_y(Length::Fill).into()
            }
            Display::ProjectSelection => {
                let projects: Column<IllusionnaAppMessage> = match &self.projects {
                    Some(result) => {
                        Column::new().extend(result.into_iter().map(|project| {
                            let content: Column<IllusionnaAppMessage> = Column::new()
                                .push(
                                    text(&project.fork_name).size(16)
                                )
                                .push(
                                    Row::new()
                                        .push(Image::new(&project.source_owner_icon).width(Length::Fixed(24f32)).height(24f32))
                                        .push(text(format!("{} - {}", &project.source_owner, &project.source_name)).size(12))
                                        .align_y(Vertical::Center)
                                        .spacing(10)
                                ).spacing(10);
                            Button::new(content).width(Length::Fixed(256f32)).height(64f32).style(sidebar_style).into()
                        }))
                    }
                    None => Column::new()
                };
                scrollable(projects).anchor_left().into()
            }
            Display::ProjectCreation => { Container::new(Text::new("")).into() }
            Display::WorkspaceSelection => { Container::new(Text::new("")).into() }
            Display::WorkspaceCreation => { Container::new(Text::new("")).into() }
            Display::ModificationCreation => { Container::new(Text::new("")).into() }
            Display::WorkingModification => { Container::new(Text::new("")).into() }
        }
    }
}
