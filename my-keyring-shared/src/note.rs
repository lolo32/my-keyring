use ulid::Ulid;

use crate::tag::{Tag, Tags};

pub struct Note {
    id: Ulid,
    message: String,
    tags: Vec<Tag>,
}

impl Note {
    pub fn new(message: &str) -> Self {
        Self {
            id: Ulid::new(),
            message: message.to_owned(),
            tags: Vec::new(),
        }
    }
}

impl Tags for Note {}
