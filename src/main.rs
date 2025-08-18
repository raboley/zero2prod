use std::net::TcpListener;

use sqlx::postgres::PgPoolOptions;
use zero2prod::configuration::get_configuration;
use zero2prod::email_client::EmailClient;
use zero2prod::startup::run;
use zero2prod::telemetry::{get_subscriber, init_subscriber};

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    let subscriber = get_subscriber("zero2prod".to_owned(), "info".to_owned(), std::io::stdout);
    init_subscriber(subscriber);

    let configuration = get_configuration().expect("Failed to read configuration.");
    let connection_pool =
        PgPoolOptions::new().connect_lazy_with(configuration.database.connect_options());
    let sender_email = configuration
        .email_client
        .sender()
        .expect("Invalid sender email address");
    let base_url = configuration
        .email_client
        .base_url()
        .expect("Invalid base address for email client");
    let timeout = std::time::Duration::from_millis(configuration.email_client.timeout_milliseconds);
    let email_client = EmailClient::new(
        base_url,
        sender_email,
        configuration.email_client.authorization_token,
        timeout,
    );

    let address = format!(
        "{}:{}",
        configuration.application.host, configuration.application.port
    );
    let listener = TcpListener::bind(address)?;
    run(listener, connection_pool, email_client)?.await
}
