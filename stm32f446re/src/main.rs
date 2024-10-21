#![deny(unsafe_code)]                           // Interdit l'utilisation de code non-sécurisé

#![allow(clippy::empty_loop)]

#![no_main]                                     // Empêche Rust de générer un point d'entrée 
                                                // standard (main). Le point d'entrée est fourni 
                                                // par le runtime Cortex-M

#![no_std]                                      // Pas de support de la biblioyèque std

// Halt on panic
use core::cell::{Cell, RefCell};
use panic_halt as _;                            // Gère les paniques en arrêtant le programme sur une erreur      
use cortex_m as _;                              // Importe le crate Cortex-M pour interagir avec les périphériques Cortex-M
// use cortex_m::delay::Delay;                     // Fournit une abstraction de temporisation basée sur SysTick
use cortex_m_rt::entry;                         // Fournit le macro "entry" pour définir le point d'entrée du programme
use rtt_target::{rtt_init_print, rprintln};     // Ces fonctions permettent de communiquer avec un débogueur via RTT (Real-Time Transfer), une méthode rapide de transfert de données.
use stm32f4xx_hal as hal;                       // Importe la bibliothèque HAL (Hardware Abstraction Layer) pour les microcontrôleurs STM32F4.
use crate::hal::{pac, prelude::*, gpio};              // Importe les périphériques spécifiques au STM32F4 et certains traits d'extension (prelude) pour simplifier l'initialisation des périphériques.


// Create a Global Variable for the GPIO Peripheral that I'm going to pass around.
static G_BUTTON: Mutex<RefCell<Option<ButtonPin>>> = Mutex::new(RefCell::new(None));
// Create a Global Variable for the delay value that I'm going to pass around for delay.
static G_DELAYMS: Mutex<Cell<u32>> = Mutex::new(Cell::new(2000_u32));


#[allow(clippy::empty_loop)]
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
        let mut button = gpioc.pc13.into_floating_input();
        button.make_interrupt_source(&mut syscfg);
        button.trigger_on_edge(&mut dp.EXTI,gpio::Edge::Falling);
        button.enable_interrupt(&mut dp.EXTI);



        // Set up the system clock. We want to run at 48MHz for this one.
        let rcc = dp.RCC.constrain();
        let clocks = rcc.cfgr.sysclk(48.MHz()).freeze();

        // Create a delay abstraction based on SysTick
        let mut delay = cp.SYST.delay(&clocks);
        loop {
            led.toggle();
            delay.delay_ms(1000);
        }
    }

    loop {}

}

#[interrupt]
fn EXTI15_10() {
    // Start a Critical Section
    cortex_m::interrupt::free(|cs| {
        // Obtain Access to Delay Global Data and Adjust Delay
        G_DELAYMS
            .borrow(cs)
            .set(G_DELAYMS.borrow(cs).get() - 500_u32);
        if G_DELAYMS.borrow(cs).get() < 500_u32 {
            G_DELAYMS.borrow(cs).set(2000_u32);
        }
        // Obtain access to Global Button Peripheral and Clear Interrupt Pending Flag
        let mut button = G_BUTTON.borrow(cs).borrow_mut();
        button.as_mut().unwrap().clear_interrupt_pending_bit();
    });
}
