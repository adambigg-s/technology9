use std::cmp;
use std::ops;

use crate::engine::kinematics;

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Axis
{
     PosX,
     PosY,
     Posz,
     NegX,
     NegY,
     Negz,
     #[default]
     Undefined,
}

#[derive(bon::Builder, Debug, Clone, Copy)]
pub struct AaBb<T, const N: usize>
{
     pub lo: [T; N],
     pub hi: [T; N],
}

impl<T, const N: usize> AaBb<T, N>
{
     pub fn new(lo: [T; N], hi: [T; N]) -> Self
     {
          Self {
               lo,
               hi,
          }
     }

     pub fn overlaps(&self, other: &Self) -> bool
     where
          T: cmp::PartialOrd,
     {
          (0 .. N).all(|dim| self.lo[dim] <= other.hi[dim] && self.hi[dim] >= other.lo[dim])
     }

     pub fn point_sides(point: [T; N], sides: [T; N]) -> Self
     where
          T: ops::Add<T, Output = T> + ops::Sub<T, Output = T> + Copy,
     {
          let mut lo = point;
          let mut hi = point;
          (0 .. N).for_each(|dim| {
               lo[dim] = lo[dim] - sides[dim];
               hi[dim] = hi[dim] + sides[dim];
          });
          Self {
               lo,
               hi,
          }
     }
}

impl<T, const N: usize> kinematics::Collision for AaBb<T, N>
where
     T: cmp::PartialOrd,
{
     type Collider = Self;

     fn collides(&self, other: Self::Collider) -> bool
     {
          self.overlaps(&other)
     }
}

impl<T, const N: usize> AaBb<T, N>
where
     T: cmp::PartialOrd + Copy,
{
     pub fn expand(&mut self, point: [T; N])
     {
          (0 .. N).for_each(|dim| {
               self.lo[dim] = match self.lo[dim].partial_cmp(&point[dim])
               {
                    | Some(cmp) =>
                    {
                         match cmp
                         {
                              | cmp::Ordering::Less => self.lo[dim],
                              | cmp::Ordering::Equal => self.lo[dim],
                              | cmp::Ordering::Greater => point[dim],
                         }
                    }
                    | None => self.lo[dim],
               };

               self.hi[dim] = match self.hi[dim].partial_cmp(&point[dim])
               {
                    | Some(cmp) =>
                    {
                         match cmp
                         {
                              | cmp::Ordering::Less => point[dim],
                              | cmp::Ordering::Equal => self.hi[dim],
                              | cmp::Ordering::Greater => self.hi[dim],
                         }
                    }
                    | None => self.hi[dim],
               };
          });
     }
}

#[cfg(test)]
mod tests
{
     use super::*;

     #[test]
     fn aabb_expand()
     {
          let mut aabb = AaBb::new(glam::IVec3::ZERO.to_array(), glam::IVec3::ZERO.to_array());
          aabb.expand(glam::IVec3::splat(-1).to_array());
          aabb.expand(glam::IVec3::splat(1).to_array());
          assert!(aabb.lo == [-1, -1, -1]);
          assert!(aabb.hi == [1, 1, 1]);
     }
}
