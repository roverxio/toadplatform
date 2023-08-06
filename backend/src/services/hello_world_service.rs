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


#[cfg(test)]
mod tests {
        use super::*;
        #[test]
        fn test_hello() {
               let hello_service = HelloWorldService{};
               assert_eq!(hello_service.hello_world(), Ok(HelloWorld {
                        name: "Hello world!".to_string(),
               }));
        }
}