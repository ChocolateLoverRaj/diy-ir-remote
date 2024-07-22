use std::time::Duration;

use bitvec::bitarr;
use bitvec::prelude::*;

use super::IrPacket;
use super::{Event, IrSignal, FIRST_LOW_DURATION, HIGH_DURATION, LOW_0_DURATION, LOW_1_DURATION};

impl IrPacket {
    pub fn encode(&self) -> Vec<Event> {
        let first_high_duration = self.remote_type.get_first_high_duration();
        let first_low_duration = Duration::from_secs_f64(FIRST_LOW_DURATION);

        let high_duration = Duration::from_secs_f64(HIGH_DURATION);
        let low_0_duration = Duration::from_secs_f64(LOW_0_DURATION);
        let low_1_duration = Duration::from_secs_f64(LOW_1_DURATION);

        let mut events = vec![
            Event {
                is_on: true,
                duration: first_high_duration,
            },
            Event {
                is_on: false,
                duration: first_low_duration,
            },
        ];

        let mut bits = bitarr!(u32, Msb0; 0; 32);
        bits[..16].store(self.receiver_id);
        bits[16..24].store(self.button);
        bits[24..].store(!self.button);
        for bit in bits {
            events.push(Event {
                is_on: true,
                duration: high_duration,
            });
            events.push(Event {
                is_on: false,
                duration: match bit {
                    true => low_1_duration,
                    false => low_0_duration,
                },
            })
        }
        events.push(Event {
            is_on: true,
            duration: high_duration,
        });
        events
    }
}

impl IrSignal {
    pub fn encode(&self) -> Vec<Event> {
        let mut events = self.packet.encode();
        if let Some(repeat) = self.repeat {
            let packet_size = events.len();
            let times_left = repeat.times - 1;
            for _ in 0..times_left {
                events.push(Event {
                    is_on: false,
                    duration: repeat.duration_between,
                });
                events.extend_from_within(..packet_size);
            }
        }
        events
    }
}
