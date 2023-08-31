pub enum Status {
    SUCCESS,
    FAILED,
    PENDING,
}

impl Status {
    pub fn to_string(&self) -> String {
        match self {
            Status::SUCCESS => String::from("success"),
            Status::FAILED => String::from("failed"),
            Status::PENDING => String::from("pending"),
        }
    }
}
