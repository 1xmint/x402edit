#![forbid(unsafe_code)]
use image::{DynamicImage, ImageFormat, ImageReader};
use std::io::Cursor;
pub const MAX_FILE_BYTES: usize = 10 * 1024 * 1024;
pub const MAX_PIXELS: u64 = 40_000_000;
#[derive(Debug, thiserror::Error)]
pub enum MediaError {
    #[error("file exceeds 10 MiB")]
    TooLarge,
    #[error("unsupported or malformed static image")]
    Invalid,
    #[error("decoded image exceeds 40 megapixels")]
    TooManyPixels,
}
pub fn decode_static(bytes: &[u8]) -> Result<DynamicImage, MediaError> {
    if bytes.len() > MAX_FILE_BYTES {
        return Err(MediaError::TooLarge);
    }
    let format = image::guess_format(bytes).map_err(|_| MediaError::Invalid)?;
    if !matches!(
        format,
        ImageFormat::Png | ImageFormat::Jpeg | ImageFormat::WebP
    ) {
        return Err(MediaError::Invalid);
    }
    let image = ImageReader::with_format(Cursor::new(bytes), format)
        .decode()
        .map_err(|_| MediaError::Invalid)?;
    if u64::from(image.width()) * u64::from(image.height()) > MAX_PIXELS {
        return Err(MediaError::TooManyPixels);
    }
    Ok(image)
}
