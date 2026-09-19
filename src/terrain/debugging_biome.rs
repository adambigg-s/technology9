#![allow(unused)]

use crate::engine::neighbors;
use crate::terrain;
use crate::world;
use crate::world::block;
use crate::world::delta;

#[derive(Debug)]
pub struct DebuggingBiome;

impl terrain::BiomeGeneration for DebuggingBiome
{
     fn generate(
          &self,
          chunk: &mut world::chunk::Chunk,
          noise: &noise::Perlin,
          config: &terrain::TerrainConfig,
          deltas: &mut delta::BlockDeltas,
     )
     {
          let size = chunk.size();
          for z in 0 .. size.z
          {
               for y in 0 .. size.y
               {
                    for x in 0 .. size.x
                    {
                         let coord = glam::ivec3(x, y, z);

                         *chunk.get_mut(coord) = block::Block::empty();
                    }
               }
          }

          *chunk.get_mut(glam::IVec3::ZERO) = block::Block::Light;
     }

     fn as_any(&self) -> &dyn std::any::Any
     {
          self
     }
}
