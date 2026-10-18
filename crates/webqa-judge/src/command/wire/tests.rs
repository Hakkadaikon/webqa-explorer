use serde_json::{Value, json};
use webqa_core::judge::{Image, ImageFormat};

use super::*;

fn png4() -> Image {
    Image::new(ImageFormat::Png, vec![0x89, b'P', b'N', b'G'])
}

fn jpeg3() -> Image {
    Image::new(ImageFormat::Jpeg, vec![0xFF, 0xD8, 0xFF])
}

fn empty() -> Image {
    Image::new(ImageFormat::Png, Vec::new())
}

fn expectation() -> Question {
    Question::Text {
        id: QuestionId::new("expectation"),
        prompt: "期待".to_owned(),
    }
}

fn value(line: &str) -> Value {
    serde_json::from_str(line).unwrap()
}

/// The `image` object of the request line with `image`.
fn image_field(image: &Image) -> Value {
    value(&request(1, "s", Some(image), &[]))["image"].clone()
}

/// The keys of an object, sorted.
fn keys(object: &Value) -> Vec<String> {
    object.as_object().unwrap().keys().cloned().collect()
}

/// Whether the line names a field of a particular judge model.
fn names_a_model(line: &str) -> bool {
    ["assets", "data_base64", "format", "TypeSafe"]
        .iter()
        .any(|name| line.contains(name))
}

/// The line with `PNG4`, as written. Returns it.
fn png_line() -> String {
    let line = request(1, "s", Some(&png4()), &[]);
    let want = json!({"version": 2, "seq": 1, "screen": "s",
        "image": {"media_type": "image/png", "data": "iVBORw=="}, "questions": []});
    assert_eq!(value(&line), want, "the media type and the base64");
    let head = line.starts_with(r#"{"version":2,"seq":1,"screen":"s","image":{"#);
    assert!(head, "in field order: {line}");
    assert_eq!(line.matches('\n').count(), 1, "one newline");
    assert!(line.ends_with('\n'), "at the end");
    line
}

/// The image objects of each format, and of no bytes.
fn image_fields() {
    let png = keys(&image_field(&png4()));
    assert_eq!(png, ["data", "media_type"], "two keys");
    let jpeg = json!({"media_type": "image/jpeg", "data": "/9j/"});
    assert_eq!(image_field(&jpeg3()), jpeg, "a JPEG");
    let none = json!({"media_type": "image/png", "data": ""});
    assert_eq!(image_field(&empty()), none, "no bytes, empty data");
}

/// The line with a long image, as written. Returns it.
fn long_line() -> String {
    let image = Image::new(ImageFormat::Png, vec![0; 1200]);
    let line = request(1, "s", Some(&image), &[]);
    assert_eq!(line.matches('\n').count(), 1, "the base64 is not wrapped");
    let data = value(&line)["image"]["data"].as_str().map(str::len);
    assert_eq!(data, Some(1600), "4 characters for each 3 bytes");
    line
}

/// The line without an image, as written. Returns it.
fn plain_line() -> String {
    let line = request(1, "s", None, &[]);
    let want = json!({"version": 2, "seq": 1, "screen": "s", "questions": []});
    assert_eq!(value(&line), want, "no image, no key");
    assert!(!line.contains("\"image\""), "not even null: {line}");
    line
}

#[test]
fn image_field_shape_is_model_neutral() {
    image_fields();
    for line in [png_line(), long_line(), plain_line()] {
        assert!(!names_a_model(&line), "no model's names in {line}");
    }
    let text = request(1, "s", None, &[expectation()]);
    let want = json!([{"kind": "text", "id": "expectation", "prompt": "期待"}]);
    assert_eq!(value(&text)["questions"], want, "the free-text question");
}
