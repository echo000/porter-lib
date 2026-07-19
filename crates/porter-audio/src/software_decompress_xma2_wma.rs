use xma2_rs::WaveFormatEx;
use xma2_rs::Xma2WaveFormatEx;

use crate::Audio;
use crate::AudioError;
use crate::AudioFormat;

/// Decompresses XMA2 (0x166) or WMA v1/v2 (0x160/0x161) audio streams to
/// 16bit integer pcm samples using the xma2-rs decoder.
pub fn decompress_xma2_wma(audio: &mut Audio) -> Result<(), AudioError> {
    let format_tag: u16 = match audio.format() {
        AudioFormat::Xma2 => 0x166,
        AudioFormat::WmaV1 => 0x160,
        AudioFormat::WmaV2 => 0x161,
        _ => return Err(AudioError::ConversionError),
    };

    let block_align = audio.block_align().ok_or(AudioError::ConversionError)?;

    let wav_format = WaveFormatEx {
        w_format_tag: format_tag,
        n_channels: audio.channels() as u16,
        n_samples_per_sec: audio.sample_rate(),
        n_avg_bytes_per_sec: audio.byte_rate().unwrap_or(0),
        n_block_align: block_align as u16,
        w_bits_per_sample: audio.bits_per_sample() as u16,
        cb_size: audio.extra().len() as u16,
    };

    // For XMA2 the extra data is the XMA2WAVEFORMATEX extension; for WMA it
    // is the codec-private data (usually empty in xWMA, in which case the
    // decoder fabricates the standard layout).
    let xma2_format = if audio.format() == AudioFormat::Xma2 {
        Some(Xma2WaveFormatEx::from_le_bytes(audio.extra()).ok_or(AudioError::ConversionError)?)
    } else {
        None
    };

    let decoded = xma2_rs::decode_audio(
        &wav_format,
        xma2_format.as_ref(),
        audio.extra(),
        audio.data(),
        audio.frame_count().map(|count| count as usize),
    )?;

    let mut samples = Vec::new();

    samples.try_reserve_exact(decoded.frames() * decoded.channels * size_of::<i16>())?;

    for sample in decoded.to_interleaved_i16() {
        samples.extend_from_slice(&sample.to_le_bytes());
    }

    let mut result = Audio::new(
        decoded.channels as u32,
        decoded.sample_rate,
        16,
        AudioFormat::IntegerPcm,
    )?;

    result.set_data(samples);

    *audio = result;

    Ok(())
}
