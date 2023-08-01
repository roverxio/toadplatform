use serde::Deserialize;

#[derive(Deserialize, Clone)]
pub struct Server {
    pub host: String,
    pub port: String,
    pub log_level: String,
}

impl Server {
    pub fn url(&self) -> String {
        self.host.as_str().to_owned() + &String::from(":") + self.port.as_str()
    }
}
