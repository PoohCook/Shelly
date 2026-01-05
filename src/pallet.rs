
use smart_leds::RGB8;

#[allow(dead_code)]
pub enum Colors {
    Black,
    Red,
    Orange,
    Yellow,
    Green,
    Cyan,
    Blue,
    Magenta,
    White,
    Off,
}

impl Colors {
    pub fn as_rgb(&self) -> RGB8 {
        match *self {
            Colors::Black => RGB8::new(0x00, 0x00, 0x00),
            Colors::Red => RGB8::new(0x3f, 0x00, 0x00),
            Colors::Orange => RGB8::new(0x3f, 0x1f, 0x00),
            Colors::Yellow => RGB8::new(0x3f, 0x3f, 0x00),
            Colors::Green => RGB8::new(0x00, 0x3f, 0x00),
            Colors::Cyan => RGB8::new(0x00, 0x3f, 0x3f),
            Colors::Blue => RGB8::new(0x00, 0x00, 0x3f),
            Colors::Magenta => RGB8::new(0x3f, 0x00, 0x3f),
            Colors::White => RGB8::new(0x3f, 0x3f, 0x3f),
            Colors::Off => RGB8::new(0x00, 0x00, 0x00),
        }
    }
}

/// Packs tint (color) and level (brightness) into a single byte
/// tint: 0-6, stored in upper 4 bits
/// level: 0-15, stored in lower 4 bits
pub fn get_temperature(tint: u8, level: u8) -> u8 {
    let tint = if tint > 6 { 0 } else { tint };
    let level = if level > 15 { 15 } else { level };

    (tint & 0x0f) << 4 | (level & 0x0f)
}

/// Adjusts the brightness level of a temperature value
/// Returns 0 if the adjusted level would be less than 1
pub fn adjust_temperature(temperature: u8, adjust: i8) -> u8 {
    let tint = (temperature & 0xf0) >> 4;
    let level = (temperature & 0x0f) as i8 + adjust;

    if level < 1 {
        return 0;
    }

    get_temperature(tint, level as u8)
}

/// Converts a temperature value to an RGB color with brightness
/// Uses a global brightness value scaled by the level
pub fn get_color_bright(temperature: u8, bright: u8) -> RGB8 {
    let tint = (temperature & 0xf0) >> 4;
    let level = (temperature & 0x0f) as f32 / 15.0;

    match tint {
        1 => RGB8::new((bright as f32 * level) as u8, 0, 0),
        2 => RGB8::new(0, (bright as f32 * level) as u8, 0),
        3 => RGB8::new(0, 0, (bright as f32 * level) as u8),
        4 => RGB8::new(
            (bright as f32 * level * 0.5) as u8,
            0,
            (bright as f32 * level * 0.5) as u8,
        ),
        5 => RGB8::new(
            (bright as f32 * level * 0.5) as u8,
            (bright as f32 * level * 0.5) as u8,
            0,
        ),
        6 => RGB8::new(
            0,
            (bright as f32 * level * 0.5) as u8,
            (bright as f32 * level * 0.5) as u8,
        ),
        _ => RGB8::new(0, 0, 0),
    }
}
