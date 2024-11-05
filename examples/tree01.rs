//! This example demonstrates how to use the `NodeIter` iterator to build a tree from a list of hashed chunks.

use rustic_cdc::{HashedChunk, Node, NodeIter};

type IntHash = u32;

use std::cell::RefCell;

thread_local! {
    static HASH_ID: RefCell<IntHash> = const { RefCell::new(0) };
}

fn get_new_hash_id() -> IntHash {
    HASH_ID.with(|hash_id| {
        let mut id = hash_id.borrow_mut();
        let current_id = *id;
        *id += 1;
        current_id
    })
}

#[allow(clippy::ptr_arg)]
fn my_new_node(level: usize, children: &Vec<IntHash>) -> Node<IntHash> {
    Node {
        hash: get_new_hash_id(),
        level,
        children: children.clone(),
    }
}

fn main() {
    let levels = [0usize, 0, 1, 0, 1, 1, 2, 1, 0, 1, 0, 1];
    //let levels = [1usize, 1, 1, 1, 1, 1, 1, 1, 0, 1, 0, 1];
    let hashed_chunk_it = levels.iter().enumerate().map(|(index, level)| HashedChunk {
        hash: IntHash::try_from(index).unwrap(),
        level: *level,
    });

    HASH_ID.set(IntHash::try_from(levels.len()).unwrap());

    for node in NodeIter::new(hashed_chunk_it, my_new_node, 0) {
        println!("{node:?}");
    }
}
