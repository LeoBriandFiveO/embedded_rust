// #![deny(unsafe_code)]
#![no_main]
#![no_std]


// Halt on panic
use panic_halt as _;

#[rtic::app(device = stm32f4xx_hal::pac, peripherals = true)] // peripherals = true makes sure that the device handle/field is available for use later in our code
mod app {
    use rtt_target::{rtt_init_print, rprintln};
    use stm32f4xx_hal::{
        gpio::{self, Edge, Input, Output, PushPull},
        pac::TIM2,
        prelude::*,
        timer::{self, Event},
    };

    #[shared]
    struct Shared {
        timer: timer::CounterMs<TIM2>,
        led_state: bool,
    }

    // Local resources go here
    #[local]
    struct Local {
        button: gpio::PC13<Input>,
        led: gpio::PA5<Output<PushPull>>,
    }

    #[init]
    fn init(mut ctx: init::Context) -> (Shared, Local, init::Monotonics) {

        rtt_init_print!();

        let mut dp = ctx.device;

        // Sépare le registre GPIOA en différentes broches (pins) pour pouvoir les manipuler individuellement.
        let gpioa = dp.GPIOA.split();
        let gpioc = dp.GPIOC.split();

        let mut syscfg = dp.SYSCFG.constrain();
        let mut led = gpioa.pa5.into_push_pull_output();
        let mut button = gpioc.pc13;

        button.make_interrupt_source(&mut syscfg);
        button.trigger_on_edge(&mut dp.EXTI,gpio::Edge::Falling);
        button.enable_interrupt(&mut dp.EXTI);

        led.toggle();

        // Set up the system clock. We want to run at 8MHz for this one.
        let rcc = dp.RCC.constrain();
        let clocks = rcc.cfgr.use_hse(8.MHz()).freeze();

        let mut timer = dp.TIM2.counter_ms(&clocks);

        // Kick off the timer with 2 seconds timeout first
        timer.start(2000.millis()).unwrap();

        // Set up to generate interrupt when timer expires
        timer.listen(Event::Update);

        (
            Shared {
               // Initialization of shared resources go here
               timer, led_state: true
            },
            Local {
                // Initialization of local resources go here
                button, led
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


    // Two tasks :
    // button_pressed toggle the led and is activated when the user button is pressed
    // timer_expired prints the led state every 2 seconds (TIM2)
    #[task(binds = EXTI15_10, local = [button, led], shared = [led_state])]
    fn button_pressed(mut ctx: button_pressed::Context) {

        // Obtain access to Button Peripheral and Clear Interrupt Pending Flag
        ctx.local.button.clear_interrupt_pending_bit();

        let led = ctx.local.led;
        let mut led_state = ctx.shared.led_state;

        // Inverser l'état de la LED
        if led.is_set_low(){
            led.set_high(); // Allumer la LED
            led_state.lock(|state| *state = true);
        } else {
            led.set_low(); // Éteindre la LED
            led_state.lock(|state| *state = false);
        }
    }

    #[task(binds = TIM2, shared=[led_state, timer])]
    fn timer_expired(mut ctx: timer_expired::Context) {

        // Clear timer pending interrupt
        ctx.shared.timer.lock(|tim| {
            // Access the TIM2 register block directly
            let tim_peripheral = unsafe { &*stm32f4xx_hal::pac::TIM2::ptr() };
            
            // Clear the UIF (Update Interrupt Flag)
            tim_peripheral.sr.modify(|_, w| w.uif().clear());
        });

        // // Comprendre pourquoi ça ne fonctionne pas ???
        // ctx.shared.timer.lock(|tim| tim.clear_interrupt(Event::Update));
    
        let mut led_state = ctx.shared.led_state;

        led_state.lock(|state| {
            if *state {
                rprintln!("LED is ON");
            } else {
                rprintln!("LED is OFF");
            }
        });

    }

}
