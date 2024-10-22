// #![deny(unsafe_code)]                           // Interdit l'utilisation de code non-sécurisé

#![allow(clippy::empty_loop)]

#![no_main]                                     // Empêche Rust de générer un point d'entrée 
                                                // standard (main). Le point d'entrée est fourni 
                                                // par le runtime Cortex-M

#![no_std]                                      // Pas de support de la biblioyèque std

// Halt on panic
use core::cell::{RefCell};
use cortex_m::interrupt::Mutex;
use cortex_m_rt::entry;
use panic_halt as _;
use stm32f4xx_hal::{
    gpio::{self, Edge, Input, Output},
    pac::{self, interrupt},
    prelude::*,
};
use rtt_target::{rtt_init_print, rprintln};    

type ButtonPin = gpio::PC13<Input>;
type LedPin = gpio::PA5<Output>;

// Create a Global Variable for the GPIO Peripheral that I'm going to pass around.
static G_BUTTON: Mutex<RefCell<Option<ButtonPin>>> = Mutex::new(RefCell::new(None));

// Create a Global Variable for the GPIO Peripheral that I'm going to pass around.
static G_LED: Mutex<RefCell<Option<LedPin>>> = Mutex::new(RefCell::new(None));


// #[allow(clippy::empty_loop)]
#[entry]
fn main() -> ! {
    if let (Some(mut dp), Some(cp)) = (
        pac::Peripherals::take(),
        cortex_m::peripheral::Peripherals::take(),
    ) 
    {
        // Initialise our debug printer.
        rtt_init_print!();

        // Send a message back via the debugger.
        rprintln!("Hello, world!");

        // Sépare le registre GPIOA en différentes broches (pins) pour pouvoir les manipuler individuellement.
        let gpioa = dp.GPIOA.split();
        let gpioc = dp.GPIOC.split();

        let mut syscfg = dp.SYSCFG.constrain();
        let mut led = gpioa.pa5.into_push_pull_output();
        let mut button = gpioc.pc13;

        button.make_interrupt_source(&mut syscfg);
        button.trigger_on_edge(&mut dp.EXTI,gpio::Edge::Rising);
        button.enable_interrupt(&mut dp.EXTI);

        led.toggle();

        unsafe {
            cortex_m::peripheral::NVIC::unmask(button.interrupt());
        }

        cortex_m::interrupt::free(|cs| {
            G_BUTTON.borrow(cs).replace(Some(button));
            G_LED.borrow(cs).replace(Some(led)); 
        });

        // Set up the system clock. We want to run at 8MHz for this one.
        let rcc = dp.RCC.constrain();
        let clocks = rcc.cfgr.use_hse(8.MHz()).freeze();
        let mut delay = dp.TIM1.delay_ms(&clocks);

        // Create a delay abstraction based on SysTick
        let mut delay = cp.SYST.delay(&clocks);
        loop {

        }
    }

    loop {}

}

#[interrupt]
fn EXTI15_10() {
    // Start a Critical Section
    cortex_m::interrupt::free(|cs| {
        rprintln!("Interrupt");
        rprintln!("Led toggled");

        // // Obtain Access to Led Global Data and toggle led
        let mut led = G_LED.borrow(cs).borrow_mut();
        led.as_mut().unwrap().toggle();

        // // Obtain Access to Button Global Data and clear interrupt bit
        let mut button = G_BUTTON.borrow(cs).borrow_mut();
        button.as_mut().unwrap().clear_interrupt_pending_bit();
    

        
    });
}
