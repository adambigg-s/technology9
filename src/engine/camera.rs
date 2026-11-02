use std::f32;
use std::fmt::Display;
use std::fmt::{self};
use std::ops;

use glam::camera::rh::proj::directx;

use crate::engine::transform;
use crate::render;

#[derive(bon::Builder, Debug, Default)]
pub struct Camera
{
     #[builder(default)]
     pub inner: transform::Transform,
     pub ar: f32,
     pub fov: f32,
     pub znear: f32,
     pub zfear: f32,
     #[builder(default)]
     pub pitch: f32,
     #[builder(default)]
     pub yaw: f32,
}

impl Camera
{
     pub fn update_rotation(&mut self, dx: f32, dy: f32, dz: f32)
     {
          self.inner.rotation *= glam::Quat::from_rotation_y(dy)
               * glam::Quat::from_rotation_x(dx)
               * glam::Quat::from_rotation_z(dz);
     }

     pub fn update_position(&mut self, dx: f32, dy: f32, dz: f32)
     {
          self.inner.position += self.inner.forward() * dz;
          self.inner.position += self.inner.right() * dx;
          self.inner.position += self.inner.up() * dy;
     }

     pub fn confine_euler(&mut self)
     {
          self.pitch = self.pitch.clamp(-f32::consts::PI / 2.0 * 0.99, f32::consts::PI / 2.0 * 0.99);
          self.yaw %= f32::consts::TAU;
     }

     pub fn view(&self) -> glam::Mat4
     {
          self.inner.to_matrix4().inverse()
     }

     pub fn proj(&self) -> glam::Mat4
     {
          directx::perspective(self.fov.to_radians(), self.ar, self.znear, self.zfear)
     }

     pub fn inner(&self) -> transform::Transform
     {
          self.inner
     }

     pub fn inner_mut(&mut self) -> &mut transform::Transform
     {
          &mut self.inner
     }

     pub fn set_inner(&mut self, inner: transform::Transform)
     {
          self.inner = inner;
     }

     pub fn ar(&self) -> f32
     {
          self.ar
     }

     pub fn ar_mut(&mut self) -> &mut f32
     {
          &mut self.ar
     }

     pub fn set_ar(&mut self, ar: f32)
     {
          self.ar = ar;
     }

     pub fn fov(&self) -> f32
     {
          self.fov
     }

     pub fn fov_mut(&mut self) -> &mut f32
     {
          &mut self.fov
     }

     pub fn set_fov(&mut self, fov: f32)
     {
          self.fov = fov;
     }

     pub fn znear(&self) -> f32
     {
          self.znear
     }

     pub fn znear_mut(&mut self) -> &mut f32
     {
          &mut self.znear
     }

     pub fn set_znear(&mut self, znear: f32)
     {
          self.znear = znear;
     }

     pub fn zfear(&self) -> f32
     {
          self.zfear
     }

     pub fn zfear_mut(&mut self) -> &mut f32
     {
          &mut self.zfear
     }

     pub fn set_zfear(&mut self, zfear: f32)
     {
          self.zfear = zfear;
     }

     pub fn pitch(&self) -> f32
     {
          self.pitch
     }

     pub fn pitch_mut(&mut self) -> &mut f32
     {
          &mut self.pitch
     }

     pub fn set_pitch(&mut self, pitch: f32)
     {
          self.pitch = pitch;
     }

     pub fn yaw(&self) -> f32
     {
          self.yaw
     }

     pub fn yaw_mut(&mut self) -> &mut f32
     {
          &mut self.yaw
     }

     pub fn set_yaw(&mut self, yaw: f32)
     {
          self.yaw = yaw;
     }
}

impl render::GfxCamera for Camera
{
     fn view_proj(&self) -> glam::Mat4
     {
          self.proj() * self.view()
     }
}

impl Display for Camera
{
     fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result
     {
          writeln!(fmt, "Camera pos: {:.1}", self.inner.position)?;
          writeln!(fmt, "Camera fvec: {:.2}", self.inner.forward())?;
          writeln!(fmt, "Camera rvec: {:.2}", self.inner.right())?;
          writeln!(fmt, "Camera uvec: {:.2}", self.inner.up())?;
          Ok(())
     }
}

impl ops::Deref for Camera
{
     type Target = transform::Transform;

     fn deref(&self) -> &Self::Target
     {
          &self.inner
     }
}

impl ops::DerefMut for Camera
{
     fn deref_mut(&mut self) -> &mut Self::Target
     {
          &mut self.inner
     }
}
