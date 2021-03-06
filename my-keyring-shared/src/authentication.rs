use fnv::FnvHashMap;
use ulid::Ulid;

use crate::tag::Tags;

#[derive(Debug)]
pub struct Authentication {
    id: Ulid,
    name: String,
    username: String,
    password: String,
    notes: String,
    tags: Vec<Ulid>,
    additional_field: FnvHashMap<String, String>,
}

impl Authentication {
    pub fn new(name: &str, username: &str, password: &str, notes: &str) -> Self {
        Self {
            id: Ulid::new(),
            name: name.to_owned(),
            username: username.to_owned(),
            password: password.to_owned(),
            notes: notes.to_owned(),
            tags: Vec::new(),
            additional_field: Default::default(),
        }
    }
}

impl Tags for Authentication {
    fn tags(&mut self) -> &mut Vec<Ulid> {
        &mut self.tags
    }
}
