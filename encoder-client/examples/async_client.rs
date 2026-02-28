use encoder_client::AsyncEncoderClient;
use std::env;
use std::time::Duration;
use tokio::time;

#[tokio::main]
async fn main() {
    // Load environment variables from .env file (if present)
    dotenvy::dotenv().ok();

    let target_port = env::var("PICO_ENCODER_UART").unwrap_or_else(|_| {
        println!("Warning: PICO_ENCODER_UART not set. Defaulting to /dev/ttyACM0");
        "/dev/ttyACM0".to_string()
    });

    println!("Starting AsyncEncoderClient on port {}", target_port);

    let client = match AsyncEncoderClient::spawn(&target_port) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Failed to open serial port {}: {}", target_port, e);
            std::process::exit(1);
        }
    };

    println!("Connected! Listening for encoder counts asynchronously. Press Ctrl+C to exit.");

    let mut interval = time::interval(Duration::from_millis(100));
    loop {
        interval.tick().await;
        let seq = client.get_sequence();
        let counts = client.get_counts();
        println!("Sequence: {:>5} | Counts: {:?}", seq, counts);
    }
}
