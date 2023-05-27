/// Enum for opt level
/// Some compilers may not support all opt levels
#[derive(Debug, Clone)]
pub enum OptLevel {
    /// No optimizations
    None,
    /// Optimize for speed
    Speed,
    /// Optimize for size
    Size,
    /// Opt level 1
    O1,
    /// Opt level 2
    O2,
    /// Opt level 3
    O3,
    /// Custom optimization level
    Custom(String)
}

impl OptLevel {
    pub fn as_stanard_opt_char(&self) -> String {
        match self {
            OptLevel::None => "0",
            OptLevel::Speed => "fast",
            OptLevel::Size => "z",
            OptLevel::O1 => "1",
            OptLevel::O2 => "2",
            OptLevel::O3 => "3",
            OptLevel::Custom(c) => c
        }.to_string()
    }
}