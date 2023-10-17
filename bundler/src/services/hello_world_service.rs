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

#[cfg(test)]
mod test {
    use crate::models::HelloWorld;
    use crate::services::HelloWorldService;

    #[actix_web::test]
    async fn test_hello_world_service() {
        let data = HelloWorldService::hello_world().unwrap();
        assert_eq!(
            data,
            HelloWorld {
                name: "Hello world!".to_string()
            }
        )
    }
}
