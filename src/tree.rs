/// Example of type to use with the generic structures below.
//pub type Hash256 = [u8; 256/8];

#[derive(Debug)]
pub struct HashedChunk<H> {
    /// The hash of the chunk.
    pub hash: H,

    /// The level of the chunk.
    pub level: usize,
}

/// A node in a tree.
#[derive(Debug)]
pub struct Node<H> {
    /// The hash of the node.
    ///
    /// # Note
    ///
    /// The hash acting as the ID of this node.
    pub hash: H,

    /// The level of the node.
    pub level: usize,

    /// The children of the node.
    pub children: Vec<H>,
}

/// An iterator that generates nodes from hashed chunks.
#[derive(Debug)]
pub struct NodeIter<I, F, H> {
    /// The chunks to generate nodes from.
    chunks: I,

    /// The function to create a new node.
    new_node: F,

    /// The maximum number of children a node can have.
    max_children: usize,

    /// The hashes at each level.
    level_hashes: Vec<Vec<H>>, // level_hashes[level] -> Vec<H>

    /// The output buffer.
    out_buffer: Vec<Node<H>>, // Fifo
}

impl<I, F, H> NodeIter<I, F, H>
where
    I: Iterator<Item = HashedChunk<H>>,
    F: Fn(usize, &Vec<H>) -> Node<H>,
    H: Copy,
{
    /// Creates a new `NodeIter`.
    pub fn new(iter: I, new_node: F, max_node_children: usize) -> Self {
        Self {
            chunks: iter,
            new_node,
            max_children: max_node_children,
            level_hashes: Vec::with_capacity(16),
            out_buffer: Vec::with_capacity(16),
        }
    }

    /// Adds a hash at a specific level.
    ///
    /// # Arguments
    ///
    /// * `level` - The level to add the hash at.
    /// * `hash` - The hash to add.
    fn add_at_level(&mut self, level: usize, hash: H) {
        // Ensures that the vector is large enough.
        if level >= self.level_hashes.len() {
            self.level_hashes.resize(level + 1, vec![]);
        }

        self.level_hashes[level].push(hash);

        // If max_children was set to non-zero, limit the number of children.
        if self.level_hashes[level].len() == self.max_children {
            self.output_level(level);
        }
    }

    /// Outputs a level.
    ///
    /// # Arguments
    ///
    /// * `level` - The level to output.
    fn output_level(&mut self, level: usize) {
        match self.level_hashes[level].len() {
            0 => {} // Don't output empty nodes.
            1 => {
                // Don't output a node with only 1 hash, move it to the upper level.
                let level_up_hash = self.level_hashes[level][0];
                self.level_hashes[level].clear();
                self.add_at_level(level + 1, level_up_hash);
            }
            _ => {
                let node = (self.new_node)(level, &self.level_hashes[level]);
                let level_up_hash = node.hash;
                self.out_buffer.push(node);
                self.level_hashes[level].clear();
                self.add_at_level(level + 1, level_up_hash);
            }
        }
    }

    /// Outputs levels below a specific level.
    ///
    /// # Arguments
    ///
    /// * `below_level` - The level to output below.
    fn output_levels(&mut self, below_level: usize) {
        for level in 0..below_level {
            self.output_level(level);
        }
    }
}

impl<I, F, H> Iterator for NodeIter<I, F, H>
where
    I: Iterator<Item = HashedChunk<H>>,
    F: Fn(usize, &Vec<H>) -> Node<H>,
    H: Copy,
{
    type Item = Node<H>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if !self.out_buffer.is_empty() {
                return self.out_buffer.pop();
            }

            if let Some(chunk) = self.chunks.next() {
                self.add_at_level(0, chunk.hash);
                self.output_levels(chunk.level);
                self.out_buffer.reverse();
            } else {
                let len = self.level_hashes.len();
                if len > 0 {
                    // Flush the remaining hashes.
                    self.output_levels(len);
                    self.level_hashes.clear();
                    self.out_buffer.reverse();
                } else {
                    return None;
                }
            }
        }
    }
}
