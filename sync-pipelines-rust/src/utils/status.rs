pub enum Status {
    SUCCESS,
}

impl Status {
    pub fn to_string(&self) -> String {
        match self {
            Status::SUCCESS => String::from("success"),
        }
    }
}
