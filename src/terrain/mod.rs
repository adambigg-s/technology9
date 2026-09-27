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

     pub fn form_chunk(&self, chunk: &mut chunk::Chunk) -> delta::BlockDeltas
     {
          let out_deltas = delta::BlockDeltas::new();
          if chunk.world_position().y >= 0
          {
               return out_deltas;
          }

          for i in 0 .. chunk.width()
          {
               for j in 0 .. chunk.width()
               {
                    *chunk.get_mut(glam::ivec3(i as i32, 0, j as i32)) = crate::world::block::Block::Plain;
               }
          }

          out_deltas
     }
}
