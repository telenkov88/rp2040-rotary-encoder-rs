use encoder_client::EncoderClient;
use std::thread;
use std::time::Duration;

// We use the same known USB port from the original firmwares UART e2e test target.
// It may vary slightly on host machines, but this is the standard macOS MKS servo dummy target.
const TARGET_PORT: &str = "/tmp/ttyEncoder0";

#[test]
#[ignore = "Requires RP2040 hardware plugged in to the host machine"]
fn test_hardware_connection_and_sampling() {
    let client_result = EncoderClient::spawn(TARGET_PORT);

    // Check if the port even exists/opens.
    if client_result.is_err() {
        println!(
            "Skipping hardware test, port {} not found or accessible.",
            TARGET_PORT
        );
        return;
    }

    let client = client_result.unwrap();

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
