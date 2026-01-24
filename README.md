# ESP32 Sensor Node: DHT22 & HC-SR501

A Rust-based embedded project running on an ESP32 that integrates a PIR motion sensor and a DHT22 temperature/humidity sensor. This project demonstrates the "Event-Driven" architecture in `no_std` Rust, moving heavy sensor logic out of Interrupt Service Routines (ISR) to maintain system stability.

## 🏗 Hardware Configuration

* **MCU:** ESP32 (via `esp-hal`)
* **DHT22 (Temperature/Humidity):** * Connected to **GPIO0**.
* Configured as `OpenDrain` with a `Flex` pin to handle the bi-directional 1-wire communication required by the DHT protocol.


* **HC-SR501 (PIR Motion Sensor):** * Connected to **GPIO1**.
* Configured to trigger a hardware interrupt on the **Rising Edge**.


## 🚦 Logic Flow

1. **Interrupt:** When the PIR sensor detects motion, the hardware triggers the `handler` function (marked with `#[ram]` for high-performance execution).
2. **Flagging:** The ISR safely borrows the `MOTION_DETECTED` boolean via a critical section and sets it to `true`. It then clears the hardware interrupt bit.
3. **Processing:** The `main` loop constantly checks the `MOTION_DETECTED` flag. When it sees a `true` value:
* It triggers a read of the DHT22 sensor.
* Converts the Celsius reading to Fahrenheit.
* Logs the data to the console using `esp_println`.
* Resets the flag to `false`, waiting for the next motion event.



## ⚠️ Lessons Learned & Safety

* **Pin Safety:** Learned that not all GPIO pins are created equal; some have specific strapping requirements or are unsafe for general use during boot.
* **Memory Safety:** Implemented `#![deny(clippy::mem_forget)]` to ensure `esp-hal` types—which often manage DMA or hardware buffers—are dropped correctly to avoid hardware state leaks.
* **Stack Management:** Restricted large stack frames to the main entry point to prevent stack overflows in the restricted embedded RAM environment.

## 🚀 Next Steps

* [ ] Implement a web server to expose temperature/humidity data over Wi-Fi.
* [ ] Add a timeout or "cool down" period for the motion sensor to prevent rapid re-triggering.
* [ ] Fine-tune the DHT22 error handling to recover more gracefully from timing-related bus errors.

---

**Would you like me to help you implement a "debounce" timer so the motion sensor doesn't trigger the DHT read too many times in a row?**