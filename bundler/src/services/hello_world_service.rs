use crate::errors::ApiError;
use crate::models::hello_world::HelloWorld;

#[derive(Clone)]
pub struct HelloWorldService {}

impl HelloWorldService {
    pub fn hello_world(&self) -> Result<HelloWorld, ApiError> {
        Ok(HelloWorld {
            name: "Hello world!".to_string(),
        })
    }
}
