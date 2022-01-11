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
                common_ancestor_block = current_map
                    .get(parent_key)
                    .expect("could not get ancestor block")
                    .clone();
                return Ok(Some(common_ancestor_block));
            }
            parent_map.insert(parent_key.to_string(), true);
            current_map.insert(current_key.to_string(), block.clone());
        }
    }
    Err(Error::new(ErrorKind::NotFound, "None"))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[actix_rt::test]
    async fn ancestor_absent() {
        let data_chain_1: Vec<u8> = vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
        let data_chain_2: Vec<u8> = vec![10, 20, 30, 40, 50, 60, 70, 80, 90, 100];

        let chain_1 = start_chain(data_chain_1);
        let chain_2 = start_chain(data_chain_2);

        let common_block: Result<Option<Block>, io::Error> =
            find_common_ancestor(&mut [chain_1.unwrap(), chain_2.unwrap()]).await;

        assert!(common_block.is_err());
    }

    #[actix_rt::test]
    async fn ancestor_present() {
        let data_chain_1: Vec<u8> = vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
        let data_chain_2: Vec<u8> = vec![10, 20, 30, 40, 50, 6, 7, 8, 9, 10];

        let chain_1 = start_chain(data_chain_1);
        let chain_2 = start_chain(data_chain_2);

        let common_block: Result<Option<Block>, io::Error> =
            find_common_ancestor(&mut [chain_1.unwrap(), chain_2.unwrap()]).await;

        let expected = Block {
            block_number: 6,
            hash: [
                103, 88, 110, 152, 250, 210, 125, 160, 185, 150, 139, 192, 57, 161, 239, 52, 201,
                57, 185, 184, 229, 35, 168, 190, 248, 157, 71, 134, 8, 197, 236, 246,
            ],
            parent_hash: [
                212, 115, 94, 58, 38, 94, 22, 238, 224, 63, 89, 113, 139, 155, 93, 3, 1, 156, 7,
                216, 182, 197, 31, 144, 218, 58, 102, 110, 236, 19, 171, 53,
            ],
            content: Box::new([6]),
        };

        assert_eq!(common_block.unwrap().unwrap(), expected);
    }
}
