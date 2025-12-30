use std::io::Write;

pub mod prelude {
    pub use super::{StreamResult, StreamWritable};
}

pub type StreamResult = Result<(), std::io::Error>;

// TODO: review naming and result type of this. not
// fully sure if this is the best way forward
pub trait StreamWritable<W: Write>: Sized {
    fn write_to_stream(self, stream: &mut W) -> StreamResult;
}
