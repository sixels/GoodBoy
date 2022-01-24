#![allow(dead_code)]

use std::{
    sync::mpsc::{self, Receiver, Sender, SyncSender, TryRecvError, TrySendError},
    thread,
    time::Duration,
};

use goodboy_core::{
    io::JoypadButton,
    vm::{Screen, SCREEN_HEIGHT, SCREEN_WIDTH, VM},
};
use wgpu::util::StagingBelt;
use wgpu_glyph::{ab_glyph, GlyphBrushBuilder, Section, Text};
use winit::{dpi::PhysicalSize, event::VirtualKeyCode, window::Window};
use winit_input_helper::WinitInputHelper;

use super::{ColorSchemeIter, IoEvent};

pub struct WgpuState {
    instance: wgpu::Instance,
    surface: wgpu::Surface,
    texture_format: wgpu::TextureFormat,
    device: wgpu::Device,
    queue: wgpu::Queue,

    frame: wgpu::Texture,
    frame_view: wgpu::TextureView,
    frame_bind_group: wgpu::BindGroup,

    staging_belt: wgpu::util::StagingBelt,
    glyph_brush: wgpu_glyph::GlyphBrush<()>,

    render_pipeline: wgpu::RenderPipeline,

    size: PhysicalSize<u32>,
}

impl WgpuState {
    pub fn render_frame(&mut self, frame_data: &[u8], fps: u16) {
        // render the screen
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor::default());

        // Get the next frame
        let frame = self
            .surface
            .get_current_texture()
            .expect("Could not get the next frame");
        let view = frame
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        // staging_belt.write_buffer(encoder, target, offset, size, device)

        // render the game frame
        {
            self.queue.write_texture(
                // Tells wgpu where to copy the pixel data
                wgpu::ImageCopyTexture {
                    texture: &self.frame,
                    mip_level: 0,
                    origin: wgpu::Origin3d::ZERO,
                    aspect: wgpu::TextureAspect::All,
                },
                // The actual pixel data
                frame_data,
                // The layout of the texture
                wgpu::ImageDataLayout {
                    offset: 0,
                    bytes_per_row: std::num::NonZeroU32::new(4 * SCREEN_WIDTH as u32),
                    rows_per_image: std::num::NonZeroU32::new(SCREEN_HEIGHT as u32),
                },
                wgpu::Extent3d {
                    width: SCREEN_WIDTH as _,
                    height: SCREEN_HEIGHT as _,
                    depth_or_array_layers: 1,
                },
            );

            let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render pass"),
                color_attachments: &[wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                        store: true,
                    },
                }],
                depth_stencil_attachment: None,
            });

            rpass.set_pipeline(&self.render_pipeline);
            rpass.set_bind_group(0, &self.frame_bind_group, &[]);
            rpass.draw(0..3, 0..2);
        }

        // render the FPS counter
        {
            self.glyph_brush.queue(Section {
                screen_position: (5.0, 5.0),
                bounds: (self.size.width as f32, self.size.height as f32),
                text: vec![Text::new(&format!("FPS: {}", fps))
                    .with_color([1.0, 1.0, 1.0, 1.0])
                    .with_scale(20.0)],
                ..Section::default()
            });

            self.glyph_brush
                .draw_queued(
                    &self.device,
                    &mut self.staging_belt,
                    &mut encoder,
                    &view,
                    self.size.width,
                    self.size.height,
                )
                .expect("Draw queued");

            self.staging_belt.finish();
        }

        self.queue.submit(Some(encoder.finish()));
        frame.present();
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        self.size.width = width;
        self.size.height = height;

        self.surface.configure(
            &self.device,
            &wgpu::SurfaceConfiguration {
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
                format: self.texture_format,
                width,
                height,
                present_mode: wgpu::PresentMode::Mailbox,
            },
        );
    }

    pub async fn new(window: &Window) -> WgpuState {
        let instance = wgpu::Instance::new(wgpu::Backends::all());
        let surface = unsafe { instance.create_surface(window) };

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .expect("Could not request the adapter");

        let texture_format = surface.get_preferred_format(&adapter).unwrap();

        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    limits: wgpu::Limits::downlevel_webgl2_defaults()
                        .using_resolution(adapter.limits()),
                    ..Default::default()
                },
                None,
            )
            .await
            .expect("Could not request a device");

        let size = window.inner_size();

        surface.configure(
            &device,
            &wgpu::SurfaceConfiguration {
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
                format: texture_format,
                width: size.width,
                height: size.height,
                present_mode: wgpu::PresentMode::Mailbox,
            },
        );

        let frame = device.create_texture(&wgpu::TextureDescriptor {
            label: None,
            size: wgpu::Extent3d {
                width: SCREEN_WIDTH as u32,
                height: SCREEN_HEIGHT as u32,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
        });
        let frame_view = frame.create_view(&Default::default());
        let frame_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: None,
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            multisampled: false,
                            view_dimension: wgpu::TextureViewDimension::D2,
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Sampler {
                            comparison: false,
                            filtering: true,
                        },
                        count: None,
                    },
                ],
            });
        let frame_bind_group = {
            let frame_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
                address_mode_u: wgpu::AddressMode::ClampToEdge,
                address_mode_v: wgpu::AddressMode::ClampToEdge,
                address_mode_w: wgpu::AddressMode::ClampToEdge,
                mag_filter: wgpu::FilterMode::Linear,
                min_filter: wgpu::FilterMode::Nearest,
                mipmap_filter: wgpu::FilterMode::Nearest,
                ..Default::default()
            });

            device.create_bind_group(&wgpu::BindGroupDescriptor {
                layout: &frame_bind_group_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::TextureView(&frame_view),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::Sampler(&frame_sampler),
                    },
                ],
                label: Some("img_bind_group"),
            })
        };

        let render_pipeline = {
            let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: None,
                bind_group_layouts: &[&frame_bind_group_layout],
                push_constant_ranges: &[],
            });

            let frame_shader = device.create_shader_module(&wgpu::ShaderModuleDescriptor {
                label: None,
                source: wgpu::ShaderSource::Wgsl(include_str!("shader.wgsl").into()),
            });

            device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: None,
                layout: Some(&pipeline_layout),
                vertex: wgpu::VertexState {
                    module: &frame_shader,
                    entry_point: "vs_main",
                    buffers: &[],
                },
                fragment: Some(wgpu::FragmentState {
                    module: &frame_shader,
                    entry_point: "fs_main",
                    targets: &[texture_format.into()],
                }),
                primitive: wgpu::PrimitiveState::default(),
                depth_stencil: None,
                multisample: wgpu::MultisampleState::default(),
            })
        };

        let staging_belt = StagingBelt::new(1024);

        // Prepare glyph_brush
        let inconsolata = ab_glyph::FontArc::try_from_slice(include_bytes!(
            "../../assets/fonts/ReturnofGanon.ttf"
        ))
        .unwrap();

        let glyph_brush = GlyphBrushBuilder::using_font(inconsolata).build(&device, texture_format);

        WgpuState {
            instance,
            surface,
            texture_format,
            device,
            queue,

            frame,
            frame_bind_group,
            frame_view,

            staging_belt,
            glyph_brush,

            render_pipeline,

            size,
        }
    }
}

#[rustfmt::skip]
pub fn handle_input(input: &mut WinitInputHelper, io_sender: &Sender<IoEvent>, color_schemes_iter: Option<&mut ColorSchemeIter>) -> Result<(), Box<dyn std::error::Error>> {
    if let Some(color_schemes_iter) = color_schemes_iter {
        if input.key_pressed(VirtualKeyCode::Tab) { io_sender.send(IoEvent::SetColorScheme(color_schemes_iter.next().unwrap()))?; }
        if input.held_shift() && input.key_pressed(VirtualKeyCode::Tab) {
            for _ in 3..super::COLOR_SCHEMES.len() {
                color_schemes_iter.next().unwrap();
            }
            io_sender.send(IoEvent::SetColorScheme(color_schemes_iter.next().unwrap()))?;
        }
    }

    if input.key_pressed(VirtualKeyCode::F1) { io_sender.send(IoEvent::ToggleFPSLimit)?; }

    if input.key_pressed(VirtualKeyCode::Right)  { io_sender.send(IoEvent::ButtonPressed(JoypadButton::Right))?;  }
    if input.key_pressed(VirtualKeyCode::Left)   { io_sender.send(IoEvent::ButtonPressed(JoypadButton::Left))?;   }
    if input.key_pressed(VirtualKeyCode::Up)     { io_sender.send(IoEvent::ButtonPressed(JoypadButton::Up))?;     }
    if input.key_pressed(VirtualKeyCode::Down)   { io_sender.send(IoEvent::ButtonPressed(JoypadButton::Down))?;   }
    if input.key_pressed(VirtualKeyCode::Z)      { io_sender.send(IoEvent::ButtonPressed(JoypadButton::A))?;      }
    if input.key_pressed(VirtualKeyCode::X)      { io_sender.send(IoEvent::ButtonPressed(JoypadButton::B))?;      }
    if input.key_pressed(VirtualKeyCode::Space)  { io_sender.send(IoEvent::ButtonPressed(JoypadButton::Select))?; }
    if input.key_pressed(VirtualKeyCode::Return) { io_sender.send(IoEvent::ButtonPressed(JoypadButton::Start))?;  }

    if input.key_released(VirtualKeyCode::Right)  { io_sender.send(IoEvent::ButtonReleased(JoypadButton::Right))?;  }
    if input.key_released(VirtualKeyCode::Left)   { io_sender.send(IoEvent::ButtonReleased(JoypadButton::Left))?;   }
    if input.key_released(VirtualKeyCode::Up)     { io_sender.send(IoEvent::ButtonReleased(JoypadButton::Up))?;     }
    if input.key_released(VirtualKeyCode::Down)   { io_sender.send(IoEvent::ButtonReleased(JoypadButton::Down))?;   }
    if input.key_released(VirtualKeyCode::Z)      { io_sender.send(IoEvent::ButtonReleased(JoypadButton::A))?;      }
    if input.key_released(VirtualKeyCode::X)      { io_sender.send(IoEvent::ButtonReleased(JoypadButton::B))?;      }
    if input.key_released(VirtualKeyCode::Space)  { io_sender.send(IoEvent::ButtonReleased(JoypadButton::Select))?; }
    if input.key_released(VirtualKeyCode::Return) { io_sender.send(IoEvent::ButtonReleased(JoypadButton::Start))?;  }

    Ok(())
}

pub fn vm_loop(mut vm: VM, screen_sender: SyncSender<Screen>, io: Receiver<IoEvent>) {
    let mut clocks = 0;
    let clocks_to_run = (4194304.0 / 1000.0 * 16f64).round() as u32;

    let timer = speed_limit(Duration::from_millis(15));
    let mut respect_timer = true;

    'vm_loop: loop {
        while clocks < clocks_to_run {
            clocks += vm.tick() as u32;

            if vm.check_vblank() {
                if let Err(TrySendError::Disconnected(..)) = screen_sender.try_send(vm.get_screen())
                {
                    break;
                }
            }
        }

        loop {
            match io.try_recv() {
                Ok(event) => match event {
                    IoEvent::ButtonPressed(button) => vm.press_button(button),
                    IoEvent::ButtonReleased(button) => vm.release_button(button),
                    IoEvent::SetColorScheme(color_scheme) => vm.set_color_scheme(color_scheme),
                    IoEvent::ToggleFPSLimit => respect_timer ^= true,

                    IoEvent::Exit => break 'vm_loop,
                },
                Err(TryRecvError::Empty) => break,
                Err(_) => break 'vm_loop,
            }
        }

        clocks -= clocks_to_run;

        if respect_timer && timer.recv().is_err() {
            break;
        }
    }
}

fn speed_limit(wait_time: Duration) -> Receiver<()> {
    let (time_sender, time_receiver) = mpsc::sync_channel(1);
    thread::spawn(move || loop {
        thread::sleep(wait_time);
        if time_sender.send(()).is_err() {
            break;
        };
    });

    time_receiver
}
