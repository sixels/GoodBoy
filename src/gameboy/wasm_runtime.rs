use std::{
    sync::mpsc::{self, TryRecvError, TrySendError},
    thread,
    time::{Duration, Instant},
};

use goodboy_core::vm::{SCREEN_HEIGHT, SCREEN_WIDTH, Screen, VM};
use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    platform::web::WindowExtWebSys,
    window::Window,
};
use winit_input_helper::WinitInputHelper;
use wgpu_glyph::{ab_glyph, GlyphBrushBuilder, Section, Text};

use super::{common, ColorSchemeIter, IoEvent};


use wasm_bindgen::prelude::wasm_bindgen;

#[wasm_bindgen]
extern "C" {
    type Date;

    #[wasm_bindgen(static_method_of = Date)]
    pub fn now() -> f64;
}

pub async fn run(window: Window, event_loop: EventLoop<()>, mut vm: VM) {
    let mut input = WinitInputHelper::new();


    let instance = wgpu::Instance::new(wgpu::Backends::all());
    let surface = unsafe { instance.create_surface(&window) };


    let adapter = instance.request_adapter(&wgpu::RequestAdapterOptions {
        power_preference: wgpu::PowerPreference::HighPerformance,
        compatible_surface: Some(&surface),
        force_fallback_adapter: false,
    }).await
    .expect("Could not request the adapter");

    let (device, queue) =
        adapter.request_device(&wgpu::DeviceDescriptor {
                label: None,
                features: wgpu::Features::empty(),
                // Make sure we use the texture resolution limits from the adapter, so we can support images the size of the swapchain.
                limits: wgpu::Limits::downlevel_webgl2_defaults()
                    .using_resolution(adapter.limits()),
            },
            None).await
            .expect("Could not request a device");

    let render_format = surface.get_preferred_format(&adapter).unwrap();

    let mut size = window.inner_size();

    surface.configure(
        &device,
        &wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: render_format,
            width: SCREEN_WIDTH as _,
            height: SCREEN_HEIGHT as _,
            present_mode: wgpu::PresentMode::Mailbox,
        },
    );

    let mut staging_belt = wgpu::util::StagingBelt::new(1024);

    let img = device.create_texture(&wgpu::TextureDescriptor {
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
    let img_view = img.create_view(&Default::default());

    let img_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
        address_mode_u: wgpu::AddressMode::ClampToEdge,
        address_mode_v: wgpu::AddressMode::ClampToEdge,
        address_mode_w: wgpu::AddressMode::ClampToEdge,
        mag_filter: wgpu::FilterMode::Linear,
        min_filter: wgpu::FilterMode::Nearest,
        mipmap_filter: wgpu::FilterMode::Nearest,
        ..Default::default()
    });

    let img_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
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
                    // This is only for TextureSampleType::Depth
                    comparison: false,
                    // This should be true if the sample_type of the texture is:
                    //     TextureSampleType::Float { filterable: true }
                    // Otherwise you'll get an error.
                    filtering: true,
                },
                count: None,
            },
        ],
        label: Some("img_bind_group_layout"),
    });

    let img_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        layout: &img_bind_group_layout,
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::TextureView(&img_view),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: wgpu::BindingResource::Sampler(&img_sampler),
            },
        ],
        label: Some("img_bind_group"),
    });

    let img_shader = device.create_shader_module(&wgpu::ShaderModuleDescriptor {
        label: None,
        source: wgpu::ShaderSource::Wgsl(include_str!("shader.wgsl").into()),
    });
    let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: None,
        bind_group_layouts: &[&img_bind_group_layout],
        push_constant_ranges: &[],
    });
    let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: None,
        layout: Some(&pipeline_layout),
        vertex: wgpu::VertexState {
            module: &img_shader,
            entry_point: "vs_main",
            buffers: &[],
        },
        fragment: Some(wgpu::FragmentState {
            module: &img_shader,
            entry_point: "fs_main",
            targets: &[render_format.into()],
        }),
        primitive: wgpu::PrimitiveState::default(),
        depth_stencil: None,
        multisample: wgpu::MultisampleState::default(),
    });

    let font =
        ab_glyph::FontArc::try_from_slice(include_bytes!("../../assets/fonts/ReturnofGanon.ttf"))
            .expect("Could not open the font");

    let mut glyph_brush = GlyphBrushBuilder::using_font(font).build(&device, render_format);
    log::info!("canvas context created");


    let (screen_sender, screen_receiver): (mpsc::SyncSender<Screen>, mpsc::Receiver<Screen>) = mpsc::sync_channel(1);
    let (io_sender, io_receiver) = mpsc::channel();


    let mut color_schemes_iter: ColorSchemeIter = box super::COLOR_SCHEMES.iter().copied().cycle();
    let mut clocks = 0;

    // context.clear_rect(0.0, 0.0, SCREEN_WIDTH as _, SCREEN_HEIGHT as _);

    let mut start = Date::now();
    let mut text_fps = String::from("FPS: 0");
    let mut fps = 0;

    window.request_redraw();

    log::info!("Starting the event loop");
    event_loop.run(move |event, _, control_flow| {
        // VM loop
        {
            let clocks_to_run = (4194304.0 / 1000.0f64 * 11.0).round() as u32;

            while clocks < clocks_to_run {
                clocks += vm.tick() as u32;

                if vm.check_vblank() {
                    if let Err(TrySendError::Disconnected(..)) =
                        screen_sender.try_send(vm.get_screen())
                    {
                        break;
                    }
                }
            }
            clocks -= clocks_to_run;

            loop {
                match io_receiver.try_recv() {
                    Ok(event) => match event {
                        IoEvent::ButtonPressed(button) => vm.press_button(button),
                        IoEvent::ButtonReleased(button) => vm.release_button(button),
                        IoEvent::SetColorScheme(color_scheme) => vm.set_color_scheme(color_scheme),
                        IoEvent::ToggleFPSLimit => (),

                        IoEvent::Exit => break,
                    },
                    Err(TryRecvError::Empty) => break,
                    Err(_) => break,
                }
            }
        }

        let now = Date::now();
        if now > start + Duration::from_secs(1000).as_secs_f64() {
            start = now;
            text_fps = format!("FPS: {}", fps);
            fps = 0
        }

        match event {
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => {
                *control_flow = ControlFlow::Exit;
            }

            Event::RedrawRequested(..) | Event::MainEventsCleared => {
                let screen = match screen_receiver.try_recv() {
                    Ok(data) => Some(data),
                    Err(mpsc::TryRecvError::Empty) => None,
                    Err(_) => {
                        *control_flow = ControlFlow::Exit;
                        None
                    }
                };

                if let Some(screen) = &screen {
                    // render the screen
                    let mut encoder =
                        device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
                            label: Some("Redraw"),
                        });

                    // Get the next frame
                    let frame = surface
                        .get_current_texture()
                        .expect("Could not get the next frame");
                    let view = frame
                        .texture
                        .create_view(&wgpu::TextureViewDescriptor::default());

                    // staging_belt.write_buffer(encoder, target, offset, size, device)

                    // render the game frame
                    {
                        queue.write_texture(
                            // Tells wgpu where to copy the pixel data
                            wgpu::ImageCopyTexture {
                                texture: &img,
                                mip_level: 0,
                                origin: wgpu::Origin3d::ZERO,
                                aspect: wgpu::TextureAspect::All,
                            },
                            // The actual pixel data
                            screen.as_ref(),
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

                        rpass.set_pipeline(&render_pipeline);
                        rpass.set_bind_group(0, &img_bind_group, &[]);
                        rpass.draw(0..3, 0..2);
                    }

                    // render the FPS counter
                    {
                        glyph_brush.queue(Section {
                            screen_position: (5.0, 5.0),
                            bounds: (size.width as f32, size.height as f32),
                            text: vec![Text::new(&text_fps)
                                .with_color([1.0, 1.0, 1.0, 1.0])
                                .with_scale(30.0)],
                            ..Section::default()
                        });
                        

                        glyph_brush
                            .draw_queued(
                                &device,
                                &mut staging_belt,
                                &mut encoder,
                                &view,
                                size.width,
                                size.height,
                            )
                            .expect("Draw queued");

                        staging_belt.finish();
                    }

                    queue.submit(Some(encoder.finish()));
                    frame.present();

                    // staging_belt.recall().await;

                    fps += 1;
                }
            }

            _ => (),
        };

        if *control_flow != ControlFlow::Exit && input.update(&event) {
            if let Some(new_size) = input.window_resized() {
                size = new_size;

                surface.configure(
                    &device,
                    &wgpu::SurfaceConfiguration {
                        usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
                        format: render_format,
                        width: size.width,
                        height: size.height,
                        present_mode: wgpu::PresentMode::Mailbox,
                    },
                );
            }
            
            if common::handle_input(&mut input, &io_sender, Some(&mut color_schemes_iter)).is_err()
            {
                *control_flow = ControlFlow::Exit;
            }
        }

        // Drop the vm before exit
        if *control_flow == ControlFlow::Exit {
            io_sender.send(IoEvent::Exit).ok();
            thread::sleep(Duration::from_millis(100));
        }

      
    });
}
