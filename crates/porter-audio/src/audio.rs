#[derive(Debug, Clone, Default)]
pub struct Audio {
    /// Frame rate
    pub frame_rate: u32,
    /// Total frame count
    pub frame_count: u32,
    /// Channel count
    pub channel_count: u16,
    /// Bits per sample
    pub bits_per_sample: u16,
    /// Sample data
    pub samples: Vec<u8>,
}

impl Audio {
    pub fn new() -> Self {
        Self::default()
    }
}
