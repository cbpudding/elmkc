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

use crate::protocol::{InboundMessage, MessageAuth, OutboundMessage};
use async_tungstenite::tungstenite;
use futures::{channel::mpsc, SinkExt, StreamExt};
use iced::subscription::{self, Subscription};

#[derive(Clone, Debug)]
pub struct Connection(mpsc::Sender<OutboundMessage>);

impl Connection {
    pub fn send(&mut self, payload: OutboundMessage) {
        self.0.try_send(payload).unwrap();
    }
}

#[derive(Clone, Debug)]
pub enum Event {
    Connected(Connection),
    Disconnected,
    Received(InboundMessage),
}

enum State {
    Connected(
        MessageAuth,
        String,
        async_tungstenite::WebSocketStream<async_tungstenite::tokio::ConnectStream>,
        mpsc::Receiver<OutboundMessage>,
    ),
    Disconnected(MessageAuth, String),
}

pub fn connect(auth: MessageAuth, server: String) -> Subscription<Event> {
    struct Connect;

    subscription::unfold(
        std::any::TypeId::of::<Connect>(),
        State::Disconnected(auth, server),
        |state| async move {
            match state {
                State::Connected(auth, server, mut websock, mut input) => {
                    let mut fused_websock = websock.by_ref().fuse();

                    futures::select! {
                        received = fused_websock.select_next_some() => {
                            match received {
                                Ok(tungstenite::Message::Text(message)) => {
                                    (
                                        Some(Event::Received(serde_json::from_str(&message).unwrap())),
                                        State::Connected(auth, server, websock, input)
                                    )
                                },
                                Ok(_) => (None, State::Connected(auth, server, websock, input)),
                                Err(_) => (Some(Event::Disconnected), State::Disconnected(auth, server))
                            }
                        }

                        message = input.select_next_some() => {
                            let result = websock.send(tungstenite::Message::Text(serde_json::to_string(&message).unwrap())).await;

                            if result.is_ok() {
                                (None, State::Connected(auth, server, websock, input))
                            } else {
                                (Some(Event::Disconnected), State::Disconnected(auth, server))
                            }
                        }
                    }
                }
                State::Disconnected(auth, server) => {
                    match async_tungstenite::tokio::connect_async(format!("wss://{server}:2002/"))
                        .await
                    {
                        Ok((websock, _)) => {
                            let (mut sender, receiver) = mpsc::channel(100);

                            sender.send(OutboundMessage::hello(&auth)).await.unwrap();

                            (
                                Some(Event::Connected(Connection(sender))),
                                State::Connected(auth, server, websock, receiver),
                            )
                        }
                        Err(_) => {
                            tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;

                            (Some(Event::Disconnected), State::Disconnected(auth, server))
                        }
                    }
                }
            }
        },
    )
}
