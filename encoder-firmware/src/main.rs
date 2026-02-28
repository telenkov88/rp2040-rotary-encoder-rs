//! RP2040 firmware that continuously tracks 8 rotary encoders and streams their accumulated values over UART.

#![no_std]
#![no_main]

use defmt::info;
use embassy_executor::{Executor, Spawner};
use embassy_rp::gpio::{Input, Pull};
use embassy_rp::pwm::{Config as PwmConfig, Pwm};
use embassy_rp::multicore::{spawn_core1, Stack};
use portable_atomic::{AtomicI32, Ordering};
use rotary_encoder_embedded::{Direction, InitalizeMode, RotaryEncoder};
use static_cell::StaticCell;

use embassy_rp::bind_interrupts;
use embassy_rp::peripherals::UART0;
use embassy_rp::uart::{BufferedInterruptHandler, BufferedUart, BufferedUartRx, Config};
use embedded_io_async::{Read, Write};

use encoder_protocol::{
    serialize_packet, Packet, SensorDataPacket, BUFFER_SIZE, MAX_ENCODERS,
};
use {defmt_rtt as _, panic_probe as _};

bind_interrupts!(struct Irqs {
    UART0_IRQ => BufferedInterruptHandler<UART0>;
});

static CORE1_STACK: StaticCell<Stack<4096>> = StaticCell::new();
static EXECUTOR1: StaticCell<Executor> = StaticCell::new();

struct Encoders {
    encoders: [RotaryEncoder<InitalizeMode, Input<'static>, Input<'static>>; MAX_ENCODERS],
}

static ENCODER_COUNTS: [AtomicI32; MAX_ENCODERS] = [const { AtomicI32::new(0) }; MAX_ENCODERS];

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let p = embassy_rp::init(Default::default());

    let mut pwm_config: PwmConfig = Default::default();
    pwm_config.top = 20000;
    let max_brightness = pwm_config.top / 40;
    pwm_config.compare_b = 0;
    let mut led_pwm = Pwm::new_output_b(p.PWM_SLICE4, p.PIN_25, pwm_config.clone());

    let (tx_pin, rx_pin, uart) = (p.PIN_16, p.PIN_17, p.UART0);

    static TX_BUF: StaticCell<[u8; BUFFER_SIZE]> = StaticCell::new();
    let tx_buf = &mut TX_BUF.init([0; BUFFER_SIZE])[..];
    static RX_BUF: StaticCell<[u8; BUFFER_SIZE]> = StaticCell::new();
    let rx_buf = &mut RX_BUF.init([0; BUFFER_SIZE])[..];
    let mut config = Config::default();
    config.baudrate = 115_200;
    
    let uart = BufferedUart::new(
        uart,
        tx_pin,
        rx_pin,
        Irqs,
        tx_buf,
        rx_buf,
        config,
    );
    let (mut tx, rx) = uart.split();

    spawner.must_spawn(reader(rx));

    let encoders = Encoders {
        encoders: [
            RotaryEncoder::new(Input::new(p.PIN_2, Pull::Up), Input::new(p.PIN_3, Pull::Up)),
            RotaryEncoder::new(Input::new(p.PIN_4, Pull::Up), Input::new(p.PIN_5, Pull::Up)),
            RotaryEncoder::new(Input::new(p.PIN_6, Pull::Up), Input::new(p.PIN_7, Pull::Up)),
            RotaryEncoder::new(Input::new(p.PIN_8, Pull::Up), Input::new(p.PIN_9, Pull::Up)),
            RotaryEncoder::new(
                Input::new(p.PIN_10, Pull::Up),
                Input::new(p.PIN_11, Pull::Up),
            ),
            RotaryEncoder::new(
                Input::new(p.PIN_12, Pull::Up),
                Input::new(p.PIN_13, Pull::Up),
            ),
            RotaryEncoder::new(
                Input::new(p.PIN_14, Pull::Up),
                Input::new(p.PIN_15, Pull::Up),
            ),
            RotaryEncoder::new(
                Input::new(p.PIN_27, Pull::Up),
                Input::new(p.PIN_26, Pull::Up),
            ),
        ],
    };

    spawn_core1(
        p.CORE1,
        CORE1_STACK.init(Stack::new()),
        move || {
            let executor1 = EXECUTOR1.init(Executor::new());
            executor1.run(|spawner| spawner.must_spawn(core1_task(encoders)));
        },
    );

    let mut sequence = 0u32;

    loop {
        embassy_time::Timer::after_millis(10).await;

        let cycle_tick = sequence % 200;
        if cycle_tick < 100 {
            pwm_config.compare_b = max_brightness;
        } else {
            pwm_config.compare_b = 0;
        }
        led_pwm.set_config(&pwm_config);

        let encoder_counts = ENCODER_COUNTS.each_ref().map(|c| c.load(Ordering::SeqCst));
        let sensor_data_packet = SensorDataPacket {
            seq: sequence,
            encoders: encoder_counts,
        };
        let packet = Packet::SensorData(sensor_data_packet);
        let buf = serialize_packet(&packet);

        if sequence % 10 == 0 {
            info!("TX Seq: {:?} Counts: {:?}", sequence, encoder_counts);
        }

        if let Err(_e) = tx.write_all(buf.as_bytes()).await {
            defmt::error!("UART write failed");
        }
        if let Err(_e) = tx.flush().await {
            defmt::error!("UART flush failed");
        }
        sequence += 1;
    }
}

/// Continuously samples all encoder inputs on Core 1 for atomic accumulation.
#[embassy_executor::task]
async fn core1_task(encoders: Encoders) {
    info!("Encoder samling started.");

    // Allow internal pull-ups to stabilize and pins to reach their default state
    // before taking the initial reading. This prevents spurious initial counts.
    embassy_time::Timer::after_millis(10).await;

    let mut encoders = encoders.encoders.map(|e| e.into_standard_mode());

    // The rotary-encoder-embedded crate's StandardMode initializes its internal
    // history buffer asymmetrically ([0xFF, 2]). If the starting pin state is (Low, Low),
    // the very first update() call interprets the history transition as an Anticlockwise movement.
    // To 'prime' the history buffer, we perform a few dummy reads and discard their results 
    // before we start accumulating real counts.
    for _ in 0..4 {
        for en in encoders.iter_mut() {
            en.update();
        }
    }

    loop {
        for (i, en) in encoders.iter_mut().enumerate() {
            match en.update() {
                Direction::Clockwise => {
                    ENCODER_COUNTS[i].fetch_add(1, Ordering::SeqCst);
                }
                Direction::Anticlockwise => {
                    ENCODER_COUNTS[i].fetch_sub(1, Ordering::SeqCst);
                }
                Direction::None => {}
            }
        }
    }
}

/// Reads data from the UART, currently only used to drain the receive buffer to prevent overflow.
#[embassy_executor::task]
async fn reader(mut rx: BufferedUartRx) {
    info!("Reading...");
    loop {
        let mut buf = [0; 1];
        if let Err(_e) = rx.read_exact(&mut buf).await {
            defmt::error!("UART read failed");
            embassy_time::Timer::after_millis(10).await;
            continue;
        }
        info!("RX {:?}", buf);
    }
}
