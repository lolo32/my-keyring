use ulid::Ulid;

use crate::tag::Tags;

#[derive(Debug)]
pub struct Note {
    id: Ulid,
    message: String,
    tags: Vec<Ulid>,
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

impl Tags for Note {
    fn tags(&mut self) -> &mut Vec<Ulid> {
        &mut self.tags
    }
}
