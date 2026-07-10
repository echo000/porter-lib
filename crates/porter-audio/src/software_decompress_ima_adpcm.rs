use std::io::Cursor;
use std::io::Read;

use porter_utils::StructReadExt;
use porter_utils::StructWriteExt;

use crate::Audio;
use crate::AudioError;
use crate::AudioFormat;

/// The step table for ImaAdpcm.
const IMA_ADPCM_STEP_TABLE: [i32; 89] = [
    7, 8, 9, 10, 11, 12, 13, 14, 16, 17, 19, 21, 23, 25, 28, 31, 34, 37, 41, 45, 50, 55, 60, 66,
    73, 80, 88, 97, 107, 118, 130, 143, 157, 173, 190, 209, 230, 253, 279, 307, 337, 371, 408, 449,
    494, 544, 598, 658, 724, 796, 876, 963, 1060, 1166, 1282, 1411, 1552, 1707, 1878, 2066, 2272,
    2499, 2749, 3024, 3327, 3660, 4026, 4428, 4871, 5358, 5894, 6484, 7132, 7845, 8630, 9493,
    10442, 11487, 12635, 13899, 15289, 16818, 18500, 20350, 22385, 24623, 27086, 29794, 32767,
];
/// The step index table for ImaAdpcm.
const IMA_ADPCM_INDEX_TABLE: [i32; 16] = [-1, -1, -1, -1, 2, 4, 6, 8, -1, -1, -1, -1, 2, 4, 6, 8];

#[repr(C, packed)]
#[derive(Debug, Clone, Copy)]
struct ImaAdpcmChHeader {
    predictor: i16,
    step_index: u8,
    reserved: u8,
}

/// Unpacks a nibble to lower, upper parts.
#[inline(always)]
fn unpack_nibble(nibble: u8) -> (u8, u8) {
    (nibble & 0xF, (nibble & 0xF0) >> 4)
}

/// Clamps a i32 to a i16 value.
#[inline(always)]
fn clamp_i16(value: i32) -> i16 {
    if value.wrapping_add(0x8000) & !0xFFFF == 0 {
        value as i16
    } else {
        0x7FFF ^ value.wrapping_shr(31) as i16
    }
}

/// Decompresses one nibble and adjusts the state.
#[inline(always)]
fn decompress_nibble(predictor: &mut i32, step_index: &mut i32, nibble: u8) -> i16 {
    let step = IMA_ADPCM_STEP_TABLE[*step_index as usize];

    let mut diff = step >> 3;

    if (nibble & 0x1) != 0 {
        diff += step >> 2;
    }
    if (nibble & 0x2) != 0 {
        diff += step >> 1;
    }
    if (nibble & 0x4) != 0 {
        diff += step;
    }
    if (nibble & 0x8) != 0 {
        *predictor -= diff;
    } else {
        *predictor += diff;
    }

    *predictor = clamp_i16(*predictor) as i32;
    *step_index = (*step_index + IMA_ADPCM_INDEX_TABLE[(nibble & 0xF) as usize]).clamp(0, 88);

    *predictor as i16
}

/// Decompress ImaAdpcm to 16bit IntegerPcm.
pub fn decompress_ima_adpcm(audio: &mut Audio) -> Result<(), AudioError> {
    let block_align = audio.block_align().ok_or(AudioError::ConversionError)? as usize;
    let channels = audio.channels();

    let buffer: Vec<u8> = Vec::new();
    let mut buffer = Cursor::new(buffer);

    for block in audio.data().chunks_exact(block_align) {
        let mut block = Cursor::new(block);

        if channels == 1 {
            let header: ImaAdpcmChHeader = block.read_struct()?;

            if header.step_index as usize >= IMA_ADPCM_STEP_TABLE.len() {
                #[cfg(debug_assertions)]
                println!("ImaAdpcm invalid step index: {:#02x?}", {
                    header.step_index
                });
                return Err(AudioError::ConversionError);
            }

            buffer.write_struct(header.predictor)?;

            let nibbles = block_align - size_of::<ImaAdpcmChHeader>();

            let mut predictor = header.predictor as i32;
            let mut step_index = header.step_index as i32;

            for _ in 0..nibbles {
                let nibble: u8 = block.read_struct()?;
                let (lower, upper) = unpack_nibble(nibble);

                for nibble in [lower, upper] {
                    let sample = decompress_nibble(&mut predictor, &mut step_index, nibble);

                    buffer.write_struct(sample)?;
                }
            }
        } else if channels == 2 {
            let l_header: ImaAdpcmChHeader = block.read_struct()?;
            let r_header: ImaAdpcmChHeader = block.read_struct()?;

            if l_header.step_index as usize >= IMA_ADPCM_STEP_TABLE.len()
                || r_header.step_index as usize >= IMA_ADPCM_STEP_TABLE.len()
            {
                #[cfg(debug_assertions)]
                println!(
                    "ImaAdpcm invalid step indexes: {:#02x?} {:#02x?}",
                    { l_header.step_index },
                    { r_header.step_index }
                );
                return Err(AudioError::ConversionError);
            }

            buffer.write_struct(l_header.predictor)?;
            buffer.write_struct(r_header.predictor)?;

            let groups = (block_align - (size_of::<ImaAdpcmChHeader>() * 2)) / 8;

            let mut l_predictor = l_header.predictor as i32;
            let mut r_predictor = r_header.predictor as i32;
            let mut l_step_index = l_header.step_index as i32;
            let mut r_step_index = r_header.step_index as i32;

            for _ in 0..groups {
                let mut l_bytes = [0u8; 4];
                let mut r_bytes = [0u8; 4];

                block.read_exact(&mut l_bytes)?;
                block.read_exact(&mut r_bytes)?;

                let mut l_samples = [0i16; 8];
                let mut r_samples = [0i16; 8];

                for (index, (l_byte, r_byte)) in l_bytes.into_iter().zip(r_bytes).enumerate() {
                    let (l_lower, l_upper) = unpack_nibble(l_byte);
                    let (r_lower, r_upper) = unpack_nibble(r_byte);

                    l_samples[index * 2] =
                        decompress_nibble(&mut l_predictor, &mut l_step_index, l_lower);
                    l_samples[index * 2 + 1] =
                        decompress_nibble(&mut l_predictor, &mut l_step_index, l_upper);

                    r_samples[index * 2] =
                        decompress_nibble(&mut r_predictor, &mut r_step_index, r_lower);
                    r_samples[index * 2 + 1] =
                        decompress_nibble(&mut r_predictor, &mut r_step_index, r_upper);
                }

                for (l_sample, r_sample) in l_samples.into_iter().zip(r_samples) {
                    buffer.write_struct(l_sample)?;
                    buffer.write_struct(r_sample)?;
                }
            }
        } else {
            #[cfg(debug_assertions)]
            println!("ImaAdpcm invalid channels: {}", channels);
            return Err(AudioError::ConversionError);
        }
    }

    let mut result = Audio::new(channels, audio.sample_rate(), 16, AudioFormat::IntegerPcm)?;

    result.set_data(buffer.into_inner());

    *audio = result;

    Ok(())
}
