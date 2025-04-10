use redis_protocol::resp3::types::OwnedFrame;
use std::collections::HashMap;

use redis_protocol::error::RedisProtocolError;
#[cfg(feature = "serde")]
use redis_protocol::resp3::{decode, encode, types::Resp3Frame};
#[cfg(feature = "serde")]
use serde::de::{Error, Visitor};
#[cfg(feature = "serde")]
use serde::{Deserialize, Deserializer, Serialize, Serializer};
#[cfg(feature = "serde")]
use std::fmt::Formatter;

#[cfg(feature = "serde")]
/// Wrapper around [`OwnedFrame`] that supports serialization and
/// deserialization using [`serde`]
pub struct SerializableFrame(pub OwnedFrame);

#[cfg(feature = "serde")]
impl Serialize for SerializableFrame {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut encoded: Vec<u8> = vec![0u8; self.0.encode_len(false)];
        encode::complete::encode(&mut encoded, &self.0, false).expect("Failed to encode");
        serializer.serialize_bytes(&*encoded)
    }
}

#[cfg(feature = "serde")]
struct FrameVisitor;

#[cfg(feature = "serde")]
impl<'de> Visitor<'de> for FrameVisitor {
    type Value = SerializableFrame;

    fn expecting(&self, formatter: &mut Formatter) -> std::fmt::Result {
        formatter.write_str("a redis OwnedFrame encoded by redis_protocol::resp3::encode::complete")
    }

    fn visit_bytes<E>(self, v: &[u8]) -> Result<Self::Value, E>
    where
        E: Error,
    {
        let bytes = decode::complete::decode(&v);
        if let Ok(Some((frame, _))) = bytes {
            Ok(SerializableFrame(frame))
        } else {
            Err(E::custom("Failed to deserialize"))
        }
    }
}

#[cfg(feature = "serde")]
impl<'de> Deserialize<'de> for SerializableFrame {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_bytes(FrameVisitor)
    }
}

#[cfg(feature = "serde")]
impl From<OwnedFrame> for SerializableFrame {
    fn from(frame: OwnedFrame) -> Self {
        SerializableFrame(frame)
    }
}

#[cfg(feature = "serde")]
impl From<SerializableFrame> for OwnedFrame {
    fn from(frame: SerializableFrame) -> Self {
        frame.0
    }
}

pub trait AsFrame {
    fn as_frame(&self) -> OwnedFrame;
}

#[cfg(feature = "serde")]
impl AsFrame for SerializableFrame {
    fn as_frame(&self) -> OwnedFrame {
        self.0.clone()
    }
}

impl AsFrame for i64 {
    fn as_frame(&self) -> OwnedFrame {
        OwnedFrame::Number {
            data: *self,
            attributes: None,
        }
    }
}

impl AsFrame for i32 {
    fn as_frame(&self) -> OwnedFrame {
        OwnedFrame::Number {
            data: *self as i64,
            attributes: None,
        }
    }
}

impl AsFrame for usize {
    fn as_frame(&self) -> OwnedFrame {
        OwnedFrame::BigNumber {
            data: self.to_ne_bytes().to_vec(),
            attributes: None,
        }
    }
}

impl AsFrame for u16 {
    fn as_frame(&self) -> OwnedFrame {
        OwnedFrame::Number {
            data: *self as i64,
            attributes: None,
        }
    }
}

impl AsFrame for String {
    fn as_frame(&self) -> OwnedFrame {
        OwnedFrame::BlobString {
            data: self.clone().into(),
            attributes: None,
        }
    }
}

impl AsFrame for &String {
    fn as_frame(&self) -> OwnedFrame {
        self.as_str().as_frame()
    }
}

impl AsFrame for str {
    fn as_frame(&self) -> OwnedFrame {
        OwnedFrame::BlobString {
            data: Vec::from(self),
            attributes: None,
        }
    }
}

impl AsFrame for &str {
    fn as_frame(&self) -> OwnedFrame {
        (*self).as_frame()
    }
}

impl AsFrame for RedisProtocolError {
    fn as_frame(&self) -> OwnedFrame {
        OwnedFrame::SimpleError {
            data: self.to_string(),
            attributes: None,
        }
    }
}

impl AsFrame for OwnedFrame {
    fn as_frame(&self) -> OwnedFrame {
        self.clone()
    }
}

impl<K, V> AsFrame for HashMap<K, V>
where
    K: AsFrame,
    V: AsFrame,
{
    fn as_frame(&self) -> OwnedFrame {
        let mut framemap: HashMap<OwnedFrame, OwnedFrame> = HashMap::new();
        for (key, value) in self {
            framemap.insert(key.as_frame(), value.as_frame());
        }
        OwnedFrame::Map {
            data: framemap,
            attributes: None,
        }
    }
}

impl<V: AsFrame> AsFrame for Vec<V> {
    fn as_frame(&self) -> OwnedFrame {
        let mut framevec: Vec<OwnedFrame> = Vec::new();
        for v in self {
            framevec.push(v.as_frame());
        }

        OwnedFrame::Array {
            data: framevec,
            attributes: None,
        }
    }
}

/// Convert to flattened list of tuple contents.
impl<T: AsFrame + Clone> AsFrame for Vec<(T, T)> {
    fn as_frame(&self) -> OwnedFrame {
        let mut intermediate: Vec<T> = Vec::new();
        for (a, b) in self {
            intermediate.push(a.clone());
            intermediate.push(b.clone());
        }

        intermediate.as_frame()
    }
}

pub fn map_to_array<T: AsFrame + Clone>(map: HashMap<T, T>) -> OwnedFrame {
    let arr: Vec<(T, T)> = map.into_iter().collect();
    arr.as_frame()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[cfg(feature = "serde")]
    fn test_serialize_deserialize() {
        // Create an array frame
        let frame = OwnedFrame::SimpleString {
            data: "Hello World".into(),
            attributes: None,
        };

        let serializable = SerializableFrame(frame.clone());

        let serialized =
            bincode::serialize(&serializable).expect("Failed to serialize SerializableFrame");

        let deserialized: SerializableFrame =
            bincode::deserialize(&serialized).expect("Failed to deserialize SerializableFrame");

        // Check
        assert_eq!(deserialized.0, frame);
    }
}

/*
    [NOTE]
    This has been removed to prevent issues when trying to convert lists (Vectors)
    of (Section, Description) pairs to OwnedFrames.

    Converting Vec<(A, B)> resulted in outputs similar to
    ```
     Array {
         Array {
             BlobString,
             BlobString,
         },
         Array {
             BlobString,
             BlobString,
         },
     }
    ```
    when typically a flattened version was desired:
    ```
     Array {
         BlobString,
         BlobString,
         BlobString,
         BlobString,
     }
    ```

impl<A, B> AsFrame for (A, B)
where
    A: AsFrame,
    B: AsFrame
{
    fn as_frame(&self) -> OwnedFrame {
        OwnedFrame::Array {
            data: vec![
                self.0.as_frame(),
                self.1.as_frame()
            ], attributes: None
        }
    }
} */
