//! This example shows how to build a tree of hash nodes from a file.

#[macro_use]
extern crate arrayref;

use std::{
    env::args,
    fmt::{self, Debug, Formatter},
    fs::File,
    io::{
        self,
        prelude::{Read, Seek},
        BufReader, SeekFrom,
    },
};

use ring::digest;
use rustic_cdc::{Chunk, ChunkIter, HashToLevel, HashedChunk, Node, NodeIter, SeparatorIter};

type Hash256 = [u8; 256 / 8];

/// A reader that calculates the digest of the data read.
pub struct DigestReader<R> {
    /// The inner reader.
    inner: R,

    /// The digest context.
    pub digest: digest::Context,
}

impl<R> Debug for DigestReader<R> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "DigestReader")
    }
}

impl<R: Read> DigestReader<R> {
    /// Creates a new `DigestReader`.
    ///
    /// # Arguments
    ///
    /// * `inner` - The inner reader.
    /// * `digest` - The digest context.
    pub fn new(inner: R, digest: digest::Context) -> Self {
        Self { inner, digest }
    }
}

impl<R: Read> Read for DigestReader<R> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let size = self.inner.read(buf)?;
        self.digest.update(&buf[0..size]);
        Ok(size)
    }
}

fn file_chunk_reader<S: Into<String>>(
    path: S,
    chunk: &Chunk,
) -> io::Result<io::Take<BufReader<File>>> {
    let mut file = File::open(path.into())?;

    _ = file.seek(SeekFrom::Start(chunk.index))?;

    Ok(BufReader::new(file).take(chunk.size))
}

fn new_hash_node(level: usize, children: &Vec<Hash256>) -> Node<Hash256> {
    let mut ctx = digest::Context::new(&digest::SHA256);
    ctx.update(&[1u8]); // To mark that it is a node, not a chunk.
    for child in children {
        ctx.update(child);
    }
    let digest = ctx.finish();
    let hash: Hash256 = *array_ref![digest.as_ref(), 0, 256 / 8];

    Node {
        hash,
        level,
        children: children.clone(),
    }
}

fn chunk_file<S: Into<String>>(path: S) -> io::Result<()> {
    let path = path.into();
    // Opens the file and gets the byte_iter.
    let f = File::open(path.clone())?;
    let stream_length = f.metadata().unwrap().len();
    let reader: BufReader<File> = BufReader::new(f);
    let byte_iter = reader.bytes().map(|b| b.unwrap());

    // Iterates on the separators.
    let separator_iter = SeparatorIter::new(byte_iter);

    // Iterates on the chunks.
    let chunk_iter = ChunkIter::new(separator_iter, stream_length);

    // Converts into hashed chunks.
    let hashed_chunk_iter = chunk_iter.map(|chunk| {
        // Calculates the sha256 of the chunks.
        let chunk_reader = file_chunk_reader(path.clone(), &chunk).unwrap();
        let mut digest_reader =
            DigestReader::new(chunk_reader, digest::Context::new(&digest::SHA256));
        digest_reader.digest.update(&[0u8]); // To mark that it is a chunk, not a node.
        _ = io::copy(&mut digest_reader, &mut io::sink()).unwrap();
        let digest = digest_reader.digest.finish();
        let hash: Hash256 = *array_ref![digest.as_ref(), 0, 256 / 8];

        // Calculates the level of the separators.
        let level = HashToLevel::custom_new(13, 3).to_level(chunk.separator_hash);

        HashedChunk { hash, level }
    });

    // Builds a tree of hash nodes.
    let mut nb_nodes = 0u64;
    let mut total_children = 0u64;
    let mut level_counts = vec![];
    for node in NodeIter::new(hashed_chunk_iter, new_hash_node, 0) {
        println!(
            "Node: {{level: {}, children.len(): {}}}",
            node.level,
            node.children.len()
        );
        nb_nodes += 1;
        total_children += node.children.len() as u64;

        if node.level >= level_counts.len() {
            level_counts.resize(node.level + 1, 0);
        }
        level_counts[node.level] += 1;
    }
    println!("Total number of nodes: {nb_nodes}.");
    println!("Average number of children: {}.", total_children / nb_nodes);
    println!("Level counts: {level_counts:?}.");

    Ok(())
}

fn main() {
    let path = args()
        .nth(1)
        .unwrap_or_else(|| "myLargeFile.bin".to_string());

    chunk_file(path).unwrap();
}
