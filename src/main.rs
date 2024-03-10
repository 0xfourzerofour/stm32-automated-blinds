#![deny(unsafe_code)]
#![deny(warnings)]
#![no_main]
#![no_std]

use panic_rtt_target as _;
use rtic::app;
use rtt_target::rtt_init_print;
use stm32f4xx_hal::gpio::{Edge, Input, Output, PushPull, PA0, PA5, PA6};
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
        motor_clockwise: PA5<Output<PushPull>>,
        motor_counter_clockwise: PA6<Output<PushPull>>,
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

        let pwm = dp.TIM1.pwm_hz(channels, 20.kHz(), &clocks).split();
        let (mut ch1, _ch2) = pwm;
        let max_duty = ch1.get_max_duty();
        ch1.set_duty(max_duty);
        ch1.enable();

        let mut motor_clockwise = gpioa.pa5.into_push_pull_output();
        motor_clockwise.set_low();

        let mut motor_counter_clockwise = gpioa.pa6.into_push_pull_output();
        motor_counter_clockwise.set_low();

        (
            Shared {},
            Local {
                receiver,
                direction: Direction::Up,
                motor_clockwise,
                motor_counter_clockwise,
            },
        )
    }

    #[task(binds = EXTI0, local = [receiver, direction, motor_clockwise, motor_counter_clockwise])]
    fn receive_signal(ctx: receive_signal::Context) {
        ctx.local.receiver.clear_interrupt_pending_bit();
        match ctx.local.direction {
            Direction::Up => {
                *ctx.local.direction = Direction::Down;
                ctx.local.motor_counter_clockwise.set_low();
                ctx.local.motor_clockwise.set_high();
            }
            Direction::Down => {
                *ctx.local.direction = Direction::Up;
                ctx.local.motor_clockwise.set_low();
                ctx.local.motor_counter_clockwise.set_high();
            }
        }
    }
}
