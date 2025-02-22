// color-cycle - render color cycle images on the terminal
// Copyright (C) 2025  Mathias Panzenböck
// 
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
// 
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
// 
// You should have received a copy of the GNU General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.

use std::fmt::Display;

#[derive(Debug)]
pub struct Error {
    message: String,
    source: Option<Box<dyn std::error::Error>>,
}

impl Error {
    pub fn new<S>(message: S) -> Self
    where S: Into<String> {
        Self {
            message: message.into(),
            source: None,
        }
    }

    pub fn with_cause<S>(message: S, source: Box<dyn std::error::Error>) -> Self
    where S: Into<String> {
        Self {
            message: message.into(),
            source: Some(source),
        }
    }
}

impl Display for Error {
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(source) = &self.source {
            write!(f, "{}: {source}", self.message)
        } else {
            self.message.fmt(f)
        }
    }
}

impl std::error::Error for Error {
    #[inline]
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        self.source.as_deref()
    }
}

impl From<crate::ilbm::Error> for Error {
    #[inline]
    fn from(value: crate::ilbm::Error) -> Self {
        Self::with_cause("ILBM error", Box::new(value))
    }
}

impl From<std::io::Error> for Error {
    #[inline]
    fn from(value: std::io::Error) -> Self {
        Self::with_cause("IO error", Box::new(value))
    }
}

impl From<serde_json::error::Error> for Error {
    #[inline]
    fn from(value: serde_json::error::Error) -> Self {
        Self::with_cause("JSON error", Box::new(value))
    }
}
