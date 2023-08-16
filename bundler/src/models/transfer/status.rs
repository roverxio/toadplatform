pub enum Status {
    SUCCESS,
    PENDING,
    FAILED,
}

impl Status {
    pub fn to_string(&self) -> String {
        match self {
            Status::SUCCESS => String::from("success"),
            Status::PENDING => String::from("pending"),
            Status::FAILED => String::from("failed"),
        }
    }
}
