use std::fmt::{Display, Formatter, Write};

use sha1::{Digest, Sha1};

/// Used to store the id of a construct.
#[derive(Clone)]
pub struct Path {
    segments: Vec<String>,
}

impl Path {
    /// Returns the name of the associated object.
    pub fn name(&self) -> &str {
        self.segments
            .iter()
            .last()
            .expect("Path expects at least one element")
    }

    /// Returns the id of this object.
    pub fn id(&self) -> String {
        let name = self.name();
        let mut hasher = Sha1::default();
        for segment in &self.segments {
            hasher.update(segment);
        }
        let hash: String = hasher
            .finalize()
            .as_slice()
            .iter()
            .map(|by| format!("{by:x}"))
            .collect();
        format!("{name}-{hash}")
    }

    /// Extend this path with new `identifier`.
    pub fn push(&mut self, identifier: impl Into<String>) {
        self.segments.push(identifier.into())
    }
}

impl From<String> for Path {
    fn from(segment: String) -> Self {
        Path {
            segments: vec![segment],
        }
    }
}

impl From<Vec<String>> for Path {
    fn from(segments: Vec<String>) -> Self {
        Path { segments }
    }
}

impl Display for Path {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        if let Some(first) = self.segments.first() {
            f.write_str(first)?;
        }
        for segment in self.segments.iter().skip(1) {
            f.write_char('/')?;
            f.write_str(segment)?;
        }
        Ok(())
    }
}
