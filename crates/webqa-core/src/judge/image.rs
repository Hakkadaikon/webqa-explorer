//! The screenshot a judge that reads images is asked about.

use std::fmt;

use serde::Serialize;

/// The format of an image, written as its media type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum ImageFormat {
    /// PNG.
    #[serde(rename = "image/png")]
    Png,
    /// JPEG.
    #[serde(rename = "image/jpeg")]
    Jpeg,
}

/// A screenshot of the page, as the browser took it. It is never redacted,
/// so its `Debug` shows the format and the length, not the bytes.
#[derive(Clone, PartialEq, Eq)]
pub struct Image {
    format: ImageFormat,
    bytes: Vec<u8>,
}

impl Image {
    /// An image of `format` made of `bytes`, which are not checked.
    #[must_use]
    pub const fn new(format: ImageFormat, bytes: Vec<u8>) -> Self {
        Self { format, bytes }
    }

    /// The format of the image.
    #[must_use]
    pub const fn format(&self) -> ImageFormat {
        self.format
    }

    /// The bytes of the image, as encoded in its format.
    #[must_use]
    pub fn bytes(&self) -> &[u8] {
        &self.bytes
    }
}

impl fmt::Debug for Image {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Image")
            .field("format", &self.format)
            .field("bytes", &self.bytes.len())
            .finish()
    }
}

#[cfg(test)]
mod tests;
