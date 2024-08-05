use iced::{Alignment, Application, Command, Element, Length, Renderer, Theme};
use iced::widget::{Button, Column, Container, Text};

#[derive(Debug, Clone, Copy)]
pub struct IllusionnaApp;

#[derive(Debug, Clone)]
pub enum IllusionnaAppMessage {
    OpenDevicePage,
}

impl Application for IllusionnaApp {
    type Executor = iced::executor::Default;
    type Message = IllusionnaAppMessage;
    type Theme = Theme;
    type Flags = ();

    fn new(_flags: ()) -> (Self, Command<Self::Message>) {
        // wrapper::authorized_instance().await.unwrap();
        (IllusionnaApp, Command::none())
    }

    fn title(&self) -> String {
        String::from("Illusionna")
    }

    fn update(&mut self, message: Self::Message) -> Command<Self::Message> {
        match message {
            IllusionnaAppMessage::OpenDevicePage => {
                Command::none()
            }
        }
    }

    fn view(&self) -> Element<'_, Self::Message, Self::Theme, Renderer> {
        let device_auth_text = Text::new("GitHub Authentication");
        let device_auth_button = Button::new("Login to GitHub via Device Flow").on_press(IllusionnaAppMessage::OpenDevicePage);
        let column = Column::new().push(device_auth_text).push(device_auth_button).align_items(Alignment::Center).spacing(10);
        return Container::new(column).center_x().center_y().width(Length::Fill).height(Length::Fill).into();
    }
}
