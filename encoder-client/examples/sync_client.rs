use encoder_client::EncoderClient;
use std::env;
use std::thread;
use std::time::Duration;

fn main() {
    // Load environment variables from .env file (if present)
    dotenvy::dotenv().ok();

    let target_port = env::var("PICO_ENCODER_UART").unwrap_or_else(|_| {
        println!("Warning: PICO_ENCODER_UART not set. Defaulting to /dev/ttyACM0");
        "/dev/ttyACM0".to_string()
    });

    println!("Starting SyncEncoderClient on port {}", target_port);

    let client = match EncoderClient::spawn(&target_port) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Failed to open serial port {}: {}", target_port, e);
            std::process::exit(1);
        }
    };

    println!("Connected! Listening for encoder counts. Press Ctrl+C to exit.");

    loop {
        let seq = client.get_sequence();
        let counts = client.get_counts();
        println!("Sequence: {:>5} | Counts: {:?}", seq, counts);
        thread::sleep(Duration::from_millis(100));
    }
}
