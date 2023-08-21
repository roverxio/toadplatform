pub enum Status {
    PENDING,
}

impl Status {
    pub fn to_string(&self) -> String {
        match self {
            Status::PENDING => String::from("pending"),
        }
    }
}
