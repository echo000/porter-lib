use std::io::Cursor;
use std::io::Read;
use std::io::Seek;
use std::io::SeekFrom;
use std::io::Write;

use porter_utils::StackVec;
use porter_utils::StructReadExt;
use porter_utils::StructWriteExt;

use crate::is_format_srgb;
use crate::Image;
use crate::ImageFileType;
use crate::ImageFormat;
use crate::TextureError;

/// Maximum number of tga frames to expand.
const MAXIMUM_TGA_FRAMES: usize = 6;
/// Maximum run-length chunk size.
const MAXIMUM_RLE_LENGTH: usize = 128;
/// The maximum bytes per pixel for a tga.
const MAXIMUM_BYTES_PER_PIXEL: usize = 4;
/// The maximum run-length buffer size.
const MAXIMUM_RLE_BUFFER: usize = MAXIMUM_BYTES_PER_PIXEL * MAXIMUM_RLE_LENGTH;

#[repr(u8)]
#[derive(Debug, Clone, Copy)]
enum ImageType {
    UncompressedRgb = 2,
    UncompressedGrayscale = 3,
    CompressedRgb = 10,
    CompressedGrayscale = 11,
}

#[derive(Debug, Clone, Copy)]
enum ColorType {
    Grayscale,
    Rgba,
}

#[repr(C, packed)]
#[derive(Debug, Clone, Copy)]
struct TgaHeader {
    id_size: u8,
    color_type: u8,
    image_type: u8,
    color_map_origin: u16,
    color_map_length: u16,
    color_map_depth: u8,
    x_origin: u16,
    y_origin: u16,
    width: u16,
    height: u16,
    bits_per_pixel: u8,
    image_descriptor: u8,
}

/// Converts an image format to a tga specification.
const fn format_to_tga(format: ImageFormat) -> Result<(ColorType, ImageType, u8), TextureError> {
    Ok(match format {
        ImageFormat::R8Unorm => (ColorType::Grayscale, ImageType::CompressedGrayscale, 8),
        ImageFormat::B8G8R8A8Unorm => (ColorType::Rgba, ImageType::CompressedRgb, 32),
        ImageFormat::B8G8R8A8UnormSrgb => (ColorType::Rgba, ImageType::CompressedRgb, 32),
        _ => {
            return Err(TextureError::ContainerFormatInvalid(
                format,
                ImageFileType::Tga,
            ))
        }
    })
}

/// Picks the proper format required to save the input format to a tga file type.
pub const fn pick_format(format: ImageFormat) -> ImageFormat {
    match format {
        // Grayscale 1bit.
        ImageFormat::R1Unorm => ImageFormat::R8Unorm,

        // Grayscale 8bit.
        ImageFormat::R8Typeless
        | ImageFormat::R8Unorm
        | ImageFormat::R8Sint
        | ImageFormat::R8Uint => ImageFormat::R8Unorm,

        // Grayscale 16bit.
        ImageFormat::R16Typeless
        | ImageFormat::R16Float
        | ImageFormat::R16Unorm
        | ImageFormat::R16Snorm
        | ImageFormat::R16Sint
        | ImageFormat::R16Uint => ImageFormat::R8Unorm,

        // Red compressed Bc4.
        ImageFormat::Bc4Typeless | ImageFormat::Bc4Unorm | ImageFormat::Bc4Snorm => {
            ImageFormat::R8Unorm
        }

        // Various compressed formats.
        _ => {
            if is_format_srgb(format) {
                ImageFormat::B8G8R8A8UnormSrgb
            } else {
                ImageFormat::B8G8R8A8Unorm
            }
        }
    }
}

/// Writes an image to a tga file to the output stream.
pub fn to_tga<O: Write + Seek>(image: &Image, output: &mut O) -> Result<(), TextureError> {
    let (color_type, image_type, bit_depth) = format_to_tga(image.format())?;

    let frames = image.frames().len();
    let width = image.width();
    let height = image.height() * frames.min(MAXIMUM_TGA_FRAMES) as u32;

    if width > u16::MAX as u32 || height > u16::MAX as u32 {
        return Err(TextureError::InvalidImageSize(width, height));
    }

    let header = TgaHeader {
        id_size: 0,
        color_type: 0,
        image_type: image_type as u8,
        color_map_origin: 0,
        color_map_length: 0,
        color_map_depth: 0,
        x_origin: 0,
        y_origin: 0,
        width: width as u16,
        height: height as u16,
        bits_per_pixel: bit_depth,
        image_descriptor: 32,
    };

    output.write_struct(header)?;

    let frame_width = image.width() as usize;
    let frame_height = image.height() as usize;

    for frame in image.frames().take(MAXIMUM_TGA_FRAMES) {
        let buf = frame.buffer();

        match color_type {
            ColorType::Grayscale => {
                const BYTES_PER_PIXEL: usize = 1;

                for y in 0..frame_height {
                    let row_start = y * frame_width * BYTES_PER_PIXEL;
                    let row_end = row_start + frame_width * BYTES_PER_PIXEL;

                    write_rle_encode::<BYTES_PER_PIXEL, _>(&buf[row_start..row_end], output)?;
                }
            }
            ColorType::Rgba => {
                const BYTES_PER_PIXEL: usize = 4;

                for y in 0..frame_height {
                    let row_start = y * frame_width * BYTES_PER_PIXEL;
                    let row_end = row_start + frame_width * BYTES_PER_PIXEL;

                    write_rle_encode::<BYTES_PER_PIXEL, _>(&buf[row_start..row_end], output)?;
                }
            }
        }
    }

    Ok(())
}

/// Reads a tga file from the input stream to an image.
pub fn from_tga<I: Read + Seek>(input: &mut I) -> Result<Image, TextureError> {
    let header: TgaHeader = input.read_struct()?;

    input.seek(SeekFrom::Current(header.id_size as i64))?;

    if header.color_type != 0 {
        return Err(TextureError::ContainerInvalid(ImageFileType::Tga));
    }

    if header.x_origin != 0 || header.y_origin != 0 {
        return Err(TextureError::ContainerInvalid(ImageFileType::Tga));
    }

    let format = match header.bits_per_pixel {
        8 => ImageFormat::R8Unorm,
        32 => ImageFormat::B8G8R8A8Unorm,
        _ => return Err(TextureError::ContainerInvalid(ImageFileType::Tga)),
    };

    let mut image = Image::new(header.width as u32, header.height as u32, format)?;
    let frame = image.create_frame()?;

    match header.image_type {
        x if x == ImageType::UncompressedRgb as u8 => {
            input.read_exact(frame.buffer_mut())?;
        }
        x if x == ImageType::UncompressedGrayscale as u8 => {
            input.read_exact(frame.buffer_mut())?;
        }
        x if x == ImageType::CompressedRgb as u8 => {
            read_rle_decode::<4, _>(frame.buffer_mut(), input)?;
        }
        x if x == ImageType::CompressedGrayscale as u8 => {
            read_rle_decode::<1, _>(frame.buffer_mut(), input)?;
        }
        _ => return Err(TextureError::ContainerInvalid(ImageFileType::Tga)),
    }

    Ok(image)
}

/// Utility method to read a run-length frame and decode it.
fn read_rle_decode<const BYTES_PER_PIXEL: usize, I: Read + Seek>(
    buffer: &mut [u8],
    input: &mut I,
) -> Result<(), TextureError> {
    let length = buffer.len() as u64;

    let mut writer = Cursor::new(buffer);

    while writer.position() < length {
        let opcode: u8 = input.read_struct()?;

        if (opcode & 0x80) != 0 {
            let len = ((opcode & !0x80) + 1) as usize;
            let pixel: [u8; BYTES_PER_PIXEL] = input.read_struct()?;

            for _ in 0..len {
                writer.write_all(&pixel)?;
            }
        } else {
            let len = (opcode + 1) as u64 * BYTES_PER_PIXEL as u64;

            std::io::copy(&mut input.take(len), &mut writer)?;
        }
    }

    Ok(())
}

/// Utility method to write a frame run-length encoded.
fn write_rle_encode<const BYTES_PER_PIXEL: usize, O: Write + Seek>(
    buffer: &[u8],
    output: &mut O,
) -> Result<(), TextureError> {
    let mut cursor = 0;

    while cursor < buffer.len() {
        let chunk_start = cursor;
        let is_rle_packet = (cursor + BYTES_PER_PIXEL < buffer.len())
            && buffer[cursor..cursor + BYTES_PER_PIXEL]
                == buffer[cursor + BYTES_PER_PIXEL..cursor + 2 * BYTES_PER_PIXEL];

        let mut packet_size = 1;
        cursor += BYTES_PER_PIXEL;

        if is_rle_packet {
            while packet_size < MAXIMUM_RLE_LENGTH
                && cursor + BYTES_PER_PIXEL <= buffer.len()
                && buffer[chunk_start..chunk_start + BYTES_PER_PIXEL]
                    == buffer[cursor..cursor + BYTES_PER_PIXEL]
            {
                packet_size += 1;
                cursor += BYTES_PER_PIXEL;
            }

            write_rle(
                &buffer[chunk_start..chunk_start + BYTES_PER_PIXEL],
                packet_size as u8,
                output,
            )?;
        } else {
            while packet_size < MAXIMUM_RLE_LENGTH
                && cursor + BYTES_PER_PIXEL <= buffer.len()
                && (cursor + BYTES_PER_PIXEL >= buffer.len()
                    || buffer[cursor..cursor + BYTES_PER_PIXEL]
                        != buffer[cursor + BYTES_PER_PIXEL..cursor + 2 * BYTES_PER_PIXEL])
            {
                packet_size += 1;
                cursor += BYTES_PER_PIXEL;
            }

            write_raw(
                &buffer[chunk_start..chunk_start + packet_size * BYTES_PER_PIXEL],
                packet_size as u8,
                output,
            )?;
        }
    }

    Ok(())
}

/// Utility method to write a raw opcode.
#[inline]
fn write_raw<O: Write + Seek>(
    buffer: &[u8],
    counter: u8,
    output: &mut O,
) -> Result<(), TextureError> {
    output.write_struct(counter - 1)?;
    output.write_all(buffer)?;

    Ok(())
}

/// Utility method to write a run-length opcode.
#[inline]
fn write_rle<O: Write + Seek>(
    buffer: &[u8],
    counter: u8,
    output: &mut O,
) -> Result<(), TextureError> {
    output.write_struct(0x80 | (counter - 1))?;
    output.write_all(buffer)?;

    Ok(())
}
