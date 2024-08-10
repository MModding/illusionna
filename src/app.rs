use iced::{Alignment, Element, Length, Renderer, Task, Theme, window};
use iced::widget::{Button, Column, Container, Image, image, Text};
use iced::widget::image::FilterMethod;
use iced::window::icon;
use octocrab::Octocrab;

use crate::wrapper;

#[derive(Debug, Clone)]
enum CrabState {
    Absent,
    Present(Octocrab)
}

#[derive(Debug, Clone)]
pub(crate) enum Display {
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
    display: Display
}

#[derive(Debug, Clone)]
pub enum IllusionnaAppMessage {
    StartDeviceFlow,
    CompleteDeviceFlow(Octocrab)
}

impl IllusionnaApp {

    pub fn new() -> (Self, Task<IllusionnaAppMessage>) {
        let icon_png = icon::from_file_data(include_bytes!("../resources/icon.png").as_slice(), None).unwrap();
        let icon_task = window::get_latest().and_then(move |id| window::change_icon(id, icon_png.clone()));
        (IllusionnaApp { crab: CrabState::Absent, display: Display::GithubConnexion }, icon_task)
    }

    pub fn get_crab(&self) -> &Octocrab {
        return match &self.crab {
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
                return Task::none()
            }
        }
    }

    pub fn view(&self) -> Element<'_, IllusionnaAppMessage, Theme, Renderer> {
        return match &self.display {
            Display::GithubConnexion => {
                let illusionna_title = Image::new(image::Handle::from_bytes(include_bytes!("../resources/title.png").as_slice()))
                    .filter_method(FilterMethod::Nearest)
                    .width(Length::Fixed(426f32))
                    .height(Length::Fixed(240f32));
                let device_auth_text = Text::new("GitHub Authentication");
                let device_auth_button = Button::new("Login to GitHub via Device Flow").on_press(IllusionnaAppMessage::StartDeviceFlow);
                let column = Column::new().push(illusionna_title).push(device_auth_text).push(device_auth_button).align_x(Alignment::Center).spacing(10);
                return Container::new(column).center_x(Length::Fill).center_y(Length::Fill).into();
            }
            Display::ProjectCreation => {
                Container::new(Text::new("")).into()
            }
            Display::ProjectSelection => { Container::new(Text::new("")).into() }
            Display::WorkspaceCreation => { Container::new(Text::new("")).into() }
            Display::WorkspaceSelection => { Container::new(Text::new("")).into() }
            Display::ModificationCreation => { Container::new(Text::new("")).into() }
            Display::WorkingModification => { Container::new(Text::new("")).into() }
        };
    }
}
