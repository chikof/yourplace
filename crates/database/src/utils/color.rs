/// Represents a color with red, green, and blue components.
pub struct Color {
    r: u8, // Red component (0-255)
    g: u8, // Green component (0-255)
    b: u8, // Blue component (0-255)
}

impl Color {
    /// Returns the red component of the color.
    #[inline(always)]
    pub fn r(&self) -> u8 {
        self.r
    }

    /// Returns the green component of the color.
    #[inline(always)]
    pub fn g(&self) -> u8 {
        self.g
    }

    /// Returns the blue component of the color.
    #[inline(always)]
    pub fn b(&self) -> u8 {
        self.b
    }

    /// Creates a `Color` instance from a hexadecimal value.
    ///
    /// # Arguments
    ///
    /// * `hex` - A 32-bit unsigned integer representing the color in hexadecimal format (e.g., 0xRRGGBB).
    ///
    /// # Returns
    ///
    /// An `Option<Color>` containing the `Color` if the hexadecimal value is valid,
    /// or `None` if the value exceeds the maximum valid hex color (0xFFFFFF).
    pub fn from_hex(hex: u32) -> Option<Self> {
        if hex > 16777215 {
            return None;
        }

        Some(Self {
            r: ((hex >> 16) & 0xFF) as u8,
            g: ((hex >> 8) & 0xFF) as u8,
            b: (hex & 0xFF) as u8,
        })
    }

    /// Converts the color to its hexadecimal representation.
    ///
    /// # Returns
    ///
    /// A 32-bit signed integer representing the color in hexadecimal format (e.g., 0xRRGGBB).
    pub fn to_hex(&self) -> i32 {
        ((self.r as i32) << 16) + ((self.g as i32) << 8) + (self.b as i32)
    }
}
