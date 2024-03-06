#![deny(unsafe_code)]
#![deny(warnings)]
#![no_main]
#![no_std]

use panic_rtt_target as _;
use rtic::app;
use rtic_monotonics::create_systick_token;
use rtic_monotonics::systick::Systick;
use rtt_target::{rprintln, rtt_init_print};
use stm32f4xx_hal::gpio::{Output, PushPull, PA5};
use stm32f4xx_hal::prelude::*;

#[app(device = stm32f4xx_hal::pac, peripherals = true, dispatchers = [SPI1])]
mod app {

    use super::*;

    #[shared]
    struct Shared {}

    #[local]
    struct Local {
        led: PA5<Output<PushPull>>,
        state: bool,
    }

    #[init]
    fn init(cx: init::Context) -> (Shared, Local) {
        rtt_init_print!();
        // Setup clocks
        let rcc = cx.device.RCC.constrain();

        let systick_mono_token = create_systick_token!();
        Systick::start(cx.core.SYST, 36_000_000, systick_mono_token);
        let _clocks = rcc.cfgr.sysclk(36.MHz()).freeze();

        // Setup LED
        let gpioa = cx.device.GPIOA.split();
        let mut led = gpioa.pa5.into_push_pull_output();
        led.set_high();

        // Schedule the blinking task
        blink::spawn().ok();

        (Shared {}, Local { led, state: false })
    }

    #[task(local = [led, state])]
    async fn blink(cx: blink::Context) {
        loop {
            if *cx.local.state {
                cx.local.led.set_high();
                *cx.local.state = false;
            } else {
                cx.local.led.set_low();
                *cx.local.state = true;
            }

            rprintln!("Blinky fuck");
            Systick::delay(1000.millis()).await;
        }
    }
}
