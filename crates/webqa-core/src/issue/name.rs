//! Accessible names that are image file names or placeholders.

/// Extensions of image files, lowercase.
const IMAGE_EXTENSIONS: [&str; 9] = [
    "png", "jpg", "jpeg", "gif", "svg", "webp", "avif", "bmp", "ico",
];

/// Placeholder values, lowercase.
const PLACEHOLDERS: [&str; 18] = [
    "button",
    "link",
    "image",
    "img",
    "icon",
    "graphic",
    "untitled",
    "placeholder",
    "todo",
    "undefined",
    "null",
    "nan",
    "[object object]",
    "lorem ipsum",
    "ボタン",
    "リンク",
    "画像",
    "アイコン",
];

/// Whether `name` is an image file name or a placeholder value.
///
/// Surrounding whitespace is ignored and ASCII letters compare case-insensitively.
/// An empty name is neither.
#[must_use]
pub fn is_placeholder_name(name: &str) -> bool {
    let name = name.trim().to_ascii_lowercase();
    is_image_file(&name) || is_placeholder(&name)
}

/// A non-empty stem followed by an image extension after the last `.`.
fn is_image_file(name: &str) -> bool {
    name.rsplit_once('.')
        .is_some_and(|(stem, ext)| !stem.is_empty() && is_image_extension(ext))
}

fn is_image_extension(ext: &str) -> bool {
    IMAGE_EXTENSIONS.contains(&ext)
}

fn is_placeholder(name: &str) -> bool {
    PLACEHOLDERS.contains(&name)
}
