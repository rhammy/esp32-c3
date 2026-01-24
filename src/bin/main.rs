#![no_std]
#![no_main]
#![deny(
    clippy::mem_forget,
    reason = "mem::forget is generally not safe to do with esp_hal types, especially those \
    holding buffers for the duration of a data transfer."
)]
#![deny(clippy::large_stack_frames)]

use esp_backtrace as _;
use critical_section::Mutex;
use embedded_dht_rs::dht22::Dht22;
use core::cell::RefCell;
use esp_hal::{
    delay::Delay,
    gpio::{Input, DriveMode, Flex, OutputConfig, Pull, InputConfig, Io, Event},
    clock::CpuClock,
    ram,
    handler,
    main
};

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
// Static global variable so the ISR has access to the sensor, wrap it in a mutex.
static MOTION_SENSOR: Mutex<RefCell<Option<Input>>> = Mutex::new(RefCell::new(None));
static MOTION_DETECTED: Mutex<RefCell<bool>> = Mutex::new(RefCell::new(false));

#[main]
fn main() -> ! {
    // generator version: 1.1.0

    esp_println::logger::init_logger_from_env();

    let config = esp_hal::Config::default().with_cpu_clock(CpuClock::max());
    let peripherals = esp_hal::init(config);
    let mut io = Io::new(peripherals.IO_MUX);
    io.set_interrupt_handler(handler);

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
    critical_section::with(|cs| {
        pir_sensor.listen(Event::RisingEdge);
        MOTION_SENSOR.borrow_ref_mut(cs).replace(pir_sensor);
    });

    loop {
        // TODO: Web server logic?
        critical_section::with(|cs| {
            let mut triggered = MOTION_DETECTED.borrow_ref_mut(cs);
            if *triggered {
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
                *triggered = false;
            }
        })

    }
    // for inspiration have a look at the examples at https://github.com/esp-rs/esp-hal/tree/esp-hal-v~1.0/examples
}

#[handler]
#[ram]
fn handler() {
    esp_println::println!("GPIO Interrupt");
    if critical_section::with(|cs| {
        MOTION_SENSOR
            .borrow_ref_mut(cs)
            .as_mut()
            .unwrap()
            .is_interrupt_set()
    }) {
        esp_println::println!("Motion was the source of the interrupt");
        // Set the flag and clear the interrupt
        critical_section::with(|cs| {
            // Borrow a mutable reference, then dereference to change the value.
            *MOTION_DETECTED.borrow_ref_mut(cs) = true;
            MOTION_SENSOR
                .borrow_ref_mut(cs)
                .as_mut()
                .unwrap()
                .clear_interrupt()
            });
        } 
    else {
        esp_println::println!("Motion was not the source of the interrupt");
    }
}
