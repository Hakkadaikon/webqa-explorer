use super::*;

fn png4() -> Image {
    Image::new(ImageFormat::Png, vec![0x89, b'P', b'N', b'G'])
}

fn empty() -> Image {
    Image::new(ImageFormat::Png, Vec::new())
}

#[test]
fn image_formats_are_media_types() {
    let names = [ImageFormat::Png, ImageFormat::Jpeg].map(|f| serde_json::to_value(f).unwrap());
    assert_eq!(
        names,
        [
            serde_json::json!("image/png"),
            serde_json::json!("image/jpeg")
        ],
        "the media types"
    );
    assert_eq!(
        png4().format(),
        ImageFormat::Png,
        "the format it was built with"
    );
    assert_eq!(
        png4().bytes(),
        [0x89, b'P', b'N', b'G'],
        "the bytes it was built with"
    );
    assert_eq!(empty().bytes(), [0u8; 0], "no bytes");
}

#[test]
fn image_debug_hides_bytes() {
    let shown = format!(
        "{:?}",
        Image::new(ImageFormat::Png, b"secret-pixels".to_vec())
    );
    assert_eq!(shown, "Image { format: Png, bytes: 13 }", "the length only");
    assert!(!shown.contains("secret"), "no bytes in {shown}");
    assert_eq!(
        format!("{:?}", empty()),
        "Image { format: Png, bytes: 0 }",
        "an empty image"
    );
}
