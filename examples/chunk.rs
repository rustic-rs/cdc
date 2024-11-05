//! This example shows how to chunk a file using the `ChunkIter` iterator.

use std::fs::File;
use std::io;
use std::io::prelude::*;
use std::io::BufReader;
use std::{
    cmp::{max, min},
    env::args,
};

use rustic_cdc::{ChunkIter, SeparatorIter};

#[inline]
fn my_default_predicate(x: u64) -> bool {
    (x & 0b1_1111_1111_1111) == 0b1_1111_1111_1111
}

#[allow(
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::cast_precision_loss
)]
fn chunk_file<S: Into<String>>(path: S) -> io::Result<()> {
    let f = File::open(path.into())?;
    let stream_length = f.metadata().unwrap().len();
    let reader: BufReader<File> = BufReader::new(f);
    let byte_iter = reader.bytes().map(|b| b.unwrap());

    //let separator_iter = SeparatorIter::new(byte_iter);
    //let separator_iter = SeparatorIter::custom_new(byte_iter, 6, |x| (x & 0b1111111111111) == 0b1111111111111);
    let separator_iter = SeparatorIter::custom_new(byte_iter, 6, my_default_predicate);
    let chunk_iter = ChunkIter::new(separator_iter, stream_length);
    let mut nb_chunk = 0;
    let mut total_size = 0;
    let mut smallest_size = u64::MAX;
    let mut largest_size = 0;
    let expected_size = 1 << 13;
    let mut size_variance = 0;
    for chunk in chunk_iter {
        println!(
            "Index: {}, size: {:6}, separator_hash: {:016x}",
            chunk.index, chunk.size, chunk.separator_hash
        );
        nb_chunk += 1;
        total_size += chunk.size;
        smallest_size = min(smallest_size, chunk.size);
        largest_size = max(largest_size, chunk.size);
        size_variance += (chunk.size as i64 - i64::from(expected_size)).pow(2);
    }

    println!(
        "{} chunks with an average size of {} bytes.",
        nb_chunk,
        total_size / nb_chunk
    );
    println!("Expected chunk size: {expected_size} bytes");
    println!("Smallest chunk: {smallest_size} bytes.");
    println!("Largest chunk: {largest_size} bytes.");
    println!(
        "Standard size deviation: {} bytes.",
        (size_variance as f64 / nb_chunk as f64).sqrt() as u64
    );

    Ok(())
}

fn main() {
    let path = args()
        .nth(1)
        .unwrap_or_else(|| "myLargeFile.bin".to_string());

    chunk_file(path).unwrap();
}
