use crate::helpers::spawn_app;

#[tokio::test]
async fn health_check_works() {
    let test_app = spawn_app().await;
    // use reqwest crate to perform HTTP requests against our application
    let client = reqwest::Client::new();

    dbg!(&test_app.address);

    let response = client
        //  use the address returned from the newly created test app.
        .get(format!("{}/health_check", &test_app.address))
        .send()
        .await
        .expect("Failed to execute request.");

    assert!(response.status().is_success());
    // assert_eq!(Some(0), response.content_length())
}
