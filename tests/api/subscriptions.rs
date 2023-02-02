use wiremock::{
    matchers::{method, path},
    Mock, ResponseTemplate,
};

use crate::helpers::spawn_app;

#[actix_rt::test]
async fn subscribe_returns_a_200_for_valid_data() {
    // Arrange
    let test_app = spawn_app().await;
    let body = "name=spacedaddy&email=space_daddy%40test.com";

    Mock::given(path("/email"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        .expect(1)
        .mount(&test_app.email_server)
        .await;

    // Act
    let response = test_app.post_subscription(body.into()).await;

    // Assert
    assert_eq!(200, response.status().as_u16());
}

#[actix_rt::test]
async fn subscribe_persists_the_new_subscriber() {
    // Arrange
    let app = spawn_app().await;
    let body = "name=spacedaddy&email=space_daddy%40test.com";

    // Act
    app.post_subscription(body.into()).await;

    // Assert
    let saved = sqlx::query!("SELECT email, name, status FROM subscriptions",)
        .fetch_one(&app.db_pool)
        .await
        .expect("Failed to fetch saved subscription");

    assert_eq!(saved.email, "space_daddy@test.com");
    assert_eq!(saved.name, "spacedaddy");
    assert_eq!(saved.status, "pending_confirmation");
}

#[actix_rt::test]
async fn subscribe_sends_a_confirmation_email_for_valid_data() {
    // Arrange
    let app = spawn_app().await;
    let body = "name=spacedaddy&email=space_daddy%40test.com";

    Mock::given(path("/email"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        .expect(1)
        .mount(&app.email_server)
        .await;

    // Act
    app.post_subscription(body.into()).await;

    // Assert
}

#[actix_rt::test]
async fn subscribe_sends_a_confirmation_email_with_a_link() {
    // Arrange
    let app = spawn_app().await;
    let body = "name=spacedaddy&email=space_daddy%40test.com";

    Mock::given(path("/email"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&app.email_server)
        .await;

    // Act
    app.post_subscription(body.into()).await;

    // Assert
    // Get the first intercepted request -- this functionality is posisble with MockeServer
    let email_request = &app.email_server.received_requests().await.unwrap()[0];
    // Parse the body as JSON, starting from raw bytes
    let body: serde_json::Value = serde_json::from_slice(&email_request.body).unwrap();

    // Extract the link from one of the request fields
    let get_link = |s: &str| {
        let links: Vec<_> = linkify::LinkFinder::new()
            .links(s)
            .filter(|l| *l.kind() == linkify::LinkKind::Url)
            .collect();
        assert_eq!(links.len(), 1);
        links[0].as_str().to_owned()
    };

    let html_link = get_link(&body["HtmlBody"].as_str().unwrap());
    let text_link = get_link(&body["TextBody"].as_str().unwrap());
    assert_eq!(html_link, text_link);
}

#[actix_rt::test]
async fn subscribe_returns_a_400_when_data_is_missing() {
    // Arrange
    let test_app = spawn_app().await;
    let test_cases = vec![
        ("name=space%20daddy", "missing the email"),
        ("email=space_daddy%40test.com", "missing the name"),
        ("", "missing both name and email"),
    ];

    // Act
    for (invalid_body, error_message) in test_cases {
        let response = test_app.post_subscription(invalid_body.into()).await;

        // Assert
        assert_eq!(
            400,
            response.status().as_u16(),
            "The API did not fail with a 400 Bad Request when the payload was {}.",
            error_message
        )
    }
}

#[actix_rt::test]
async fn subscribe_returns_a_400_when_fields_are_present_but_invalid() {
    // Arrange
    let test_app = spawn_app().await;
    let test_cases = vec![
        ("name=&email=space_daddy%40example.com", "empty name"),
        ("name=Space&email=", "empty email"),
        ("name=Space&email=not-an-email", "invalid email"),
    ];

    for (body, description) in test_cases {
        // Act
        let response = test_app.post_subscription(body.into()).await;

        // Assert
        assert_eq!(
            400,
            response.status().as_u16(),
            "The API did not return a 400 Bad Request when the payload was {}.",
            description
        )
    }
}
