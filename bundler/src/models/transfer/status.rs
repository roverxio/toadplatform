pub enum Status {
    FAILED,
    PENDING,
}

impl Status {
    pub fn to_string(&self) -> String {
        match self {
            Status::FAILED => String::from("failed"),
            Status::PENDING => String::from("pending"),
        }
    }
}
