//! Bounded JSON file I/O for `.sdlk` documents.
//!
//! This crate preserves the document's JSON structure and deliberately does not
//! apply any score-specific validation or interpretation.

use std::cell::Cell;
use std::fs::File;
use std::io::{self, Read, Write};
use std::path::Path;

use atomicwrites::{AllowOverwrite, AtomicFile};
use serde::de::{DeserializeSeed, MapAccess, SeqAccess, Visitor};
use serde::{Deserialize, Serialize};
use serde_json::{Map, Number, Value};

/// Maximum accepted `.sdlk` input size.
pub const MAX_INPUT_BYTES: usize = 1024 * 1024;
/// Maximum number of nested JSON arrays/objects, including the root container.
pub const MAX_JSON_DEPTH: usize = 64;

/// Failures while reading, parsing, or atomically writing a JSON document.
#[derive(Debug)]
pub enum CodecError {
    Io(io::Error),
    InputTooLarge { limit: usize, at_least: usize },
    JsonDepthExceeded { limit: usize },
    Parse(serde_json::Error),
    Serialize(serde_json::Error),
    AtomicWrite(atomicwrites::Error<io::Error>),
}

impl std::fmt::Display for CodecError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Io(error) => write!(f, "could not read input file: {error}"),
            Self::InputTooLarge { limit, at_least } => write!(
                f,
                "input file is at least {at_least} bytes; limit is {limit} bytes"
            ),
            Self::JsonDepthExceeded { limit } => {
                write!(f, "JSON nesting exceeds the maximum depth of {limit}")
            }
            Self::Parse(error) => write!(f, "invalid JSON document: {error}"),
            Self::Serialize(error) => write!(f, "could not encode JSON document: {error}"),
            Self::AtomicWrite(error) => write!(f, "could not atomically write file: {error}"),
        }
    }
}

impl std::error::Error for CodecError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Io(error) => Some(error),
            Self::Parse(error) | Self::Serialize(error) => Some(error),
            Self::AtomicWrite(error) => Some(error),
            Self::InputTooLarge { .. } | Self::JsonDepthExceeded { .. } => None,
        }
    }
}

/// Read and decode a typed JSON document after applying the input safety limits.
pub fn read_json<T: for<'de> Deserialize<'de>>(path: impl AsRef<Path>) -> Result<T, CodecError> {
    let mut file = File::open(path).map_err(CodecError::Io)?;
    let mut bytes = Vec::new();
    Read::by_ref(&mut file)
        .take((MAX_INPUT_BYTES + 1) as u64)
        .read_to_end(&mut bytes)
        .map_err(CodecError::Io)?;
    decode_json(&bytes)
}

/// Decode typed JSON bytes after applying the input safety limits.
pub fn decode_json<T: for<'de> Deserialize<'de>>(bytes: &[u8]) -> Result<T, CodecError> {
    if bytes.len() > MAX_INPUT_BYTES {
        return Err(CodecError::InputTooLarge {
            limit: MAX_INPUT_BYTES,
            at_least: bytes.len(),
        });
    }

    let depth_exceeded = Cell::new(false);
    let mut deserializer = serde_json::Deserializer::from_slice(bytes);
    let value = DepthSeed {
        depth: 0,
        exceeded: &depth_exceeded,
    }
    .deserialize(&mut deserializer)
    .and_then(|value| {
        deserializer.end()?;
        Ok(value)
    });

    let value = match value {
        Ok(value) => value,
        Err(_) if depth_exceeded.get() => {
            return Err(CodecError::JsonDepthExceeded {
                limit: MAX_JSON_DEPTH,
            });
        }
        Err(error) => return Err(CodecError::Parse(error)),
    };

    serde_json::from_value(value).map_err(CodecError::Parse)
}

/// Encode and atomically replace a JSON file using a sibling temporary file.
pub fn write_json_atomic<T: Serialize>(
    path: impl AsRef<Path>,
    value: &T,
) -> Result<(), CodecError> {
    let bytes = serde_json::to_vec_pretty(value).map_err(CodecError::Serialize)?;
    if bytes.len() > MAX_INPUT_BYTES {
        return Err(CodecError::InputTooLarge {
            limit: MAX_INPUT_BYTES,
            at_least: bytes.len(),
        });
    }
    validate_json_depth(&bytes)?;

    write_atomically(path.as_ref(), |file| file.write_all(&bytes))
}

/// Atomically replace an arbitrary file using a sibling temporary file.
pub fn write_bytes_atomic(path: impl AsRef<Path>, bytes: &[u8]) -> Result<(), CodecError> {
    write_atomically(path.as_ref(), |file| file.write_all(bytes))
}

fn write_atomically(
    path: &Path,
    write: impl FnOnce(&mut File) -> io::Result<()>,
) -> Result<(), CodecError> {
    AtomicFile::new(path, AllowOverwrite)
        .write(write)
        .map_err(CodecError::AtomicWrite)
}

fn validate_json_depth(bytes: &[u8]) -> Result<(), CodecError> {
    let depth_exceeded = Cell::new(false);
    let mut deserializer = serde_json::Deserializer::from_slice(bytes);
    let result = DepthSeed {
        depth: 0,
        exceeded: &depth_exceeded,
    }
    .deserialize(&mut deserializer)
    .and_then(|_| deserializer.end());

    match result {
        Ok(()) => Ok(()),
        Err(_) if depth_exceeded.get() => Err(CodecError::JsonDepthExceeded {
            limit: MAX_JSON_DEPTH,
        }),
        Err(error) => Err(CodecError::Parse(error)),
    }
}

struct DepthSeed<'a> {
    depth: usize,
    exceeded: &'a Cell<bool>,
}

impl<'de> DeserializeSeed<'de> for DepthSeed<'_> {
    type Value = Value;

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_any(DepthVisitor(self))
    }
}

struct DepthVisitor<'a>(DepthSeed<'a>);

impl<'de> Visitor<'de> for DepthVisitor<'_> {
    type Value = Value;

    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("a JSON value")
    }

    fn visit_bool<E>(self, value: bool) -> Result<Self::Value, E> {
        Ok(Value::Bool(value))
    }

    fn visit_i64<E>(self, value: i64) -> Result<Self::Value, E> {
        Ok(Value::Number(Number::from(value)))
    }

    fn visit_u64<E>(self, value: u64) -> Result<Self::Value, E> {
        Ok(Value::Number(Number::from(value)))
    }

    fn visit_f64<E>(self, value: f64) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Number::from_f64(value)
            .map(Value::Number)
            .ok_or_else(|| E::custom("non-finite JSON number"))
    }

    fn visit_str<E>(self, value: &str) -> Result<Self::Value, E> {
        Ok(Value::String(value.to_owned()))
    }

    fn visit_string<E>(self, value: String) -> Result<Self::Value, E> {
        Ok(Value::String(value))
    }

    fn visit_unit<E>(self) -> Result<Self::Value, E> {
        Ok(Value::Null)
    }

    fn visit_none<E>(self) -> Result<Self::Value, E> {
        Ok(Value::Null)
    }

    fn visit_some<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        self.0.deserialize(deserializer)
    }

    fn visit_seq<A>(self, mut sequence: A) -> Result<Self::Value, A::Error>
    where
        A: SeqAccess<'de>,
    {
        let next_depth = self.next_depth::<A::Error>()?;
        let mut values = Vec::new();
        while let Some(value) = sequence.next_element_seed(DepthSeed {
            depth: next_depth,
            exceeded: self.0.exceeded,
        })? {
            values.push(value);
        }
        Ok(Value::Array(values))
    }

    fn visit_map<A>(self, mut object: A) -> Result<Self::Value, A::Error>
    where
        A: MapAccess<'de>,
    {
        let next_depth = self.next_depth::<A::Error>()?;
        let mut values = Map::new();
        while let Some(key) = object.next_key::<String>()? {
            let value = object.next_value_seed(DepthSeed {
                depth: next_depth,
                exceeded: self.0.exceeded,
            })?;
            values.insert(key, value);
        }
        Ok(Value::Object(values))
    }
}

impl DepthVisitor<'_> {
    fn next_depth<E: serde::de::Error>(&self) -> Result<usize, E> {
        if self.0.depth >= MAX_JSON_DEPTH {
            self.0.exceeded.set(true);
            return Err(E::custom("maximum JSON nesting depth exceeded"));
        }
        Ok(self.0.depth + 1)
    }
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::io::Write;
    use std::time::{SystemTime, UNIX_EPOCH};

    use serde::{Deserialize, Serialize};

    use super::{
        CodecError, MAX_INPUT_BYTES, decode_json, read_json, write_atomically, write_bytes_atomic,
        write_json_atomic,
    };

    #[derive(Debug, Deserialize, Serialize, PartialEq)]
    struct FixtureLike {
        title: String,
        extensions: serde_json::Value,
    }

    fn temp_path(label: &str) -> std::path::PathBuf {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock after epoch")
            .as_nanos();
        std::env::temp_dir().join(format!("sandikala-io-{label}-{nonce}.sdlk"))
    }

    #[test]
    fn typed_decode_preserves_unknown_json_extensions() {
        let document = decode_json::<FixtureLike>(
            br#"{"title":"Example","extensions":{"vendor":{"flag":true}}}"#,
        )
        .expect("valid document");
        assert_eq!(document.extensions["vendor"]["flag"], true);
    }

    #[test]
    fn byte_limit_is_checked_before_json_parsing() {
        let bytes = vec![b' '; MAX_INPUT_BYTES + 1];
        assert!(matches!(
            decode_json::<serde_json::Value>(&bytes),
            Err(CodecError::InputTooLarge { .. })
        ));
    }

    #[test]
    fn depth_limit_accepts_64_and_rejects_65_containers() {
        let within_limit = format!("{}0{}", "[".repeat(64), "]".repeat(64));
        decode_json::<serde_json::Value>(within_limit.as_bytes()).expect("depth 64 accepted");

        let over_limit = format!("{}0{}", "[".repeat(65), "]".repeat(65));
        assert!(matches!(
            decode_json::<serde_json::Value>(over_limit.as_bytes()),
            Err(CodecError::JsonDepthExceeded { .. })
        ));
    }

    #[test]
    fn atomic_write_replaces_and_reads_a_typed_document() {
        let path = temp_path("roundtrip");
        let value = FixtureLike {
            title: "A short score".to_owned(),
            extensions: serde_json::json!({"vendor": [1, 2, 3]}),
        };

        write_json_atomic(&path, &value).expect("atomic write");
        let decoded: FixtureLike = read_json(&path).expect("typed read");
        assert_eq!(decoded, value);

        let replacement = FixtureLike {
            title: "Replaced".to_owned(),
            extensions: serde_json::Value::Null,
        };
        write_json_atomic(&path, &replacement).expect("atomic overwrite");
        assert_eq!(
            read_json::<FixtureLike>(&path).expect("re-read"),
            replacement
        );
        fs::remove_file(path).expect("remove test file");
    }

    #[test]
    fn atomic_byte_write_replaces_existing_file_content() {
        let path = temp_path("bytes");
        write_bytes_atomic(&path, b"first export").expect("first write");
        write_bytes_atomic(&path, b"replacement export").expect("atomic overwrite");
        assert_eq!(fs::read(&path).expect("read export"), b"replacement export");
        fs::remove_file(path).expect("remove test file");
    }

    #[test]
    fn failed_atomic_write_preserves_existing_destination() {
        let path = temp_path("failed-write");
        fs::write(&path, b"original document").expect("create original file");

        let error = write_atomically(&path, |file| {
            file.write_all(b"partial replacement")?;
            Err(std::io::Error::other("injected write failure"))
        })
        .expect_err("injected failure should abort replacement");

        assert!(matches!(error, CodecError::AtomicWrite(_)));
        assert_eq!(
            fs::read(&path).expect("read preserved destination"),
            b"original document"
        );
        fs::remove_file(path).expect("remove test file");
    }
}
