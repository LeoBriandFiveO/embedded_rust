#![no_std]

use stm32f4xx_hal::{
    gpio::{Output, PushPull, Input},
    timer::{DelayUs, CounterUs},
};
use rtt_target::rprintln;

pub struct UltrasonicSensor<T, E, D> {
    trigger_pin: T,
    echo_pin: E,
    delay: D,
}

impl<T, E, D> UltrasonicSensor<T, E, D>
where
    T: Output<PushPull>,
    E: Input,
    D: DelayUs<u32>,
{
    pub fn new(trigger_pin: T, echo_pin: E, delay: D) -> Self {
        Self {
            trigger_pin,
            echo_pin,
            delay,
        }
    }

    pub fn measure_distance(
        &mut self,
        timer: &mut CounterUs,
    ) -> Option<f64> {
        rprintln!("Measuring distance...");

        // Envoyer une impulsion de 10 µs sur le trigger pour démarrer la mesure
        self.trigger_pin.set_high();
        self.delay.delay_us(10_u32);
        self.trigger_pin.set_low();

        // Attendre que l'écho passe à HIGH
        while self.echo_pin.is_low() {
        }

        // Démarrer le timer pour mesurer la durée de l'écho
        let start_time = timer.now();

        // Attendre que l'écho passe à LOW
        while self.echo_pin.is_high() {
        }

        // Lire la durée du signal d'écho en µs
        let echo_time = (timer.now() - start_time).to_micros();
        rprintln!("Duration time : {}µs", echo_time);

        // Calculer la distance en cm
        let distance_cm = (echo_time as f64)*17.0/100.0;
        rprintln!("Distance: {}cm", distance_cm);

        Some(distance_cm)
    }
}
