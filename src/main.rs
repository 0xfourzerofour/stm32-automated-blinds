#![deny(unsafe_code)]
#![deny(warnings)]
#![no_main]
#![no_std]

use panic_rtt_target as _;
use rtic::app;
use rtt_target::{rprintln, rtt_init_print};
use stm32f4xx_hal::gpio::{Edge, Input, PA0};
use stm32f4xx_hal::prelude::*;

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

        let gpioa = dp.GPIOA.split();
        let mut receiver = gpioa.pa0.into_pull_up_input();

        receiver.make_interrupt_source(&mut syscfg);
        receiver.trigger_on_edge(&mut dp.EXTI, Edge::Falling);
        receiver.enable_interrupt(&mut dp.EXTI);

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
        rprintln!("{:?}", ctx.local.direction);
        match ctx.local.direction {
            Direction::Up => *ctx.local.direction = Direction::Down,
            Direction::Down => *ctx.local.direction = Direction::Up,
        }
    }
}
