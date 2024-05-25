use strum_macros::EnumIter;

use crate::utils::{u32_rgba_to_u8_rgba, u8_rgba_to_u32_rgba, EMPTY_COLOR};

// Matter identifier representing matter that we simulate
#[repr(u8)]
#[derive(EnumIter, Debug, Copy, Clone, Eq, PartialEq)]
pub enum MatterId {
    Empty = 0,
    Rock = 1,
    Sand = 2,
    Water = 3,
}

impl Default for MatterId {
    fn default() -> Self {
        MatterId::Empty
    }
}

impl From<u8> for MatterId {
    fn from(item: u8) -> Self {
        unsafe { std::mem::transmute(item) }
    }
}

impl MatterId {
    fn color_rgba_u8(&self) -> [u8; 4] {
        let color = match *self {
            MatterId::Empty => EMPTY_COLOR,
            MatterId::Rock => 0xa9a9a9ff,
            MatterId::Sand => 0xc2b280ff,
            MatterId::Water => 0x0000ffff,
        };
        u32_rgba_to_u8_rgba(color)
    }
}

// Matter data where first 3 bytes are saved for color and last 4th byte is saved for matter identifier
#[derive(Default, Copy, Clone)]
pub struct MatterWithColor {
    pub value: u32,
}

impl MatterWithColor {
    pub fn new(matter_id: MatterId) -> MatterWithColor {
        let color = matter_id.color_rgba_u8();
        MatterWithColor {
            value: u8_rgba_to_u32_rgba(color[0], color[1], color[2], matter_id as u8),
        }
    }
}

impl From<u32> for MatterWithColor {
    fn from(item: u32) -> Self {
        Self { value: item }
    }
}
