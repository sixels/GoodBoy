use std::sync::mpsc;

use egui::{self, ClippedMesh, CtxRef};
use egui_wgpu_backend::{BackendError, RenderPass, ScreenDescriptor};

use pixels::PixelsContext;
use winit::window::Window;

mod widgets;

use crate::io::IoEvent;

use self::widgets::MenuBar;

// State for egui.
pub struct Framework {
    egui_ctx: CtxRef,
    egui_state: egui_winit::State,
    screen_descriptor: ScreenDescriptor,
    rpass: RenderPass,
    paint_jobs: Vec<ClippedMesh>,

    gui: Gui,
}

pub struct Gui {
    _io_sender: mpsc::Sender<IoEvent>,
    menu_bar: MenuBar,
}

impl Framework {
    pub fn new(
        width: u32,
        height: u32,
        scale_factor: f32,
        pixels: &pixels::Pixels,
        io_sender: mpsc::Sender<IoEvent>,
    ) -> Self {
        let egui_ctx = CtxRef::default();
        let egui_state = egui_winit::State::from_pixels_per_point(scale_factor);
        let screen_descriptor = ScreenDescriptor {
            physical_width: width,
            physical_height: height,
            scale_factor,
        };
        let rpass = RenderPass::new(pixels.device(), pixels.render_texture_format(), 1);

        let menu_bar = MenuBar::new(io_sender.clone());

        Self {
            egui_ctx,
            egui_state,
            screen_descriptor,
            rpass,
            paint_jobs: Vec::new(),

            gui: Gui {
                _io_sender: io_sender,
                menu_bar,
            },
        }
    }

    pub fn handle_event(&mut self, event: &winit::event::WindowEvent) {
        self.egui_state.on_event(&self.egui_ctx, event);
    }
    pub fn resize(&mut self, width: u32, height: u32) {
        if width > 0 && height > 0 {
            self.screen_descriptor.physical_width = width;
            self.screen_descriptor.physical_height = height;
        }
    }
    pub fn scale_factor(&mut self, scale_factor: f64) {
        self.screen_descriptor.scale_factor = scale_factor as f32;
    }
    pub fn prepare(&mut self, window: &Window) {
        // Run the egui frame and create all paint jobs to prepare for rendering.
        let raw_input = self.egui_state.take_egui_input(window);
        let (output, paint_commands) = self.egui_ctx.run(raw_input, |egui_ctx| {
            self.gui.update(&egui_ctx);
        });

        self.egui_state
            .handle_output(window, &self.egui_ctx, output);
        self.paint_jobs = self.egui_ctx.tessellate(paint_commands);
    }
    pub fn render(
        &mut self,
        encoder: &mut wgpu::CommandEncoder,
        render_target: &wgpu::TextureView,
        context: &PixelsContext,
    ) -> Result<(), BackendError> {
        // Upload all resources to the GPU.
        self.rpass
            .update_texture(&context.device, &context.queue, &self.egui_ctx.font_image());
        self.rpass
            .update_user_textures(&context.device, &context.queue);
        self.rpass.update_buffers(
            &context.device,
            &context.queue,
            &self.paint_jobs,
            &self.screen_descriptor,
        );

        // Record all render passes.
        self.rpass.execute(
            encoder,
            render_target,
            &self.paint_jobs,
            &self.screen_descriptor,
            None,
        )
    }
}

impl Gui {
    pub fn update(&mut self, ctx: &CtxRef) {
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| self.menu_bar.render(ui));

        // egui::CentralPanel::default().show(ctx, |ui| {
        //     match self.screen_chan.1.try_recv() {
        //         Ok(screen) => {
        //             self.fps.update();

        //             // let image = egui::Image::from_rgba_unmultiplied(
        //             //     [SCREEN_WIDTH as _, SCREEN_HEIGHT as _],
        //             //     screen.as_ref(),
        //             // );

        //             // self.display
        //             //     .replace(frame.alloc_texture(image))
        //             //     .map(|id| frame.free_texture(id));
        //         }
        //         // Err(mpsc::TryRecvError::Disconnected) => frame.quit(),
        //         _ => {}
        //     }

        //     let fps = self.fps.current_rate();
        //     ui.label(format!("FPS: {fps}"));

        //     self.display.map(|display| {
        //         ui.with_layout(
        //             egui::Layout::centered_and_justified(egui::Direction::TopDown),
        //             |ui| {
        //                 ui.image(display, [(SCREEN_WIDTH * 3) as _, (SCREEN_HEIGHT * 3) as _]);
        //             },
        //         );
        //     });

        //     io::handle_input(ui.input(), self.io_chan.0.clone());
        // });
        // ctx.request_repaint();
    }
}

// impl epi::App for App {
//     fn name(&self) -> &str {
//         "Good Boy 🐶"
//     }

//     fn setup(
//         &mut self,
//         _ctx: &egui::CtxRef,
//         _frame: &epi::Frame,
//         _storage: Option<&dyn epi::Storage>,
//     ) {
//         #[cfg(not(target_arch = "wasm32"))]
//         {
//             let screen_sender = self.screen_chan.0.clone();
//             let io_receiver = self.io_chan.1.take().unwrap();

//             let vm = self.vm.take();

//             self.vm_loop_handle = Some(thread::spawn(move || {
//                 let screen_sender_clone = screen_sender.clone();
//                 thread::spawn(move || vm_loop(vm, screen_sender_clone, io_receiver));
//             }));
//         }
//     }

//     fn on_exit(&mut self) {
//         self.io_chan.0.send(IoEvent::Exit).ok();

//         #[cfg(not(target_arch = "wasm32"))]
//         self.vm_loop_handle.take().map(thread::JoinHandle::join);
//     }

//     fn update(&mut self, ctx: &egui::CtxRef, frame: &epi::Frame) {
//         #[cfg(target_arch = "wasm32")]
//         {
//             let screen_sender = self.screen_chan.0.clone();
//             let io_receiver = self.io_chan.1.as_ref().unwrap();

//             update_vm(&mut self.vm, screen_sender, io_receiver, 0, None).ok();
//         }

//         egui::TopBottomPanel::top("top_panel").show(ctx, |ui| self.menu_bar.render(ui, frame));

//         egui::CentralPanel::default().show(ctx, |ui| {
//             match self.screen_chan.1.try_recv() {
//                 Ok(screen) => {
//                     self.fps.update();

//                     let image = epi::Image::from_rgba_unmultiplied(
//                         [SCREEN_WIDTH as _, SCREEN_HEIGHT as _],
//                         screen.as_ref(),
//                     );

//                     self.display
//                         .replace(frame.alloc_texture(image))
//                         .map(|id| frame.free_texture(id));
//                 }
//                 Err(mpsc::TryRecvError::Disconnected) => frame.quit(),
//                 _ => {}
//             }

//             let fps = self.fps.current_rate();
//             ui.label(format!("FPS: {fps}"));

//             self.display.map(|display| {
//                 ui.with_layout(
//                     egui::Layout::centered_and_justified(egui::Direction::TopDown),
//                     |ui| {
//                         ui.image(display, [(SCREEN_WIDTH * 3) as _, (SCREEN_HEIGHT * 3) as _]);
//                     },
//                 );
//             });

//             io::handle_input(ui.input(), self.io_chan.0.clone());
//         });
//         ctx.request_repaint();
//     }
// }
