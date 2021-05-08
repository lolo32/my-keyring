use saphir::prelude::*;
use ulid::Ulid;

pub struct GuardCheckUlid {
    id: String,
    err_code: StatusCode,
}

#[guard]
impl GuardCheckUlid {
    pub fn new((id, err_code): (&str, StatusCode)) -> Self {
        Self {
            id: id.to_owned(),
            err_code,
        }
    }

    async fn validate(&self, req: Request) -> Result<Request, StatusCode> {
        let id = req.captures().get(&self.id).unwrap();
        match Ulid::from_string(&id) {
            Ok(_id) => Ok(req),
            Err(_) => Err(self.err_code),
        }
    }
}
