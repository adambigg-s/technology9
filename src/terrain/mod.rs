use std::any;
use std::fmt::Debug;
use std::hash;
use std::hash::Hash;
use std::hash::Hasher;

use crate::world::chunk;
use crate::world::delta;

#[derive(bon::Builder, Debug)]
pub struct TerrainGenerator {}

impl TerrainGenerator
{
     pub fn new(seed: u32) -> Self
     {
          Self {}
     }

     pub fn form_chunk(&self, chunk: &mut chunk::Chunk) -> delta::BlockDeltas
     {
          todo!()
     }
}
