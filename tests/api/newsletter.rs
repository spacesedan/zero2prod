use crate::helpers::{spawn_app, ConfirmationLinks, TestApp};
use wiremock::{
    matchers::{any, method, path},
    Mock, ResponseTemplate,
};

async fn create_unconfirmed_subscriber(app: &TestApp) -> ConfirmationLinks {
    let body = "name=spacedaddy&email=space_daddy%40test.com";

    let _mock_gaurd = Mock::given(path("/email"))
        .and(method(reqwest::Method::POST))
        .respond_with(ResponseTemplate::new(reqwest::StatusCode::OK))
        .named("Create unconfirmed subscriber")
        .expect(1)
        .mount_as_scoped(&app.email_server)
        .await;

    app.post_subscription(body.into())
        .await
        .error_for_status()
        .unwrap();

    let email_request = &app
        .email_server
        .received_requests()
        .await
        .unwrap()
        .pop()
        .unwrap();

    app.get_confirmation_links(&email_request)
}

async fn create_confirmed_subscriber(app: &TestApp) {
    let confirmation_link = create_unconfirmed_subscriber(app).await.html;

    reqwest::get(confirmation_link)
        .await
        .unwrap()
        .error_for_status()
        .unwrap();
}

#[actix_rt::test]
async fn newsletters_are_not_delivered_to_unconfirmed_subscribers() {
    // Arrange
    let app = spawn_app().await;
    create_unconfirmed_subscriber(&app).await;

    Mock::given(any())
        .respond_with(ResponseTemplate::new(200))
        .expect(0)
        .mount(&app.email_server)
        .await;

    // Act
    let newletter_request_body = serde_json::json!({
        "title": "Newsletter Title",
        "content" : {
            "text": "Newsletter body as plain text",
            "html": "<p>Newsletter body as HTML</p>"
        }
    });
    let response = app.post_newsletters(newletter_request_body).await;

    // Assert
    assert_eq!(response.status().as_u16(), reqwest::StatusCode::OK);
}

#[actix_rt::test]
pub async fn newsletters_are_delivered_to_confirmed_subscribers() {
    // Arrange
    let app = spawn_app().await;
    create_confirmed_subscriber(&app).await;

    Mock::given(path("/email"))
        .and(method(reqwest::Method::POST))
        .respond_with(ResponseTemplate::new(reqwest::StatusCode::OK))
        .expect(1)
        .mount(&app.email_server)
        .await;

    // Act
    let newletter_request_body = serde_json::json!({
        "title": "Newsletter Title",
        "content" : {
            "text": "Newsletter body as plain text",
            "html": "<p>Newsletter body as HTML</p>"
        }
    });
    let response = app.post_newsletters(newletter_request_body).await;

    // Assert
    assert_eq!(response.status().as_u16(), reqwest::StatusCode::OK)
}

#[tokio::test]
async fn newsletters_returns_400_for_invalid_data() {
    // Arrange
    let app = spawn_app().await;
    let test_cases = vec![
        (
            serde_json::json!(
            {
                "content":{
                    "text": "Newsletter body as plain text",
                    "html": "<p>Newsletter body as HTML</p>"
                }
            }
            ),
            "missing title",
        ),
        (
            serde_json::json!(
            {
            "title": "Newsletter"
            }

                ),
            "missing content",
        ),
    ];

    // Act
    for (invalid_body, error_message) in test_cases {
        let response = app.post_newsletters(invalid_body).await;
        // Assert
        assert_eq!(
            reqwest::StatusCode::BAD_REQUEST,
            response.status().as_u16(),
            "The API did not fail with 400 Bad Request when the payload was {}",
            error_message
        );
    }
}
