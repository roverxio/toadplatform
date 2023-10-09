use actix_web::Error;

use crate::models::HelloWorld;

#[derive(Clone)]
pub struct HelloWorldService;

impl HelloWorldService {
    pub fn hello_world() -> Result<HelloWorld, Error> {
        Ok(HelloWorld {
            name: "Hello world!".to_string(),
        })
    }
}
