pub enum Status {
    FAILED,
    PENDING,
    SUCCESS,
}

impl Status {
    pub fn to_string(&self) -> String {
        match self {
            Status::FAILED => String::from("failed"),
            Status::PENDING => String::from("pending"),
            Status::SUCCESS => String::from("success"),
        }
    }
}
