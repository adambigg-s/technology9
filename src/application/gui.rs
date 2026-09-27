#[derive(bon::Builder)]
pub struct GuiContext
{
     pub context: egui::Context,
     pub state: egui_winit::State,
     pub renderer: egui_wgpu::Renderer,
}

impl GuiContext
{
     pub async fn new() -> anyhow::Result<Self>
     {
          todo!()
     }
}
