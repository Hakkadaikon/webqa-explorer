use super::*;

#[test]
fn declaration_line_names_version_and_images() {
    let cases: [(&[u8], Result<bool, StartError>); 15] = [
        (br#"{"version":2,"images":true}"#, Ok(true)),
        (br#"{"version":2,"images":false}"#, Ok(false)),
        (
            br#"{"images":true,"model":"typesafe-ai/jev-1.14.0","version":2}"#,
            Ok(true),
        ),
        (
            br#"{"version":2,"seq":1,"images":false,"answers":{}}"#,
            Ok(false),
        ),
        (br#"{"version":1,"images":true}"#, Err(StartError::Version)),
        (br#"{"version":3,"images":true}"#, Err(StartError::Version)),
        (
            br#"{"version":1,"seq":1,"answers":{}}"#,
            Err(StartError::Version),
        ),
        (br#"{"version":2}"#, Err(StartError::Declaration)),
        (
            br#"{"version":2,"images":"true"}"#,
            Err(StartError::Declaration),
        ),
        (
            br#"{"version":2,"images":null}"#,
            Err(StartError::Declaration),
        ),
        (br#"{"images":true}"#, Err(StartError::Declaration)),
        (
            br#"{"version":"2","images":true}"#,
            Err(StartError::Declaration),
        ),
        (b"not json", Err(StartError::Declaration)),
        (b"", Err(StartError::Declaration)),
        (&[0xFF], Err(StartError::Declaration)),
    ];
    for (line, want) in cases {
        let shown = String::from_utf8_lossy(line);
        assert_eq!(read(line), want, "{shown}");
    }
}
