use std::time::Duration;

use ir_remote::ir_signal::{IrPacket, IrSignal, RemoteType, Repeat};
use ordered_hash_map::OrderedHashMap;
use promptuity::{
    prompts::{Select, SelectOption},
    themes::FancyTheme,
    Promptuity, Term,
};
use rppal::gpio::{Gpio, Level};

const LED_PIN: u8 = 24;

const CARRIER_FREQUENCY: f64 = 38_000.0;

#[derive(Clone)]
struct RemoteCodes {
    remote_type: RemoteType,
    receiver_id: u16,
    codes: OrderedHashMap<&'static str, u8>,
    repeat: Option<Repeat>,
}

impl Default for RemoteCodes {
    fn default() -> Self {
        Self {
            remote_type: RemoteType::Generic,
            codes: Default::default(),
            receiver_id: Default::default(),
            repeat: Default::default(),
        }
    }
}

fn get_remotes() -> OrderedHashMap<&'static str, RemoteCodes> {
    let mut m = OrderedHashMap::default();
    m.insert(
        "Zenyatta Light",
        RemoteCodes {
            remote_type: RemoteType::Generic,
            receiver_id: 0x00FF,
            codes: {
                let mut m = OrderedHashMap::default();
                m.insert("Brightness Up", 0x90);
                m.insert("Brightness Down", 0xB8);
                m.insert("Off", 0xF8);
                m.insert("On", 0xB0);
                m.insert("Red", 0x98);
                m.insert("Green", 0xD8);
                m.insert("Blue", 0x88);
                m.insert("White", 0xA8);
                m.insert("Flash", 0xB2);
                m.insert("Strobe", 0x00);
                m.insert("Fade", 0x58);
                m.insert("Smooth", 0x30);
                m
            },
            repeat: None,
        },
    );
    m.insert(
        "Pioneer Sound System",
        RemoteCodes {
            remote_type: RemoteType::Generic,
            receiver_id: 0xA55A,
            codes: {
                let mut m = OrderedHashMap::new();
                m.insert("Power", 0x38);
                m.insert("Sleep", 0x12);
                m.insert("TV", 0x30);
                m.insert("Volume Up", 0x50);
                m.insert("Volume Down", 0xD0);
                m.insert("Mute", 0x48);
                m
            },
            repeat: Some(Repeat {
                times: 2,
                duration_between: Duration::from_secs_f64(0.027116677),
            }),
        },
    );
    m.insert(
        "[TV] Samsung 6 Series (55)",
        RemoteCodes {
            remote_type: RemoteType::Samsung,
            receiver_id: 0xE0E0,
            codes: {
                let mut m = OrderedHashMap::new();
                m.insert("Power", 0x67);
                m
            },
            repeat: None,
        },
    );
    m
}

fn main() -> anyhow::Result<()> {
    // I am using software PWM because hardware PWM wasn't showing up on NixOS.
    // You can probably use hardware PWM instead
    // (but I don't think it really matters unless the rpi has heavy CPU usage that would mess up software PWM)
    let mut pin = Gpio::new()?.get(LED_PIN)?.into_output();

    let mut term = Term::default();
    let mut theme = FancyTheme::default();
    let mut p = Promptuity::new(&mut term, &mut theme);

    p.term().clear()?;
    p.with_intro("IR Remote").begin()?;

    if let Ok((name, remote)) = p.prompt(&mut Select::new(
        "Which receiver to send signals to?",
        get_remotes()
            .into_iter()
            .map(|(name, remote)| SelectOption::new(name, (name, remote)))
            .collect(),
    )) {
        while let Ok(button) = p.prompt(
            Select::new(
                format!("Which button to press for {name}?"),
                remote
                    .codes
                    .clone()
                    .into_iter()
                    .map(|(name, button)| SelectOption::new(name, button))
                    .collect(),
            )
            .with_page_size(usize::MAX),
        ) {
            for event in (IrSignal {
                packet: IrPacket {
                    remote_type: remote.remote_type,
                    receiver_id: remote.receiver_id,
                    button,
                },
                repeat: remote.repeat,
            })
            .encode()
            {
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
    }

    Ok(())
}
