// #![deny(unsafe_code)]
#![no_main]
#![no_std]
#![allow(unused_must_use)]

use panic_halt as _;
use rtt_target::{rtt_init_print, rprintln};
use stm32f4xx_hal::{
    gpio::{self, Output, PushPull},
    pac::TIM2,
    pac::TIM1,
    prelude::*,
    timer::{self, Event},
};
use ultrasonic_sensor::UltrasonicSensor;  // Importation de ton module

#[rtic::app(device = stm32f4xx_hal::pac, peripherals = true)]
mod app {
    use super::*;

    #[shared]
    struct Shared {}

    #[local]
    struct Local {
        sensor: UltrasonicSensor<gpio::PC2<Output<PushPull>>, gpio::PC3<Input>, timer::DelayUs<TIM1>>,
        timer: timer::CounterUs<TIM2>,
    }

    #[init]
    fn init(mut ctx: init::Context) -> (Shared, Local, init::Monotonics) {
        rtt_init_print!();

        let mut dp = ctx.device;

        let gpioc = dp.GPIOC.split();
        let trigger_pin = gpioc.pc2.into_push_pull_output();
        let echo_pin = gpioc.pc3.into_pull_down_input();

        let rcc = dp.RCC.constrain();
        let clocks = rcc
            .cfgr
            .use_hse(8.MHz())
            .sysclk(168.MHz())
            .freeze();

        let delay = dp.TIM1.delay_us(&clocks);
        let mut timer = dp.TIM2.counter_us(&clocks);
        timer.start(100.millis()).unwrap();
        timer.listen(Event::Update);

        let sensor = UltrasonicSensor::new(trigger_pin, echo_pin, delay);

        (
            Shared {},
            Local {
                sensor,
                timer,
            },
            init::Monotonics(),
        )
    }

    #[task(binds = TIM2, local=[sensor, timer])]
    fn read_sensor(mut ctx: read_sensor::Context) {
        rprintln!("Task : Read sensor");

        if let Some(distance) = ctx.local.sensor.measure_distance(ctx.local.timer) {
            rprintln!("Measured distance: {}cm", distance);
        } else {
            rprintln!("No distance measured");
        }

        ctx.local.timer.start(100.millis()).unwrap();
    }
}
