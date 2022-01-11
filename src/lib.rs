extern crate futures;

use futures::io;
use std::collections::HashMap;
use std::fmt::{self, Debug, Formatter};
use std::io::{Error, ErrorKind};

#[derive(Debug)]
pub struct BlockStream {
    pub blocks: Vec<Block>,
}

#[derive(Clone, PartialEq)]
pub struct Block {
    // Block number, monotonically increasing as the chain grows.
    pub block_number: u64,
    // Hash of the curent block.
    pub hash: [u8; 32],
    // Hash of the parent block.
    pub parent_hash: [u8; 32],
    // Block content.
    pub content: Box<[u8]>,
}

impl Debug for Block {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(
            f,
            "Block[{}] : {:#?} with data : {:#?}",
            self.block_number,
            &hex::encode(self.hash),
            self.content
        )
    }
}

impl BlockStream {
    pub fn init_stream() -> BlockStream {
        BlockStream { blocks: Vec::new() }
    }
}

impl Block {
    pub fn add_block(stream: &mut BlockStream, content: Box<[u8]>) {
        let current_hash: &[u8; 32] = &get_hash(content.clone()).try_into().unwrap();
        let (block_number, parent_hash): (u64, [u8; 32]) = match &stream.blocks.len() {
            0 => (1, [0; 32]),
            _ => {
                let tip = &stream.blocks[stream.blocks.len() - 1];
                (tip.block_number + 1, tip.hash)
            }
        };
        let new_block = Block {
            block_number: block_number,
            hash: *current_hash,
            parent_hash: parent_hash,
            content: content,
        };
        stream.blocks.push(new_block);
    }
}

pub fn get_hash(content: Box<[u8]>) -> Vec<u8> {
    let x = crypto_hash::digest(
        crypto_hash::Algorithm::SHA256,
        Box::<[u8]>::leak(content.clone()),
    );
    let converted_hash: &[u8] = &x;
    converted_hash.to_vec()
}

pub fn start_chain(data: Vec<u8>) -> Option<BlockStream> {
    let mut block_chain: BlockStream = BlockStream { blocks: Vec::new() };

    for item in data.iter() {
        Block::add_block(&mut block_chain, Box::new([*item]));
    }

    Some(block_chain)
}

pub async fn find_common_ancestor(
    blockchain_streams: &mut [BlockStream],
) -> Result<Option<Block>, io::Error> {
    let mut parent_map: HashMap<String, bool> = HashMap::new();
    let mut current_map: HashMap<String, Block> = HashMap::new();
    let common_ancestor_block: Block;
    for chain in blockchain_streams {
        for block in chain.blocks.iter() {
            let parent_key = &hex::encode(block.parent_hash);
            let current_key = &hex::encode(block.hash);
            if block.block_number == 1 {
                current_map.insert(current_key.to_string(), block.clone());
                continue;
            }
            if parent_map.contains_key(parent_key) {
                println!("{:?}", &hex::encode(block.parent_hash));
                common_ancestor_block = current_map
                    .get(parent_key)
                    .expect("could not get ancestor block")
                    .clone();
                println!("{:?}", common_ancestor_block);
                return Ok(Some(common_ancestor_block));
            }
            parent_map.insert(parent_key.to_string(), true);
            current_map.insert(current_key.to_string(), block.clone());
        }
    }
    Err(Error::new(ErrorKind::NotFound, "None"))
}
