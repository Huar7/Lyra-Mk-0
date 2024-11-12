// Windowing sama seperti namanya adalah untuk membuat WIndow dan juga menginisialisasi Renderer Unit
use std::sync::Arc;

use pollster::{block_on, FutureExt};

use wgpu::util::{DeviceExt, RenderEncoder};
use wgpu::{Adapter, Device, Instance, PresentMode, Queue, Surface, SurfaceCapabilities, SurfaceConfiguration};// barang utmama untuk membuat state
use wgpu::MemoryHints;

use winit::application::ApplicationHandler;
use winit::dpi::PhysicalSize;
use winit::event::{ElementState, KeyEvent, WindowEvent};
use winit::event_loop::{ActiveEventLoop, EventLoop};
use winit::keyboard::{KeyCode, PhysicalKey};
use winit::window::{Window, WindowId};



/*--------------------------------------------------------------------------------------------------------
Handle Vertex Position
----------------------------------------------------------------------------------------------------------*/


#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]

struct Vertex {
    position: [f32; 3],
    color: [f32; 3],
}


impl Vertex{

    // ini untuk attributes pada vertex buffer layout
    const ATTRIBUTE : [wgpu::VertexAttribute;2]= wgpu::vertex_attr_array![0=> Float32x3, 1=> Float32x3];
    
    fn desc() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout { 
            array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &Self::ATTRIBUTE,
        }
    }
}

const VERTICES: &[Vertex] = &[
    Vertex { position: [0.5, 0.5, 0.0], color: [1.0, 0.0, 0.0] }, // A
    Vertex { position: [-0.5, 0.5, 0.0], color: [0.0, 0.0, 1.0] }, // B
    Vertex { position: [0.5, -0.5, 0.0], color: [1.0, 0.0, 0.0] }, // C
    Vertex { position: [-0.5, -0.5, 0.0], color: [0.0, 1.0, 0.0] }, // D
];

const INDICES: &[u16] = &[
    0,1,2,
    3,2,1,
];




/* ----------------------------------------------------------------------------------------------------
Handle Renderer / State
------------------------------------------------------------------------------------------------------- */

struct State{
    surface: Surface<'static>, // ini surface rendering
    device: Device, // devicenya
    queue: Queue,
    config: wgpu::SurfaceConfiguration, // ini untuk konfigurasi surfacenya 

    size: PhysicalSize<u32>,
    window: Arc<Window>,
    render_pipeline: wgpu::RenderPipeline,
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    num_indices: u32,
}

impl State{
    pub fn new(window:Window) -> Self{
        let window_arc = Arc::new(window);
        let size = window_arc.inner_size();
        let instance = Self::create_gpu_instance();
        let surface = instance.create_surface(window_arc.clone()).unwrap();
        let adapter = Self::create_adapter(instance, &surface); // Adapter ada untuk mengetahui / memberikan informasi dari gpu kita
        let (device, queue) = Self::create_device(&adapter);
        let surface_caps = surface.get_capabilities(&adapter);
        let config = Self::create_surface_config(size, surface_caps);
        surface.configure(&device, &config);

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor{
            label: Some("Shader Utama"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shader.wgsl").into())
        }); //membaca shadernya

        let render_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor { //untuk membuat pipeline
            label: Some("Render Pipeline Layout"), 
            bind_group_layouts: &[], 
            push_constant_ranges: &[] 
        });

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor { 
            label: Some("Render Pipeline"), 
            layout: Some(&render_pipeline_layout), 
            vertex: wgpu::VertexState { 
                module: &shader, 
                entry_point:Some("vs_main") , 
                compilation_options:wgpu::PipelineCompilationOptions::default(), 
                buffers: &[Vertex::desc(),] 
            }, 
            primitive: wgpu::PrimitiveState { 
                topology: wgpu::PrimitiveTopology::TriangleList, 
                strip_index_format: None, 
                front_face: wgpu::FrontFace::Ccw, 
                cull_mode: Some(wgpu::Face::Back), 
                unclipped_depth: false, 
                polygon_mode: wgpu::PolygonMode::Fill, 
                conservative:  false
            }, 
            depth_stencil: None, 
            multisample: wgpu::MultisampleState { 
                count: 1, 
                mask: !0, 
                alpha_to_coverage_enabled: false 
            }, 
            fragment: Some(wgpu::FragmentState { 
                module: &shader, 
                entry_point: Some("fs_main"), 
                compilation_options: wgpu::PipelineCompilationOptions::default(), 
                targets:&[Some(wgpu::ColorTargetState { 
                    format: config.format, 
                    blend: Some(wgpu::BlendState::REPLACE), 
                    write_mask:wgpu::ColorWrites::ALL,
                })],
            }), 
            multiview: None, 
            cache: None, 
        });

        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor{
           label: Some("Wgpu Buffer"), contents:bytemuck::cast_slice(VERTICES), usage: wgpu::BufferUsages::VERTEX, 
        });
        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor{
                label: Some("Index Buffer"), contents:bytemuck::cast_slice(INDICES), usage: wgpu::BufferUsages::INDEX
            }
        );
        let num_indices = INDICES.len() as u32;

        Self{surface,device,queue,config,size,window:window_arc, render_pipeline, vertex_buffer, index_buffer, num_indices}

    }

    fn create_surface_config(size: PhysicalSize<u32>, capabilities: SurfaceCapabilities,)-> wgpu::SurfaceConfiguration{
        
        let surface_format = capabilities
            .formats
            .iter()
            .find(|f| f.is_srgb())
            .copied()
            .unwrap_or(capabilities.formats[0]);

        wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width,
            height: size.height,
            present_mode: PresentMode::AutoNoVsync,
            alpha_mode: capabilities.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        }
    }

  fn create_device(adapter: &Adapter) -> (Device, Queue){
        adapter
            .request_device(
                &wgpu::DeviceDescriptor{
                    memory_hints: MemoryHints::Performance,
                    required_features: wgpu::Features::empty(),
                    required_limits: wgpu::Limits::default(),
                    label: None,
                }, 
                None,
            )
            .block_on()
            .unwrap()
    }

    fn create_adapter(instance: Instance, surface: &Surface) -> Adapter{
        instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                compatible_surface: Some(&surface),
                force_fallback_adapter: false
            })
            .block_on()
            .unwrap()
    }

    fn create_gpu_instance() -> Instance{
        Instance::new(wgpu::InstanceDescriptor{
            backends: wgpu::Backends::VULKAN,
            ..Default::default()
        })
    }

    pub fn resize(&mut self, new_size: PhysicalSize<u32>) { 
        //untuk meresize rendering pipeline berdasarkan ukuran windows
        self.size = new_size;

        self.config.width = new_size.width;
        self.config.height = new_size.height;

        self.surface.configure(&self.device, &self.config);
        
    }

    pub fn render(&mut self) -> Result<(), wgpu::SurfaceError>{
        let output = self.surface.get_current_texture().unwrap();
        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = self

.device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor{
                label: Some("Render Encoder"),
            });
        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor { 
                label: Some("Render Pass"), 
                color_attachments: &[Some(wgpu::RenderPassColorAttachment{
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations { 
                        load: wgpu::LoadOp::Clear(wgpu::Color { 
                            r:0.0 , 
                            g: 0.0, 
                            b: 0.0, 
                            a:  1.0}), // warna background
                        store:  wgpu::StoreOp::Store,}
                })], 
                depth_stencil_attachment: None, 
                timestamp_writes: None, 
                occlusion_query_set: None,  
            });


            render_pass.set_pipeline(&self.render_pipeline);
            
            render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
            render_pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
            render_pass.draw_indexed(0..self.num_indices,0,0..1);
        }
        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();

        Ok(())
    }
    pub fn window(&self) -> &Window{
        &self.window
    }
}




struct StateApp {
    state: Option<State>,
    name: String,
}

impl StateApp{
    pub fn new(namae: String) -> Self{
        Self { state: None , name: namae}
    }
}

impl ApplicationHandler for StateApp{
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {

        let window = event_loop 
            .create_window(Window::default_attributes().with_title(&self.name)) // untuk membuat windows
            .unwrap();
        self.state = Some(State::new(window));
    }
    fn window_event(

        &mut self,

        event_loop: &ActiveEventLoop,

        window_id: WindowId,

        event: WindowEvent,

    ) {
        let window = self.state.as_ref().unwrap().window();

        if window.id() == window_id {
            match event {
                WindowEvent::CloseRequested => {event_loop.exit();},
                WindowEvent::Resized(physical_size) => {self.state.as_mut().unwrap().resize(physical_size);}, // ini untuk mengupdate ukuran ketika winit di perbesar / di perkecil
                WindowEvent::RedrawRequested => {self.state.as_mut().unwrap().render().unwrap();}, // ini untuk mengupdate renderer ketika ada yang terjadi pada winit nya
                WindowEvent::KeyboardInput { //input dari keyboard
                    event: 
                        KeyEvent {
                            state: ElementState::Pressed, 
                            physical_key: PhysicalKey::Code(KeyCode::Escape),
                            ..
                        },
                    ..
                } => event_loop.exit(), // input masukan escape untuk keluar dari aplikasi
                WindowEvent::KeyboardInput { 
                    event:
                        KeyEvent {
                            physical_key: PhysicalKey::Code(KeyCode::Space), 
                            state: ElementState::Pressed, 
                            .. },
                    ..} => {println!("Uji Coba Repeat")},
                
                _ => {},
            }
        }
    }

    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
        let window = self.state.as_ref().unwrap().window();
        window.request_redraw();
    }
}



// Lyra Setup --------------------------------------------------
/*
this is for the lyra engine itself
*/

pub struct SetupGames{
    pub name:String,
    
}

impl SetupGames{
    pub async fn play(self){
        let event_loop = EventLoop::new().unwrap();
        let mut  window_state = StateApp::new(self.name);
    
        let _ = event_loop.run_app(&mut window_state);
    }
    pub fn run(self){
        block_on(self.play())
    }
}
