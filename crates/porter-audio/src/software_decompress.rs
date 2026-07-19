use crate::Audio;
use crate::AudioError;
use crate::AudioFormat;

/// Utility method for formats that require decompression before conversion.
pub fn software_decompress_audio(audio: &mut Audio) -> Result<(), AudioError> {
    match audio.format() {
        AudioFormat::MsAdpcm => {
            #[cfg(feature = "ms-adpcm")]
            crate::decompress_ms_adpcm(audio)?;
            #[cfg(not(feature = "ms-adpcm"))]
            return Err(AudioError::ConversionFeatureDisabled);
        }
        AudioFormat::ImaAdpcm => {
            #[cfg(feature = "ima-adpcm")]
            crate::decompress_ima_adpcm(audio)?;
            #[cfg(not(feature = "ima-adpcm"))]
            return Err(AudioError::ConversionFeatureDisabled);
        }
        AudioFormat::WwiseVorbis => {
            #[cfg(feature = "wwise-vorbis")]
            crate::decompress_wwise_vorbis(audio)?;
            #[cfg(not(feature = "wwise-vorbis"))]
            return Err(AudioError::ConversionFeatureDisabled);
        }
        AudioFormat::RawFlac => {
            #[cfg(feature = "raw-flac")]
            crate::decompress_raw_flac(audio)?;
            #[cfg(not(feature = "raw-flac"))]
            return Err(AudioError::ConversionFeatureDisabled);
        }
        AudioFormat::Xma2 | AudioFormat::WmaV1 | AudioFormat::WmaV2 => {
            #[cfg(feature = "xma2-wma")]
            crate::decompress_xma2_wma(audio)?;
            #[cfg(not(feature = "xma2-wma"))]
            return Err(AudioError::ConversionFeatureDisabled);
        }
        _ => return Err(AudioError::ConversionError),
    }

    #[allow(unreachable_code)]
    Ok(())
}
