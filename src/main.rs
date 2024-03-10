#![deny(unsafe_code)]
#![deny(warnings)]
#![no_main]
#![no_std]

use panic_rtt_target as _;
use rtic::app;
use rtt_target::rtt_init_print;
use stm32f4xx_hal::gpio::{Edge, Input, PA0};
use stm32f4xx_hal::prelude::*;
use stm32f4xx_hal::timer::{Channel1, Channel2};

#[app(device = stm32f4xx_hal::pac, peripherals = true)]
mod app {

    use super::*;

    #[derive(Debug)]
    enum Direction {
        Up,
        Down,
    }

    #[shared]
    struct Shared {}

    #[local]
    struct Local {
        receiver: PA0<Input>,
        direction: Direction,
    }

    #[init]
    fn init(cx: init::Context) -> (Shared, Local) {
        rtt_init_print!();
        let mut dp = cx.device;
        let mut syscfg = dp.SYSCFG.constrain();
        let rcc = dp.RCC.constrain();
        let clocks = rcc.cfgr.freeze();

        let gpioa = dp.GPIOA.split();
        let mut receiver = gpioa.pa0.into_pull_up_input();

        receiver.make_interrupt_source(&mut syscfg);
        receiver.trigger_on_edge(&mut dp.EXTI, Edge::Falling);
        receiver.enable_interrupt(&mut dp.EXTI);

        let channels = (Channel1::new(gpioa.pa8), Channel2::new(gpioa.pa9));

        let pwm = dp.TIM1.pwm_hz(channels, 36.kHz(), &clocks).split();
        let (mut ch1, _ch2) = pwm;
        let max_duty = ch1.get_max_duty();
        ch1.set_duty(max_duty / 2);
        ch1.enable();

        (
            Shared {},
            Local {
                receiver,
                direction: Direction::Up,
            },
        )
    }

    #[task(binds = EXTI0, local = [receiver, direction])]
    fn receive_signal(ctx: receive_signal::Context) {
        ctx.local.receiver.clear_interrupt_pending_bit();
        match ctx.local.direction {
            Direction::Up => {
                *ctx.local.direction = Direction::Down;
            }
            Direction::Down => {
                *ctx.local.direction = Direction::Up;
            }
        }
    }
}
