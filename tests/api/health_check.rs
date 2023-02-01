use crate::helpers::spawn_app;

// actix_rt::test is the testing equivalent of `actix_web::main`
#[actix_rt::test]
async fn health_check_works() {
    let test_app = spawn_app().await;
    // use reqwest crate to perform HTTP requests against our application
    let client = reqwest::Client::new();

    let response = client
        //  use the address returned from the newly created test app.
        .get(format!("{}/health_check", &test_app.address))
        .send()
        .await
        .expect("Failed to execute request.");

    assert!(response.status().is_success());
    // assert_eq!(Some(0), response.content_length())
}
