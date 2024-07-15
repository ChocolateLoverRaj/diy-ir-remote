use std::{
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    time::{Duration, SystemTime},
};

use ir_remote::Event;
use rppal::gpio::{Gpio, Level};
use simple_signal::Signal;

const INPUT_PIN: u8 = 23;

fn main() -> anyhow::Result<()> {
    let mut pin = Gpio::new()?.get(INPUT_PIN)?.into_input_pulldown();
    pin.set_interrupt(rppal::gpio::Trigger::Both)?;

    // When a SIGINT (Ctrl-C) or SIGTERM signal is caught, atomically set running to false.
    let running = Arc::new(AtomicBool::new(true));
    simple_signal::set_handler(&[Signal::Int, Signal::Term], {
        let running = running.clone();
        move |_| {
            running.store(false, Ordering::SeqCst);
        }
    });

    #[derive(Clone, Copy)]
    struct PreviousChange {
        level: Level,
        time: SystemTime,
    }
    let mut previous = None::<PreviousChange>;
    while running.load(Ordering::SeqCst) {
        if let Ok(Some(level)) = pin.poll_interrupt(false, Some(Duration::from_millis(50))) {
            let now = SystemTime::now();
            let change = PreviousChange { level, time: now };
            match previous {
                Some(previous) => {
                    if level == previous.level {
                        continue;
                    }
                    let event = Event {
                        level: previous.level,
                        duration: now.duration_since(previous.time)?,
                    };
                    println!("{event:?}");
                }
                None => {}
            }
            previous = Some(change);
        }
    }
    if let Some(previous) = previous {
        println!();
        println!("Last level: {:?}", previous.level);
    }

    Ok(())
}
