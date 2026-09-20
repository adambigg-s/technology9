use std::sync;

use rustc_hash as rh;

use crate::visual::light;
use crate::world::block;
use crate::world::chunk;

pub trait DeltaValue
where
     Self: Sized + Clone + Copy,
{
     fn resolve(&self, delta: ChunkDelta<Self>, chunk: &mut chunk::Chunk);
}

#[derive(bon::Builder, Debug, Default, Clone, Copy)]
pub struct ChunkDelta<Delta>
{
     pub coord: glam::IVec3,
     pub delta: Delta,
}

#[derive(bon::Builder, Debug, Default)]
pub struct ChunkDeltaMap<Delta>
{
     pub deltas: rh::FxHashMap<glam::IVec3, sync::Arc<Vec<ChunkDelta<Delta>>>>,
}

impl<Delta> ChunkDeltaMap<Delta>
where
     Delta: DeltaValue,
{
     pub fn new() -> Self
     where
          Delta: Default,
     {
          Self::default()
     }

     pub fn merge(&mut self, other: &Self)
     {
          other.deltas.iter().for_each(|(&coord, delta)| {
               let entry = self.deltas.entry(coord).or_default();
               let existing = sync::Arc::make_mut(entry);
               existing.extend(delta.iter().copied());
          });
     }

     pub fn insert(&mut self, coord: glam::IVec3, delta: ChunkDelta<Delta>)
     {
          let entry = self.deltas.entry(coord).or_default();
          let existing = sync::Arc::make_mut(entry);
          existing.push(delta);
     }

     pub fn get_deltas(&self, coord: glam::IVec3) -> sync::Arc<Vec<ChunkDelta<Delta>>>
     where
          Delta: Clone + Copy,
     {
          self.deltas.get(&coord).cloned().unwrap_or_default()
     }

     pub fn take_deltas(&mut self, coord: glam::IVec3) -> sync::Arc<Vec<ChunkDelta<Delta>>>
     {
          self.deltas.remove(&coord).unwrap_or_default()
     }
}

pub type BlockDeltas = ChunkDeltaMap<block::Block>;

impl DeltaValue for block::Block
{
     fn resolve(&self, delta: ChunkDelta<Self>, chunk: &mut chunk::Chunk)
     {
          _ = (delta, chunk);
     }
}

pub type LightDeltas = ChunkDeltaMap<light::LightDelta>;

impl DeltaValue for light::LightDelta
{
     fn resolve(&self, delta: ChunkDelta<Self>, chunk: &mut chunk::Chunk)
     {
          _ = (delta, chunk);
     }
}
