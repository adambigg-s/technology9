pub mod application;
pub mod engine;
pub mod entry;
pub mod render;
pub mod terrain;
pub mod visual;
pub mod world;

fn main() -> anyhow::Result<()>
{
     application::run::<entry::State>()?;
     Ok(())
}
