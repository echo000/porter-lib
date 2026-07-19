/// Audio formats.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AudioFormat {
    /// Placeholder for an unknown audio format.
    Unknown,
    /// Pulse code modulation: https://wiki.multimedia.cx/index.php/PCM
    IntegerPcm,
    /// MS ADPCM: https://wiki.multimedia.cx/index.php/Microsoft_ADPCM
    MsAdpcm,
    /// Pulse code modulation: https://wiki.multimedia.cx/index.php/PCM
    FloatPcm,
    /// Wwise, custom vorbis: https://wiki.multimedia.cx/index.php/Vorbis
    WwiseVorbis,
    /// Raw headerless flac: https://wiki.multimedia.cx/index.php/FLAC
    RawFlac,
    /// IMA ADPCM: https://wiki.multimedia.cx/index.php/IMA_ADPCM
    ImaAdpcm,
    /// Xbox 360 XMA2: https://wiki.multimedia.cx/index.php/Microsoft_Xbox_Media_Audio
    Xma2,
    /// Windows Media Audio 1: https://wiki.multimedia.cx/index.php/Windows_Media_Audio
    WmaV1,
    /// Windows Media Audio 2 (as found in xWMA): https://wiki.multimedia.cx/index.php/Windows_Media_Audio
    WmaV2,
}

impl AudioFormat {
    /// Whether or not the audio format is compressed.
    pub const fn is_compressed(&self) -> bool {
        matches!(
            self,
            Self::MsAdpcm
                | Self::WwiseVorbis
                | Self::RawFlac
                | Self::ImaAdpcm
                | Self::Xma2
                | Self::WmaV1
                | Self::WmaV2
        )
    }

    /// Whether or not the audio format is a coercible version of the given format.
    pub const fn is_coercible(&self, format: Self) -> bool {
        matches!(
            (self, format),
            (Self::IntegerPcm, Self::FloatPcm) | (Self::FloatPcm, Self::IntegerPcm)
        )
    }

    /// Whether or not the audio format is compressible.
    pub const fn is_compressible(&self) -> bool {
        false
    }
}
