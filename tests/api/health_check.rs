use crate::helpers::spawn_app;

#[tokio::test]
async fn health_check_works() {
    // arrange
    let address = spawn_app().await.address;
    let client = reqwest::Client::new();

    // act
    let response = client
        .get(&format!("{}/health_check", address))
        .send()
        .await
        .expect("Failed to execute request.");

    assert!(response.status().is_success());
    assert_eq!(Some(0), response.content_length());
}
