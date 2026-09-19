#![allow(unused)]

use crate::engine::neighbors;
use crate::terrain;
use crate::world;
use crate::world::block;
use crate::world::delta;

#[derive(Debug)]
pub struct Pitfalls;

impl terrain::BiomeGeneration for Pitfalls
{
     fn generate(
          &self,
          chunk: &mut world::chunk::Chunk,
          noise: &noise::Perlin,
          config: &terrain::TerrainConfig,
          deltas: &mut delta::BlockDeltas,
     )
     {
          chunk.blocks_mut().fill(block::Block::Plain);
          let size = chunk.size();
          for z in 0 .. size.z
          {
               for x in 0 .. size.x
               {
                    if x % 5 == 0 && z % 5 == 0
                    {
                         let coord = glam::ivec3(x, 0, z);

                         *chunk.get_mut(coord) = block::Block::Light
                    }

                    if x % 5 == 0 || z % 5 == 0 || x == size.x - 1 || z == size.z - 1
                    {
                         continue;
                    }

                    for y in 0 .. size.y + size.y
                    {
                         let coord = glam::ivec3(x, y, z);

                         if chunk.check_index(coord)
                         {
                              *chunk.get_mut(coord) = block::Block::Air;
                         }
                         else
                         {
                              let world_coord = chunk.world_position() + coord;
                              let chunk_world_coord = chunk.chunk_world_coords(world_coord);
                              let chunk_coord = chunk.to_chunk_coords(world_coord);
                              deltas.insert(
                                   chunk_world_coord,
                                   delta::ChunkDelta {
                                        coord: chunk_coord,
                                        delta: block::Block::Air,
                                   },
                              );
                         }
                    }
               }
          }
     }

     fn as_any(&self) -> &dyn std::any::Any
     {
          self
     }
}
