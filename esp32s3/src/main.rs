#![no_std]
#![no_main]
#![feature(impl_trait_in_assoc_type)]

use embassy_executor::Spawner;
extern crate alloc;

use esp_alloc as _;
use esp_backtrace as _;
use esp_hal::psram::{self, PsramSize};
use esp_hal::rtc_cntl::Rtc;
use esp_hal::timer::{AnyTimer, timg::TimerGroup};
use esp_println::println;
use esp_storage::FlashStorage;

use embedded_storage::{ReadStorage, Storage};

#[esp_rtos::main]
async fn main(spawner: Spawner) {
    // init CPU
    let config = esp_hal::Config::default().with_cpu_clock(esp_hal::clock::CpuClock::max());

    let peripherals = esp_hal::init(config);
    let rtc = Rtc::new(peripherals.LPWR);

    let timg0 = TimerGroup::new(peripherals.TIMG0);

    esp_rtos::start(timg0.timer0);

    esp_alloc::psram_allocator!(peripherals.PSRAM, psram);

    let mut storage;
    unsafe {
        let peripherals: esp_hal::peripherals::Peripherals =
            esp_hal::peripherals::Peripherals::steal();
        storage = FlashStorage::new(peripherals.FLASH);
    }
    println!("Flash size = {} MB", storage.capacity() / 1_000_000);

    let mut filesystem: little_weirdo_filesystem::WeirdoFileSystem<FlashStorage> =
        little_weirdo_filesystem::WeirdoFileSystem::new(storage, 0x00220000, 0x100000);

}
