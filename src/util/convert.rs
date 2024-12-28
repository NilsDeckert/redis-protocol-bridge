use std::collections::HashMap;
use redis_protocol::resp3::types::OwnedFrame;

pub trait AsFrame {
    fn as_frame(&self) -> OwnedFrame;
}

impl AsFrame for i64 {
    fn as_frame(&self) -> OwnedFrame {
        OwnedFrame::Number {
            data: *self,
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
