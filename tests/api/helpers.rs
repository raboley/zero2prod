use secrecy::Secret;
use sqlx::types::Uuid;
use sqlx::{Connection, Executor, PgConnection, PgPool};
use std::{net::TcpListener, sync::LazyLock};
use zero2prod::configuration::{get_configuration, DatabaseSettings};
use zero2prod::email_client::EmailClient;
use zero2prod::telemetry::{get_subscriber, init_subscriber};

pub struct TestApp {
    pub address: String,
    pub db_pool: PgPool,
}

static TRACING: LazyLock<()> = LazyLock::new(|| {
    let default_filter_level = "debug".into();
    let subscriber_name = "test".to_string();

    if std::env::var("TEST_LOG").is_ok() {
        let subscriber = get_subscriber(subscriber_name, default_filter_level, std::io::stdout);
        init_subscriber(subscriber);
    } else {
        let subscriber = get_subscriber(subscriber_name, default_filter_level, std::io::sink);
        init_subscriber(subscriber);
    }
});

pub async fn spawn_app() -> TestApp {
    LazyLock::force(&TRACING);

    // port 0 is a special os port that will scan randomly for an available port.
    let listener = TcpListener::bind("127.0.0.1:0").expect("Failed to bind to random port");
    let port = listener.local_addr().unwrap().port();

    let mut configuration = get_configuration().expect("Failed to read configuration.");
    configuration.database.database_name = Uuid::new_v4().to_string();

    let connection_pool = configure_database(&configuration.database).await;
    let sender_email = configuration
        .email_client
        .sender()
        .expect("Invalid email client for sender");
    let base_url = configuration
        .email_client
        .base_url()
        .expect("Invalid base url client for sender");
    let authorization_token = configuration.email_client.authorization_token;
    let timeout = std::time::Duration::from_millis(configuration.email_client.timeout_milliseconds);
    let email_client = EmailClient::new(base_url, sender_email, authorization_token, timeout);

    let server = zero2prod::startup::run(listener, connection_pool.clone(), email_client)
        .expect("Failed to bind to address");
    let _ = tokio::spawn(server);
    let address = format!("http://127.0.0.1:{}", port);
    TestApp {
        address,
        db_pool: connection_pool,
    }
}

async fn configure_database(config: &DatabaseSettings) -> PgPool {
    // Create Database
    let maintenance_settings = DatabaseSettings {
        database_name: "postgres".to_string(),
        username: "postgres".to_string(),
        password: Secret::new("password".to_string()),
        ..config.clone()
    };

    let mut connection = PgConnection::connect_with(&maintenance_settings.connect_options())
        .await
        .expect("Failed to connect to Postgres");
    connection
        .execute(format!(r#"CREATE DATABASE "{}";"#, config.database_name).as_str())
        .await
        .expect("Failed to create database");

    // Migrate database
    let connection_pool = PgPool::connect_with(config.connect_options())
        .await
        .expect("Failed to connect to Postgres.");
    sqlx::migrate!("./migrations")
        .run(&connection_pool)
        .await
        .expect("Failed to migrate the database");

    connection_pool
}
