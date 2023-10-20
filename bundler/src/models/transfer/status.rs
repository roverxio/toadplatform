pub enum Status {
    FAILED,
    INITIATED,
    PENDING,
    SUBMITTED,
    SUCCESS,
}

impl Status {
    pub fn to_string(&self) -> String {
        match self {
            Status::FAILED => String::from("failed"),
            Status::INITIATED => String::from("initiated"),
            Status::PENDING => String::from("pending"),
            Status::SUBMITTED => String::from("submitted"),
            Status::SUCCESS => String::from("success"),
        }
    }
}
