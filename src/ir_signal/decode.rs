use std::time::Duration;

use bitvec::bitarr;
use bitvec::prelude::*;

use crate::are_all_equal::AreAllEqual;
use crate::is_around::IsAround;

use super::{
    Event, IrPacket, IrSignal, RemoteType, Repeat, FIRST_LOW_DURATION, HIGH_DURATION,
    LOW_0_DURATION, LOW_1_DURATION,
};

#[derive(Debug)]
pub enum DecodeError {
    FirstHighMissing,
    LengthBad,
    FirstHighBad,
    FirstLowBad,
    HighNotHigh,
    HighBad(Duration),
    LowNotLow,
    LowBad(Duration),
    IdBad,
    LastByteBad,
    SpaceNotLow,
    MultipleDifferentPackets(Vec<IrPacket>),
}

impl IrSignal {
    pub fn decode<'a, T: Iterator<Item = &'a Event>>(mut events: T) -> Result<Self, DecodeError> {
        let acceptable_error: f64 = 0.2;

        let decode_packet = |events: &mut T| {
            // Check for first two special events
            let signal_type = {
                let first_high = events.next().ok_or(DecodeError::FirstHighMissing)?;
                if !first_high.is_on {
                    return Err(DecodeError::FirstHighBad);
                }

                RemoteType::decode_first_high(first_high.duration).ok_or(DecodeError::FirstHighBad)
            }?;
            {
                let first_low = events.next().ok_or(DecodeError::LengthBad)?;
                if first_low.is_on {
                    return Err(DecodeError::FirstLowBad);
                }
                if !first_low.duration.is_around(
                    Duration::from_secs_f64(FIRST_LOW_DURATION),
                    acceptable_error,
                ) {
                    return Err(DecodeError::FirstLowBad);
                }
            }

            let consume_high = |events: &mut T| -> Result<(), DecodeError> {
                let high = events.next().ok_or(DecodeError::LengthBad)?;
                if !high.is_on {
                    return Err(DecodeError::HighNotHigh);
                }
                if !high
                    .duration
                    .is_around(Duration::from_secs_f64(HIGH_DURATION), acceptable_error)
                {
                    return Err(DecodeError::HighBad(high.duration));
                }
                Ok(())
            };
            // Get actual bits
            let mut bits = bitarr!(u32, Msb0; 0; 32);
            for i in 0..bits.len() {
                consume_high(events)?;
                {
                    let low = events.next().ok_or(DecodeError::LengthBad)?;
                    if low.is_on {
                        return Err(DecodeError::LowNotLow);
                    }
                    let bit = match low.duration {
                        duration
                            if duration.is_around(
                                Duration::from_secs_f64(LOW_0_DURATION),
                                acceptable_error,
                            ) =>
                        {
                            Some(false)
                        }
                        duration
                            if duration.is_around(
                                Duration::from_secs_f64(LOW_1_DURATION),
                                acceptable_error,
                            ) =>
                        {
                            Some(true)
                        }
                        _ => None,
                    };
                    bits.set(i, bit.ok_or(DecodeError::LowBad(low.duration))?);
                }
            }
            // Always one last high
            consume_high(events)?;

            // Make sure 4th byte is 3rd byte inverted
            if bits[24..32] != !bits[16..24].to_owned() {
                return Err(DecodeError::LastByteBad);
            }

            Ok(IrPacket {
                remote_type: signal_type,
                receiver_id: bits[..16].load(),
                button: bits[16..24].load(),
            })
        };
        let mut packets = vec![decode_packet(&mut events)?];
        let mut spaces = vec![];
        loop {
            match events.next() {
                Some(event) => {
                    if event.is_on {
                        println!("{:#?}", event);
                        break Err(DecodeError::SpaceNotLow);
                    }
                    spaces.push(event.duration);
                    packets.push(decode_packet(&mut events)?);
                }
                None => {
                    if !packets.are_all_equal() {
                        break Err(DecodeError::MultipleDifferentPackets(packets));
                    }
                    // Verify packets are all identical
                    // TODO: Verify consistent space durations with a margin of error
                    break Ok(Self {
                        packet: packets[0],
                        repeat: match spaces.len() {
                            0 => None,
                            1.. => Some(Repeat {
                                times: packets.len(),
                                // TODO: get average duration between
                                duration_between: spaces[0],
                            }),
                        },
                    });
                }
            };
        }
    }
}
