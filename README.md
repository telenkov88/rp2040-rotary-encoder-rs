# RP2040 Rotary Encoder

This project implements an embedded solution that utilizes an RP2040 microcontroller to continuously track 8 rotary encoders at high frequency via interrupts (utilizing Core 1 exclusively for sampling) while reliably transmitting the accumulated values back to the host computer at 100Hz over UART text strings from Core 0.

## Project Structure

- `encoder-firmware`: The embedded `no_std` `embassy-rp` application that runs on the actual RP2040 microcontroller. It maintains atomic hardware counts and spits them out as ASCII (`42:-100,5,-420,0,1,0,0,0\n`) every 10 milliseconds.
- `encoder-client`: A ready-to-use thread-safe Rust library exposing an `Arc<RwLock<[i32; 8]>>` mapped in real-time over the host's serial connection context, permitting trivially simple polling inside external ecosystem software setups (like motor drivers, etc.).
- `shared`: Internal protocol mappings defining packets and limits intended for bidirectional sharing.

## Hardware PIN Mapping

The RP2040 firmware expects the following pin connections:

| Component      | RP2040 Pin | Function |
| -------------- | ---------- | -------- |
| **UART TX**    | PIN 16     | Data to Host (115200 baud) |
| **UART RX**    | PIN 17     | Data from Host (Currently just for basic ping/response or future use) |
| **Status LED** | PIN 25     | PWM Activity Indicator |
| **Encoder 0**  | PIN 2, 3   | A, B phases |
| **Encoder 1**  | PIN 4, 5   | A, B phases |
| **Encoder 2**  | PIN 6, 7   | A, B phases |
| **Encoder 3**  | PIN 8, 9   | A, B phases |
| **Encoder 4**  | PIN 10, 11 | A, B phases |
| **Encoder 5**  | PIN 12, 13 | A, B phases |
| **Encoder 6**  | PIN 14, 15 | A, B phases |
| **Encoder 7**  | PIN 27, 26 | A, B phases |

*Note: All encoder pins are configured with internal pull-up resistors.*

## Using `encoder-client`

To parse variables locally on a linux/macOS host with a serial connection, add `encoder-client` to your Cargo dependencies and initialize the daemon:

```rust
use encoder_client::EncoderClient;

// Spawns a background thread immediately tracking the stream
let client = EncoderClient::spawn("/dev/cu.usbmodem1101")
    .expect("Failed to initialize UART client");

// Trivial real-time polling from application logic loops
let sensor_counts: [i32; 8] = client.get_counts();
println!("Latest Encoders: {:?}", sensor_counts);
```

## Hardware Testing

To run the hardware connection tests on the client host, you must provide the target UART path via an environment variable. You can specify it manually or simply create a `.env` file at the root of the project:

```bash
# Example .env file content
PICO_ENCODER_UART=/dev/ttyEncoder0
```

Then run the tests:

```bash
make test-hardware
```
