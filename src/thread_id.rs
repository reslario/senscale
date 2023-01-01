use std::fmt;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct ThreadId(u32);

impl From<u32> for ThreadId {
    fn from(id: u32) -> Self {
        ThreadId(id)
    }
}

impl From<ThreadId> for u32 {
    fn from(id: ThreadId) -> Self {
        id.0
    }
}

impl fmt::Display for ThreadId {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.0.fmt(f)
    }
}
