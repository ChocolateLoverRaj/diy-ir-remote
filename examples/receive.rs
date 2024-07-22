use std::{
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    time::{Duration, SystemTime},
};

use ir_remote::ir_signal::{Event, IrSignal};
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
        is_on: bool,
        time: SystemTime,
    }
    let mut previous = None::<PreviousChange>;
    let mut events = vec![];
    while running.load(Ordering::SeqCst) {
        if let Ok(Some(level)) = pin.poll_interrupt(false, Some(Duration::from_millis(50))) {
            let level = !level;
            let now = SystemTime::now();
            let change = PreviousChange {
                is_on: level == Level::High,
                time: now,
            };
            if let Some(previous) = previous {
                if (level == Level::High) == previous.is_on {
                    continue;
                }
                let event = Event {
                    is_on: previous.is_on,
                    duration: now.duration_since(previous.time)?,
                };
                // println!("{event:?}");
                events.push(event);
            }
            previous = Some(change);
        }
    }
    println!();
    println!("{events:#?} {}", events.len());
    if let Some(previous) = previous {
        println!("Last is on: {:?}", previous.is_on);
    }

    println!("Decoded signal: {:#?}", IrSignal::decode(events.iter()));
    Ok(())
}
