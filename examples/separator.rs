//! This example shows how to use the `SeparatorIter` to find the separators in a file.

use std::{
    env::args,
    fs::File,
    io::{self, prelude::Read, BufReader},
};

use rustic_cdc::SeparatorIter;

fn chunk_file<S: Into<String>>(path: S) -> io::Result<()> {
    let f = File::open(path.into())?;
    let reader: BufReader<File> = BufReader::new(f);
    let byte_iter = reader.bytes().map(|b| b.unwrap());

    let mut nb_separator: usize = 0;
    for separator in SeparatorIter::new(byte_iter) {
        println!("Index: {}, hash: {:016x}", separator.index, separator.hash);
        nb_separator += 1;
    }
    println!("We found {nb_separator} separators.");

    Ok(())
}

fn main() {
    let path = args()
        .nth(1)
        .unwrap_or_else(|| "myLargeFile.bin".to_string());
    
    chunk_file(path).unwrap();
}
