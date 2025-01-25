use std::collections::HashMap;
use redis_protocol::resp3::types::OwnedFrame;

#[cfg(feature = "serde")]
use serde::{Serialize, Serializer, Deserialize, Deserializer};
#[cfg(feature = "serde")]
use redis_protocol::resp3::{decode, encode, types::Resp3Frame};
#[cfg(feature = "serde")]
use serde::de;

#[cfg(feature = "serde")]
pub struct SerializableFrame(OwnedFrame);

#[cfg(feature = "serde")]
impl Serialize for SerializableFrame {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer
    {
        let mut encoded: Vec<u8> = vec![0u8; self.0.encode_len(false)];
        encode::complete::encode(
            &mut encoded,
            &self.0,
            false).expect("Failed to encode");
        serializer.serialize_bytes(&*encoded)

    }
}

#[cfg(feature = "serde")]
impl<'de> Deserialize<'de> for SerializableFrame {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        // First, deserialize the incoming bytes
        let bytes: Vec<u8> = Deserialize::deserialize(deserializer)?;

        // Use the decode function from redis_protocol to parse the frame
        if let Ok(decoded) = decode::complete::decode(&bytes) {
            match decoded {
                Some((frame, _)) => Ok(SerializableFrame(frame)),
                None => Err(de::Error::custom("Decoded yielded nothing")),
            }
        } else {
           Err(de::Error::custom("Failed to decode"))
        }
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

impl AsFrame for usize {
    fn as_frame(&self) -> OwnedFrame {
        OwnedFrame::BigNumber {
            data: self.to_ne_bytes().to_vec(),
            attributes: None
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

impl AsFrame for str {
    fn as_frame(&self) -> OwnedFrame {
        OwnedFrame::BlobString {
            data: Vec::from(self),
            attributes: None,
        }
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
            framemap.insert(
                key.as_frame(),
                value.as_frame()
            );
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[cfg(feature = "serde")]
    fn test_serialize_deserialize_array() {
        // Create an array frame
        let frame = OwnedFrame::SimpleString {
            data: "Hello World".into(),
            attributes: None,
        };

        let serializable = SerializableFrame(frame.clone());

        let serialized = bincode::serialize(&serializable)
            .expect("Failed to serialize SerializableFrame");

        let deserialized: SerializableFrame = bincode::deserialize(&serialized)
            .expect("Failed to deserialize SerializableFrame");

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
