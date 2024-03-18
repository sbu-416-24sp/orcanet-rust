use std::fmt;

#[derive(Debug, Clone)]
pub struct ChunkOutOfBoundsError;
impl fmt::Display for ChunkOutOfBoundsError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Chunk out of bounds")
    }
}
