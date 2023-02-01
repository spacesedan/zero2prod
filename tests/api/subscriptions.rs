use crate::helpers::spawn_app;

#[actix_rt::test]
async fn subscribe_returns_a_200_for_valid_data() {
    // Arrange
    let test_app = spawn_app().await;
    let body = "name=spacedaddy&email=space_daddy%40test.com";

    // Act
    let response = test_app.post_subscription(body.into()).await;

    // Assert
    assert_eq!(200, response.status().as_u16());
    // assert_matches!(200, response_status);

    let saved = sqlx::query!("SELECT email, name FROM subscriptions",)
        .fetch_one(&test_app.db_pool)
        .await
        .expect("Failed to fetch saved subscription");

    assert_eq!(saved.email, "space_daddy@test.com");
    assert_eq!(saved.name, "spacedaddy")
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
