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
use std::{fs, path::Path};

#[derive(Deserialize, Serialize)]
pub struct Configuration {
    server: String,
    pub text_size: u16,
    token: String
}

impl Configuration {
    pub fn load<P: AsRef<Path>>(path: P) -> Self {
        if path.as_ref().exists() {
            let buffer = fs::read_to_string(path).unwrap();
            toml::from_str(&buffer).unwrap()
        } else {
            let config = Self::default();
            config.save(path);
            config
        }
    }
    
    pub fn save<P: AsRef<Path>>(&self, path: P) {
        let buffer = toml::to_string_pretty(self).unwrap();
        fs::write(path, buffer).unwrap();
    }

    pub fn server(&self) -> &String {
        &self.server
    }

    pub fn token(&self) -> &String {
        &self.token
    }
}

impl Default for Configuration {
    fn default() -> Self {
        Self {
            server: String::from("server.mattkc.com"),
            text_size: 16,
            token: String::from("Your token here")
        }
    }
}