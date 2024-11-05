use crate::Separator;

/// A chunk is a part of a stream of data that is separated by a separator.
#[derive(Debug, Clone, Copy)]
pub struct Chunk {
    /// The index of the chunk in the stream.
    pub index: u64,

    /// The size of the chunk.
    pub size: u64,

    /// The hash of the separator that separates the chunk from the next chunk.
    pub separator_hash: u64,
}

/// An iterator that chunks data.
#[derive(Debug)]
pub struct ChunkIter<Iter> {
    /// The separators that separate the chunks.
    separators: Iter,

    /// The length of the stream.
    stream_length: u64,

    /// The index of the last separator.
    last_separator_index: u64,
}

impl<Iter: Iterator<Item = Separator>> ChunkIter<Iter> {
    /// Creates a new `ChunkIter`.
    ///
    /// # Arguments
    ///
    /// * `iter` - The separators that separate the chunks.
    /// * `stream_length` - The length of the stream.
    pub fn new(iter: Iter, stream_length: u64) -> Self {
        Self {
            separators: iter,
            stream_length,
            last_separator_index: 0,
        }
    }
}

impl<Iter: Iterator<Item = Separator>> Iterator for ChunkIter<Iter> {
    type Item = Chunk;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(separator) = self.separators.next() {
            let chunk_size = separator.index - self.last_separator_index;
            self.last_separator_index = separator.index;
            Some(Chunk {
                index: self.last_separator_index,
                size: chunk_size,
                separator_hash: separator.hash,
            })
        } else {
            let chunk_size = self.stream_length - self.last_separator_index;
            self.last_separator_index = self.stream_length;
            if chunk_size > 0 {
                Some(Chunk {
                    index: self.last_separator_index,
                    size: chunk_size,
                    separator_hash: 0, // any value is ok, last chunk of the stream.
                })
            } else {
                None
            }
        }
    }
}
