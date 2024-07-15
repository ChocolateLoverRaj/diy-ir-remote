use std::time::Duration;

use rppal::gpio::Level;

#[derive(Debug)]
pub struct Event {
    pub level: Level,
    pub duration: Duration,
}

pub struct Remote {
    pub id: [u8; 2],
}

impl Remote {
    pub fn encode(&self, data: u8) -> Vec<Event> {
        let first_high_duration = Duration::from_secs_f64(0.009108749);
        let first_low_duration = Duration::from_secs_f64(0.004424661);

        let high_duration = Duration::from_secs_f64(0.000627288);
        let low_0_duration = Duration::from_secs_f64(0.000503018);
        let low_1_duration = Duration::from_secs_f64(0.001632658);

        let mut events = vec![
            Event {
                level: Level::High,
                duration: first_high_duration,
            },
            Event {
                level: Level::Low,
                duration: first_low_duration,
            },
        ];

        let bytes = [self.id[0], self.id[1], data, !data];
        let bits = bytes.into_iter().flat_map(|byte| {
            let mut bits = vec![];
            for i in 0..8 {
                bits.push(byte & (1 << (7 - i)) != 0)
            }
            bits
        });

        for bit in bits.clone() {
            events.push(Event {
                level: Level::High,
                duration: high_duration,
            });
            events.push(Event {
                level: Level::Low,
                duration: match bit {
                    true => low_1_duration,
                    false => low_0_duration,
                },
            })
        }
        events.push(Event {
            level: Level::High,
            duration: high_duration,
        });

        events
    }
}
