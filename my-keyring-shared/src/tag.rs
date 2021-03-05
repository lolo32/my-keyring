use ulid::Ulid;

#[derive(Debug, Clone)]
pub struct Tag {
    id: Ulid,
    name: String,
}

impl Tag {
    pub fn new(name: &str) -> Self {
        Self {
            id: Ulid::new(),
            name: name.to_owned(),
        }
    }

    pub fn set_name(&mut self, new_name: &str) {
        self.name = new_name.to_owned();
    }

    pub fn get_name(&self) -> &String {
        &self.name
    }

    pub fn get_id(&self) -> Ulid {
        self.id
    }
}

#[derive(Debug, Clone)]
pub struct TagPool {
    tags: Vec<Tag>,
}

impl TagPool {
    pub fn add_tag(&mut self, name: &str) {
        self.del_tag(name);
        self.tags.push(Tag::new(name));
    }

    pub fn del_tag(&mut self, name: &str) {
        self.tags.retain(|t| t.name != name);
    }

    pub fn get_tag_id(&self, name: &str) -> Option<Ulid> {
        for tag in self.tags {
            if tag.get_name() == name {
                return Some(tag.get_id());
            }
        }
        None
    }
}

pub trait Tags {
    fn add_tag(&mut self, tag_id: Ulid) {
        self.del_tag(tag_id);
        self.tags.push(tag_id);
        self.tags.sort_by(|a, b| a.get_name().cmp(b.get_name()));
    }

    fn del_tag(&mut self, tag_id: Ulid) {
        self.tags.retain(|t| t != tag_id);
    }
}
