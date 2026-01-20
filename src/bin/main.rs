#![no_std]
#![no_main]
#![deny(
    clippy::mem_forget,
    reason = "mem::forget is generally not safe to do with esp_hal types, especially those \
    holding buffers for the duration of a data transfer."
)]
#![deny(clippy::large_stack_frames)]

use esp_backtrace as _;
use embedded_dht_rs::dht22::Dht22;
use embassy_executor::Spawner;
use esp_hal::{
    delay::Delay,
    gpio::{Input, DriveMode, Flex, OutputConfig, Pull, InputConfig},
    clock::CpuClock
};
use embassy_time::{Duration, Timer};
use log::info;

// This creates a default app-descriptor required by the esp-idf bootloader.
// For more information see: <https://docs.espressif.com/projects/esp-idf/en/stable/esp32/api-reference/system/app_image_format.html#application-description>
esp_bootloader_esp_idf::esp_app_desc!();

#[allow(
    clippy::large_stack_frames,
    reason = "it's not unusual to allocate larger buffers etc. in main"
)]
fn c_to_f(celcius: f32) -> f32 {
    celcius * 9.0/5.0 + 32.0
}

#[embassy_executor::task]
async fn run() {
    loop {
        esp_println::println!("Hello world from embassy!");
        Timer::after(Duration::from_millis(1_000)).await;
    }
}

#[esp_rtos::main]
async fn main(spawner: Spawner) -> ! {
    // generator version: 1.1.0

    esp_println::logger::init_logger_from_env();

    let config = esp_hal::Config::default().with_cpu_clock(CpuClock::max());
    let peripherals = esp_hal::init(config);
    let delay = Delay::new();

    // DHT sensor is hooked into 3V power, GND, and GPIO0
    // Learning that some GPIO pins are unsafe to use on a board
    // DHT22 sensor setup (GPIO0), DHT library needed.
    let mut dht22_pin = Flex::new(peripherals.GPIO0);
    dht22_pin.apply_output_config(
        &OutputConfig::default()
            .with_drive_mode(DriveMode::OpenDrain)
            .with_pull(Pull::None),
    );
    dht22_pin.set_output_enable(true);
    dht22_pin.set_input_enable(true);
    dht22_pin.set_high();

    let mut dht22 = Dht22::new(dht22_pin, Delay::new());

    // HC-SR501 PIR Sensor setup (GPIO1)
    let mut pir_sensor = Input::new(peripherals.GPIO1, InputConfig::default());
    spawner.spawn(run()).ok();

    loop {
        pir_sensor.wait_for_high().await;
        info!("Motion detected!");
        // Can only read DHT every 2s
        delay.delay_millis(2000);
        match dht22.read() {
            Ok(sensor_reading) => {
                esp_println::println!{"DHT Sensor: Temp {}, Humidity {} ", 
                    c_to_f(sensor_reading.temperature),
                    sensor_reading.humidity 
                };
            },
            Err(e) => {
                esp_println::dbg!("An error occurred: {}", e);
            }
        }
    }
    // for inspiration have a look at the examples at https://github.com/esp-rs/esp-hal/tree/esp-hal-v~1.0/examples
}
