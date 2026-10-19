//! Screenshots of the page, as the browser shows it.

use webqa_core::judge::{Image, ImageFormat};
use webqa_core::ports::BrowserError;

use crate::launch::PlaywrightBrowser;

impl PlaywrightBrowser {
    /// Takes a PNG screenshot of the page. A failure carries none of
    /// Playwright's text.
    pub(crate) async fn capture(&self) -> Result<Image, BrowserError> {
        self.page
            .screenshot(None)
            .await
            .map(png)
            .or(Err(BrowserError))
    }
}

/// The PNG image made of `bytes`.
const fn png(bytes: Vec<u8>) -> Image {
    Image::new(ImageFormat::Png, bytes)
}
