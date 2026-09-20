use std::fmt::Debug;

use crate::world::chunk;
use crate::world::delta;

#[derive(bon::Builder, Debug)]
pub struct TerrainGenerator {}

impl TerrainGenerator
{
     pub fn new(_seed: u32) -> Self
     {
          Self {}
     }

     pub fn form_chunk(&self, _chunk: &mut chunk::Chunk) -> delta::BlockDeltas
     {
          todo!()
     }
}
