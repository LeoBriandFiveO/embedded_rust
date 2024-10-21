#![deny(unsafe_code)]                           // Interdit l'utilisation de code non-sécurisé

#![allow(clippy::empty_loop)]

#![no_main]                                     // Empêche Rust de générer un point d'entrée 
                                                // standard (main). Le point d'entrée est fourni 
                                                // par le runtime Cortex-M

#![no_std]                                      // Pas de support de la biblioyèque std

// Halt on panic
use panic_halt as _;                            // Gère les paniques en arrêtant le programme sur une erreur      
use cortex_m as _;                              // Importe le crate Cortex-M pour interagir avec les périphériques Cortex-M
use cortex_m::delay::Delay;                     // Fournit une abstraction de temporisation basée sur SysTick
use cortex_m_rt::entry;                         // Fournit le macro "entry" pour définir le point d'entrée du programme
use rtt_target::{rtt_init_print, rprintln};     // Ces fonctions permettent de communiquer avec un débogueur via RTT (Real-Time Transfer), une méthode rapide de transfert de données.
use stm32f4xx_hal as hal;                       // Importe la bibliothèque HAL (Hardware Abstraction Layer) pour les microcontrôleurs STM32F4.
use crate::hal::{pac, prelude::*};              // Importe les périphériques spécifiques au STM32F4 et certains traits d'extension (prelude) pour simplifier l'initialisation des périphériques.

#[allow(clippy::empty_loop)]
#[entry]
fn main() -> ! {
    if let (Some(dp), Some(cp)) = (
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
        
        let mut led = gpioa.pa5.into_push_pull_output();

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
