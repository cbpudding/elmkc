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

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(content = "data", rename_all = "lowercase", tag = "type")]
pub enum InboundData {
    Accepted { message: String },
    AuthLevel { value: usize },
    Chat {
        auth: usize,
        author: String,
        author_color: String,
        author_id: usize,
        author_level: usize,
        donate_value: String,
        id: usize,
        message: String,
        reply: usize,
        time: usize
    },
    Delete { messages: Vec<usize> },
    GetUserConf { color: String, name: String },
    Join { name: String },
    Part { name: String },
    ServerMsg { message: String },
    Status { status: UserStatus }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct InboundMessage {
    #[serde(flatten)]
    data: InboundData,
}

impl InboundMessage {
    pub fn data(&self) -> &InboundData {
        &self.data
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "lowercase", tag = "auth")]
pub enum MessageAuth {
    Google { token: String },
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(content = "data", rename_all = "lowercase", tag = "type")]
enum OutboundData {
    Hello { last_message: isize },
    Message { reply: usize, text: String },
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct OutboundMessage {
    #[serde(flatten)]
    auth: MessageAuth,
    #[serde(flatten)]
    data: OutboundData,
}

impl OutboundMessage {
    pub fn hello(auth: &MessageAuth) -> Self {
        Self {
            auth: auth.clone(),
            data: OutboundData::Hello { last_message: -1 },
        }
    }

    pub fn message<S: Into<String>>(auth: &MessageAuth, content: S, reply: Option<usize>) -> Self {
        Self {
            auth: auth.clone(),
            data: OutboundData::Message {
                reply: if let Some(id) = reply { id } else { 0 },
                text: content.into(),
            },
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum UserStatus {
    Authenticated,
    Banned,
    NameExists,
    NameInvalid,
    NameLength,
    NameTimeout,
    Rename,
    SetUserConf,
    Unauthenticated
}