// #![deny(unsafe_code)]
#![no_main]
#![no_std]
#![allow(unused_must_use)]

// Halt on panic
use panic_halt as _;

#[rtic::app(device = stm32f4xx_hal::pac, peripherals = true)] // peripherals = true makes sure that the device handle/field is available for use later in our code
mod app {
    use rtt_target::{rtt_init_print, rprintln};
    use stm32f4xx_hal::{
        gpio::{self, Input, Output, PushPull},
        pac::TIM2,
        pac::TIM1,
        prelude::*,
        timer::{self, Event},
    };

    #[shared]
    struct Shared {
    }

    // Local resources go here
    #[local]
    struct Local {
        trigger_pin: gpio::PC2<Output<PushPull>>,   // Pin pour déclencher l'ultrason
        echo_pin: gpio::PC3<Input>,                 // Pin pour lire l'écho
        delay: timer::DelayUs<TIM1>,
        timer: timer::CounterUs<TIM2>,
    }

    #[init]
    fn init(mut ctx: init::Context) -> (Shared, Local, init::Monotonics) {

        rtt_init_print!();

        let mut dp = ctx.device;

        // Sépare le registre GPIOC en différentes broches (pins) pour pouvoir les manipuler individuellement.
        let gpioc = dp.GPIOC.split();

        // let mut syscfg = dp.SYSCFG.constrain();
        let trigger_pin = gpioc.pc2.into_push_pull_output();
        let echo_pin = gpioc.pc3.into_pull_down_input();

        // Set up the system clock. We want to run at 8MHz for this one.
        let rcc = dp.RCC.constrain();
        let clocks = rcc
        .cfgr
        .use_hse(8.MHz())
        .sysclk(168.MHz())
        .freeze();

        let mut delay = dp.TIM1.delay_us(&clocks);
        let mut timer = dp.TIM2.counter_us(&clocks);

        // Kick off the timer with 100 milliseconds timeout first
        timer.start(100.millis()).unwrap();

        // Set up to generate interrupt when timer expires
        timer.listen(Event::Update);

        (
            Shared {
               // Initialization of shared resources go here
            },
            Local {
                // Initialization of local resources go here
                trigger_pin,                                    // Pin pour déclencher l'ultrason
                echo_pin,                                       // Pin pour lire l'écho
                delay,
                timer,
            },
            init::Monotonics()
        )
    }

    // Optional idle, can be removed if not needed.
    #[idle]
    fn idle(_: idle::Context) -> ! {
        loop {
            continue;
        }
    }

    #[task(binds = TIM2, local=[trigger_pin, echo_pin, delay, timer])]
    fn read_sensor(mut ctx: read_sensor::Context) {

        rprintln!("Task : Read sensor");

        let trigger_pin = ctx.local.trigger_pin;
        let echo_pin = ctx.local.echo_pin;
        let delay = ctx.local.delay;
        let timer = ctx.local.timer;

        // Envoyer une impulsion de 10 µs sur le trigger pour démarrer la mesure
        trigger_pin.set_high();
        delay.delay_us(10_u32);  // Délai précis de 10 µs
        trigger_pin.set_low();

        // Attendre que l'écho passe à HIGH
        while echo_pin.is_low() {
        }

        // // Démarrer le timer pour mesurer la durée de l'écho
        let start_time = timer.now(); 

        // Attendre que l'écho passe à LOW (c'est la durée du signal)
        while echo_pin.is_high() {
        }

        // Lire la durée du signal d'écho en µs
        let echo_time = (timer.now() - start_time).to_micros();
        let _ = timer.cancel();

        // Calculer la distance
        rprintln!("Duration time : {}",echo_time);               
        let distance_cm = (echo_time as f64)*17.0/100.0;
        rprintln!("Distance : {}cm", distance_cm);

        let _ = timer.start(100.millis());

    }

}
