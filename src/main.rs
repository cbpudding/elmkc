/* An open source desktop client for ChatKC servers
Copyright (C) 2023 Alexander Hill

This program is free software: you can redistribute it and/or modify
it under the terms of the GNU Affero General Public License as published
by the Free Software Foundation, either version 3 of the License, or
(at your option) any later version.

This program is distributed in the hope that it will be useful,
but WITHOUT ANY WARRANTY; without even the implied warranty of
MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
GNU Affero General Public License for more details.

You should have received a copy of the GNU Affero General Public License
along with this program.  If not, see <https://www.gnu.org/licenses/>. */

mod config;
mod protocol;
mod socket;

use crate::{
    config::Configuration,
    protocol::{InboundData, MessageAuth, OutboundMessage},
};
use chrono::{DateTime, Local, TimeZone};
use iced::{
    executor,
    widget::{column, row, scrollable, text, text_input, Column},
    Application, Color, Command, Element, Length, Renderer, Settings, Subscription, Theme,
};
use ketos::Interpreter;
use once_cell::sync::Lazy;
use std::path::Path;

static MESSAGE_LOG: Lazy<scrollable::Id> = Lazy::new(scrollable::Id::unique);

#[derive(Clone, Debug)]
enum Event {
    InputChange(String),
    SendMessage,
    Socket(socket::Event),
}

struct ElmKC {
    auth: MessageAuth,
    config: Configuration,
    input: String,
    messages: Vec<Message>,
    scripts: Vec<Interpreter>,
    socket: SocketState,
    username: Option<String>,
}

impl Application for ElmKC {
    type Executor = executor::Default;
    type Flags = ();
    type Message = Event;
    type Theme = Theme;

    fn new(_flags: Self::Flags) -> (Self, Command<Self::Message>) {
        let config = Configuration::load("config.toml");
        let scripts = config
            .scripts()
            .iter()
            .map(|path| {
                let interp = Interpreter::new();
                interp.run_file(Path::new(path)).unwrap();
                interp
            })
            .collect();
        (
            Self {
                auth: MessageAuth::Google {
                    token: config.token().clone(),
                },
                config,
                input: String::new(),
                messages: Vec::new(),
                scripts,
                socket: SocketState::Disconnected,
                username: None,
            },
            Command::none(),
        )
    }

    fn subscription(&self) -> Subscription<Event> {
        socket::connect(self.auth.clone(), self.config.server().clone()).map(Event::Socket)
    }

    fn theme(&self) -> Self::Theme {
        Theme::Dark
    }

    fn title(&self) -> String {
        if let Some(username) = &self.username {
            format!("{username}@{} - ElmKC", self.config.server())
        } else {
            format!("{} - ElmKC", self.config.server())
        }
    }

    fn update(&mut self, message: Self::Message) -> Command<Self::Message> {
        match message {
            Event::InputChange(s) => {
                self.input = s;
                Command::none()
            }
            Event::SendMessage => match &mut self.socket {
                SocketState::Connected(connection) => {
                    let payload = OutboundMessage::message(&self.auth, &self.input, None);
                    connection.send(payload);
                    self.input.clear();
                    Command::none()
                }
                SocketState::Disconnected => Command::none(),
            },
            Event::Socket(event) => match event {
                socket::Event::Connected(connection) => {
                    self.socket = SocketState::Connected(connection);
                    Command::none()
                }
                socket::Event::Disconnected => {
                    self.socket = SocketState::Disconnected;
                    Command::none()
                }
                socket::Event::Received(message) => match message.data() {
                    InboundData::Chat {
                        author,
                        author_color,
                        message,
                        id,
                        time,
                        ..
                    } => {
                        let color = if author_color.len() == 6 {
                            if let Ok(raw) = u32::from_str_radix(&author_color, 16) {
                                let red = ((raw & 0xFF0000) >> 16) as u8;
                                let green = ((raw & 0xFF00) >> 8) as u8;
                                let blue = (raw & 0xFF) as u8;
                                Some(Color::from_rgb8(red, green, blue))
                            } else {
                                None
                            }
                        } else {
                            None
                        };
                        let mut raw_content = String::new();
                        html_escape::decode_html_entities_to_string(&message, &mut raw_content);
                        // Am I doing this right? ~Bread
                        let timestamp = Local.timestamp_millis_opt(*time as _).unwrap();
                        self.messages.push(Message::Normal {
                            author: author.clone(),
                            color,
                            content: raw_content,
                            id: *id,
                            timestamp,
                        });
                        scrollable::snap_to(MESSAGE_LOG.clone(), scrollable::RelativeOffset::END)
                    }
                    InboundData::Delete { messages } => {
                        let mut victims = Vec::new();
                        for i in 0..self.messages.len() {
                            if let Message::Normal { id, .. } = self.messages[i] {
                                if messages.contains(&id) {
                                    victims.push(i);
                                }
                            }
                        }
                        // This is probably the correct way to handle this
                        // since the indices might change, but I'm too tired to
                        // think properly. ~Bread
                        victims.sort();
                        for i in (0..victims.len()).rev() {
                            self.messages.remove(i);
                        }
                        scrollable::snap_to(MESSAGE_LOG.clone(), scrollable::RelativeOffset::END)
                    }
                    InboundData::GetUserConf { name, .. } => {
                        self.username = Some(name.clone());
                        Command::none()
                    }
                    InboundData::Join { name } => {
                        self.messages.push(Message::Join(name.clone()));
                        scrollable::snap_to(MESSAGE_LOG.clone(), scrollable::RelativeOffset::END)
                    }
                    InboundData::Part { name } => {
                        self.messages.push(Message::Leave(name.clone()));
                        scrollable::snap_to(MESSAGE_LOG.clone(), scrollable::RelativeOffset::END)
                    }
                    InboundData::ServerMsg { message } => {
                        self.messages.push(Message::System(message.clone()));
                        scrollable::snap_to(MESSAGE_LOG.clone(), scrollable::RelativeOffset::END)
                    }
                    _ => Command::none(),
                },
            },
        }
    }

    fn view(&self) -> Element<'_, Self::Message, Renderer<Self::Theme>> {
        column![
            scrollable(
                Column::with_children(
                    self.messages
                        .iter()
                        .cloned()
                        .map(|msg| {
                            match msg {
                                Message::Join(name) => Element::from(
                                    text(format!("+{name}"))
                                        .size(self.config.text_size)
                                        .style(Color::from_rgb8(178, 245, 178)),
                                ),
                                Message::Leave(name) => Element::from(
                                    text(format!("-{name}"))
                                        .size(self.config.text_size)
                                        .style(Color::from_rgb8(245, 178, 178)),
                                ),
                                Message::Normal {
                                    author,
                                    color,
                                    content,
                                    timestamp,
                                    ..
                                } => {
                                    let mut name = text(author);
                                    if let Some(c) = color {
                                        name = name.style(c);
                                    }
                                    Element::from(row![
                                        text(timestamp.format(&self.config.timestamp))
                                            .style(Color::from_rgb8(127, 127, 127))
                                            .size(self.config.text_size),
                                        name.size(self.config.text_size),
                                        text(": ").size(self.config.text_size),
                                        text(content).size(self.config.text_size)
                                    ])
                                }
                                Message::System(content) => Element::from(Column::with_children(
                                    content
                                        .split("<br>")
                                        .map(|t| {
                                            text(t)
                                                .style(Color::from_rgb8(127, 127, 127))
                                                .size(self.config.text_size)
                                        })
                                        .map(Element::from)
                                        .collect(),
                                )),
                            }
                        })
                        .collect()
                )
                .width(Length::Fill)
            )
            .id(MESSAGE_LOG.clone())
            .height(Length::Fill),
            text_input("Message", &self.input, |s| { Event::InputChange(s) })
                .on_submit(Event::SendMessage)
                .size(self.config.text_size)
        ]
        .height(Length::Fill)
        .width(Length::Fill)
        .into()
    }
}

#[derive(Clone)]
enum Message {
    Join(String),
    Leave(String),
    Normal {
        author: String,
        color: Option<Color>,
        content: String,
        id: usize,
        timestamp: DateTime<Local>,
    },
    System(String),
}

enum SocketState {
    Connected(socket::Connection),
    Disconnected,
}

fn main() -> iced::Result {
    ElmKC::run(Settings::default())
}
