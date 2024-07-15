use ir_remote::Remote;
use promptuity::{
    prompts::{Select, SelectOption},
    themes::FancyTheme,
    Promptuity, Term,
};
use rppal::gpio::{Gpio, Level};

const LED_PIN: u8 = 24;

const CONSTANT_NUMBERS: [u8; 2] = [0, 255];

const CARRIER_FREQUENCY: f64 = 38_000.0;

fn main() -> anyhow::Result<()> {
    // I am using software PWM because hardware PWM wasn't showing up on NixOS.
    // You can probably use hardware PWM instead
    // (but I don't think it really matters unless the rpi has heavy CPU usage that would mess up software PWM)
    let mut pin = Gpio::new()?.get(LED_PIN)?.into_output();
    let remote = Remote {
        id: CONSTANT_NUMBERS,
    };

    let mut term = Term::default();
    let mut theme = FancyTheme::default();
    let mut p = Promptuity::new(&mut term, &mut theme);

    p.term().clear()?;
    p.with_intro("Zenyatta LED Control").begin()?;
    while let Ok(button) = p.prompt(
        Select::new(
            "Choose a signal to send to the Zenyatta remote",
            vec![
                SelectOption::new("Brightness Up", 0x90),
                SelectOption::new("Brightness Down", 0xB8),
                SelectOption::new("Off", 0xF8),
                SelectOption::new("On", 0xB0),
                SelectOption::new("Red", 0x98),
                SelectOption::new("Green", 0xD8),
                SelectOption::new("Blue", 0x88),
                SelectOption::new("White", 0xA8),
                SelectOption::new("Flash", 0xB2),
                SelectOption::new("Strobe", 0x00),
                SelectOption::new("Fade", 0x58),
                SelectOption::new("Smooth", 0x30),
            ],
        )
        .with_page_size(usize::MAX),
    ) {
        for event in remote.encode(button) {
            match event.level {
                Level::High => {
                    pin.set_pwm_frequency(CARRIER_FREQUENCY, 0.5)?;
                }
                Level::Low => {
                    pin.set_pwm_frequency(CARRIER_FREQUENCY, 0.0)?;
                }
            }
            spin_sleep::sleep(event.duration);
        }
        pin.clear_pwm()?;
    }

    Ok(())
}
