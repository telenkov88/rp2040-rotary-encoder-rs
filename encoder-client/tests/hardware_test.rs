use encoder_client::{AsyncEncoderClient, EncoderClient};
use std::env;
use std::sync::Mutex;
use std::thread;
use std::time::Duration;

static SERIAL_PORT_LOCK: Mutex<()> = Mutex::new(());

#[test]
#[ignore = "Requires RP2040 hardware plugged in to the host machine"]
fn test_hardware_connection_and_sampling() {
    // Load environment variables from .env file
    dotenvy::dotenv().ok();

    let _lock = SERIAL_PORT_LOCK.lock().unwrap();

    let target_port = env::var("PICO_ENCODER_UART").expect(
        "PICO_ENCODER_UART environment variable must be set (e.g. in .env file) to run this test.",
    );

    let client_result = EncoderClient::spawn(&target_port);

    // Check if the port even exists/opens. The test should FAIL if it doesn't open.
    let client = client_result.expect(&format!("Failed to open serial port: {}", target_port));

    // Sleep for 1.5 seconds to let the RP2040 emit data
    // and the thread reader to sample and parse the highest sequence.
    println!("Waiting for sequences...");
    thread::sleep(Duration::from_millis(1500));

    // After 1500ms, the sample sequence should definitely be greater than 0
    let seq = client.get_sequence();
    assert!(
        seq > 0,
        "Failed to read standard sequence count from board stream."
    );

    let counts = client.get_counts();
    println!(
        "Successfully captured real sequence {}: counts {:?}",
        seq, counts
    );

    // We expect some valid array dimension to come out
    assert_eq!(counts.len(), 8);
}

#[tokio::test]
#[ignore = "Requires RP2040 hardware plugged in to the host machine"]
async fn test_async_hardware_connection_and_sampling() {
    // Load environment variables from .env file
    dotenvy::dotenv().ok();

    let _lock = SERIAL_PORT_LOCK.lock().unwrap();

    let target_port = env::var("PICO_ENCODER_UART").expect(
        "PICO_ENCODER_UART environment variable must be set (e.g. in .env file) to run this test.",
    );

    let client_result = AsyncEncoderClient::spawn(&target_port);

    // Check if the port even exists/opens. The test should FAIL if it doesn't open.
    let client = client_result.expect(&format!("Failed to open serial port: {}", target_port));

    // Sleep for 1.5 seconds to let the RP2040 emit data
    // and the tokio task to sample and parse the highest sequence.
    println!("Waiting for sequences asynchronously...");
    tokio::time::sleep(Duration::from_millis(1500)).await;

    // After 1500ms, the sample sequence should definitely be greater than 0
    let seq = client.get_sequence();
    assert!(
        seq > 0,
        "Failed to read standard sequence count from board stream (async)."
    );

    let counts = client.get_counts();
    println!(
        "Successfully captured real sequence {} asynchronously: counts {:?}",
        seq, counts
    );

    // We expect some valid array dimension to come out
    assert_eq!(counts.len(), 8);
}
