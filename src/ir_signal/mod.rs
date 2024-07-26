use std::fmt::Debug;
use std::time::Duration;

use crate::is_around::IsAround;

pub mod decode;
pub mod encode;

#[derive(Debug, Clone, Copy)]
pub struct Event {
    pub is_on: bool,
    pub duration: Duration,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum RemoteType {
    Generic,
    Samsung,
}

const FIRST_HIGH_GENERIC_DURATION: f64 = 0.009108749;
const FIRST_HIGH_SAMSUNG_DURATION: f64 = 0.004413791;
const FIRST_LOW_DURATION: f64 = 0.004424661;

const HIGH_DURATION: f64 = 0.000627288;
const LOW_0_DURATION: f64 = 0.000503018;
const LOW_1_DURATION: f64 = 0.001632658;

impl RemoteType {
    pub fn decode_first_high(duration: Duration) -> Option<Self> {
        let acceptable_error: f64 = 0.2;
        match duration {
            duration
                if duration.is_around(
                    Duration::from_secs_f64(FIRST_HIGH_GENERIC_DURATION),
                    acceptable_error,
                ) =>
            {
                Some(Self::Generic)
            }
            duration
                if duration.is_around(
                    Duration::from_secs_f64(FIRST_HIGH_SAMSUNG_DURATION),
                    acceptable_error,
                ) =>
            {
                Some(Self::Samsung)
            }
            _ => None,
        }
    }

    pub fn get_first_high_duration(&self) -> Duration {
        match self {
            Self::Generic => Duration::from_secs_f64(FIRST_HIGH_GENERIC_DURATION),
            Self::Samsung => Duration::from_secs_f64(FIRST_HIGH_SAMSUNG_DURATION),
        }
    }
}

#[derive(Debug, Clone, Copy)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Repeat {
    /// Total number of times to send the message
    pub times: usize,
    pub duration_between: Duration,
}

#[derive(PartialEq, Eq, Clone, Copy)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct IrPacket {
    pub remote_type: RemoteType,
    pub receiver_id: u16,
    pub button: u8,
}

impl Debug for IrPacket {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Debug::fmt(&IrPacketDebug::from(self), f)
    }
}

#[derive(Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct IrSignal {
    pub packet: IrPacket,
    pub repeat: Option<Repeat>,
}

#[derive(Debug)]
pub struct IrPacketDebug {
    pub remote_type: RemoteType,
    pub receiver_id: String,
    pub button: String,
}

impl From<&IrPacket> for IrPacketDebug {
    fn from(value: &IrPacket) -> Self {
        Self {
            remote_type: value.remote_type,
            receiver_id: format!("{:#06X}", value.receiver_id),
            button: format!("{:#04X}", value.button),
        }
    }
}
