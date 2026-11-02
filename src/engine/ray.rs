use std::range;

pub trait Cast
{
     type Hit;

     fn cast(&self, ray: Ray) -> Option<Self::Hit>;
}

#[derive(bon::Builder, Debug, Default, Clone, Copy)]
pub struct Ray
{
     pub origin: glam::Vec3,
     pub direction: glam::Vec3,
     pub tspan: range::Range<f32>,
}

impl Ray
{
     pub fn at(&self, time: f32) -> glam::Vec3
     {
          self.origin + self.direction * time
     }

     pub fn origin(&self) -> glam::prelude::Vec3
     {
          self.origin
     }

     pub fn origin_mut(&mut self) -> &mut glam::Vec3
     {
          &mut self.origin
     }

     pub fn set_origin(&mut self, origin: glam::Vec3)
     {
          self.origin = origin;
     }

     pub fn direction(&self) -> glam::prelude::Vec3
     {
          self.direction
     }

     pub fn direction_mut(&mut self) -> &mut glam::Vec3
     {
          &mut self.direction
     }

     pub fn set_direction(&mut self, direction: glam::Vec3)
     {
          self.direction = direction;
     }

     pub fn tspan(&self) -> range::Range<f32>
     {
          self.tspan
     }

     pub fn tspan_mut(&mut self) -> &mut range::Range<f32>
     {
          &mut self.tspan
     }

     pub fn set_tspan(&mut self, tspan: range::Range<f32>)
     {
          self.tspan = tspan;
     }
}
