# RP2040 Rotary Encoder

This project implements an embedded solution that utilizes an RP2040 microcontroller to continuously track 8 rotary encoders at high frequency via interrupts (utilizing Core 1 exclusively for sampling) while reliably transmitting the accumulated values back to the host computer at 100Hz over UART text strings from Core 0.

## Project Structure

- `encoder-firmware`: The embedded `no_std` `embassy-rp` application that runs on the actual RP2040 microcontroller. It maintains atomic hardware counts and spits them out as ASCII (`42:-100,5,-420,0,1,0,0,0\n`) every 10 milliseconds.
- `encoder-client`: A ready-to-use thread-safe Rust library exposing an `Arc<RwLock<[i32; 8]>>` mapped in real-time over the host's serial connection context, permitting trivially simple polling inside external ecosystem software setups (like motor drivers, etc.).
- `shared`: Internal protocol mappings defining packets and limits intended for bidirectional sharing.

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
