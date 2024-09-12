pub struct MyThreeApp {
    pub angle: f32,
}

impl MyThreeApp {
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        Self { angle: 0.2 }
    }
}

pub fn three_d_draw(this: &mut MyThreeApp, ui: &mut Ui) {
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 0.0;
        ui.label("The triangle is being painted using ");
        ui.hyperlink_to("three-d", "https://github.com/asny/three-d");
        ui.label(".");
    });

    egui::ScrollArea::both().show(ui, |ui| {
        egui::Frame::canvas(ui.style()).show(ui, |ui| {
            // let (rect, response) = ui.allocate_exact_size(egui::Vec2::splat(512.0), egui::Sense::drag());

            let (rect, response) = ui.allocate_at_least(ui.available_size(), egui::Sense::drag());

            this.angle += response.drag_delta().x * 0.01;

            // Clone locals so we can move them into the paint callback:
            let angle = this.angle;

            let mut events = vec![];

            let egui_events = ui.ctx().input(|s| s.raw.events.clone());

            let mouse_pos = ui.ctx().pointer_hover_pos();

            // for ev in &egui_events {
            //     // let ff: Event =
            //     match ev {
            //         // egui::Event::Key { key, physical_key, pressed, repeat, modifiers } => todo!(),
            //         // egui::Event::PointerMoved(_) => todo!(),
            //         // egui::Event::PointerButton { pos, button, pressed, modifiers } => todo!(),
            //         // egui::Event::PointerGone => todo!(),

            //         // egui::Event::Scroll(_) => todo!(),
            //         // egui::Event::Zoom(_) => todo!(),
            //         egui::Event::MouseWheel {
            //             unit,
            //             delta,
            //             modifiers,
            //         } => {
            //             if let Some(pos) = mouse_pos {
            //                 let p = Event::MouseWheel {
            //                     delta: delta.into(),
            //                     position: LogicalPoint {
            //                         x: pos.x,
            //                         y: pos.y,
            //                         device_pixel_ratio: 1.5,
            //                         height: 800.,
            //                     },

            //                     modifiers: Modifiers {
            //                         alt: false,
            //                         ctrl: false,
            //                         shift: false,
            //                         command: false,
            //                     },
            //                     handled: false,
            //                 };

            //                 events.push(p);
            //             };
            //         }
            //         _ => {}
            //     };
            // }

            // let d: LogicalPoint

            let callback = egui::PaintCallback {
                rect,
                callback: std::sync::Arc::new(egui_glow::CallbackFn::new(move |info, painter| {
                    with_three_d(painter.gl(), |three_d| {
                        three_d.frame(
                            FrameInput::new(&three_d.context, &info, painter, events.clone()),
                            angle,
                        );
                    });
                })),
            };
            ui.painter().add(callback);
        });
        // ui.label("Drag to rotate!");
    });
}

/// We get a [`glow::Context`] from `eframe` and we want to construct a [`ThreeDApp`].
///
/// Sadly we can't just create a [`ThreeDApp`] in [`MyApp::new`] and pass it
/// to the [`egui::PaintCallback`] because [`glow::Context`] isn't `Send+Sync` on web, which
/// [`egui::PaintCallback`] needs. If you do not target web, then you can construct the [`ThreeDApp`] in [`MyApp::new`].
fn with_three_d<R>(
    gl: &std::sync::Arc<egui_glow::glow::Context>,
    f: impl FnOnce(&mut ThreeDApp) -> R,
) -> R {
    use std::cell::RefCell;
    thread_local! {
        pub static THREE_D: RefCell<Option<ThreeDApp>> = RefCell::new(None);
    }

    THREE_D.with(|three_d| {
        let mut three_d = three_d.borrow_mut();
        let three_d = three_d.get_or_insert_with(|| ThreeDApp::new(gl.clone()));
        f(three_d)
    })
}

///
/// Translates from egui input to three-d input
///
pub struct FrameInput<'a> {
    screen: three_d::RenderTarget<'a>,
    viewport: three_d::Viewport,
    scissor_box: three_d::ScissorBox,

    events: Vec<Event>,
}

impl FrameInput<'_> {
    pub fn new(
        context: &three_d::Context,
        info: &egui::PaintCallbackInfo,
        painter: &egui_glow::Painter,
        events: Vec<Event>,
    ) -> Self {
        use three_d::*;

        // Disable sRGB textures for three-d
        #[cfg(not(target_arch = "wasm32"))]
        #[allow(unsafe_code)]
        unsafe {
            use egui_glow::glow::HasContext as _;
            context.disable(egui_glow::glow::FRAMEBUFFER_SRGB);
        }

        // Constructs a screen render target to render the final image to
        let screen = painter.intermediate_fbo().map_or_else(
            || {
                RenderTarget::screen(
                    context,
                    info.viewport.width() as u32,
                    info.viewport.height() as u32,
                )
            },
            |fbo| {
                RenderTarget::from_framebuffer(
                    context,
                    info.viewport.width() as u32,
                    info.viewport.height() as u32,
                    fbo,
                )
            },
        );

        // Set where to paint
        let viewport = info.viewport_in_pixels();
        let viewport = Viewport {
            // x: viewport.left_px.round() as _,
            // y: viewport.from_bottom_px.round() as _,
            // width: viewport.width_px.round() as _,
            // height: viewport.height_px.round() as _,
            x: viewport.left_px,
            y: viewport.from_bottom_px,
            width: viewport.width_px as u32,
            height: viewport.height_px as u32,
        };

        // Respect the egui clip region (e.g. if we are inside an `egui::ScrollArea`).
        let clip_rect = info.clip_rect_in_pixels();
        let scissor_box = ScissorBox {
            // x: clip_rect.left_px.round() as _,
            // y: clip_rect.from_bottom_px.round() as _,
            // width: clip_rect.width_px.round() as _,
            // height: clip_rect.height_px.round() as _,
            x: clip_rect.left_px,
            y: clip_rect.from_bottom_px,
            width: clip_rect.width_px as u32,
            height: clip_rect.height_px as u32,
        };
        Self {
            events,
            screen,
            scissor_box,
            viewport,
        }
    }
}

use egui::Ui;
///
/// Based on the `three-d` [Triangle example](https://github.com/asny/three-d/blob/master/examples/triangle/src/main.rs).
/// This is where you'll need to customize
///
use three_d::*;
pub struct ThreeDApp {
    context: Context,
    camera: Camera,
    control: OrbitControl,
    model: Gm<Mesh, ColorMaterial>,
}

impl ThreeDApp {
    pub fn new(gl: std::sync::Arc<egui_glow::glow::Context>) -> Self {
        let context = Context::from_gl_context(gl).unwrap();
        // Create a camera

        let camera = Camera::new_perspective(
            // window.viewport(),
            Viewport::new_at_origo(1, 1),
            vec3(4.0, 4.0, 5.0),
            vec3(0.0, 0.0, 0.0),
            vec3(0.0, 1.0, 0.0),
            degrees(45.0),
            0.1,
            1000.0,
        );

        // let mut control = OrbitControl::new(*camera.target(), 1.0, 100.0);

        // Create a CPU-side mesh consisting of a single colored triangle
        let positions = vec![
            vec3(0.5, -0.5, 0.0),  // bottom right
            vec3(-0.5, -0.5, 0.0), // bottom left
            vec3(0.0, 0.5, 0.0),   // top
        ];

        let colors = vec![
            three_d::Srgba::new(255, 0, 0, 255), // bottom right
            three_d::Srgba::new(0, 255, 0, 255), // bottom left
            three_d::Srgba::new(0, 0, 255, 255), // top
        ];
        let cpu_mesh = CpuMesh {
            positions: Positions::F32(positions),
            colors: Some(colors),
            ..Default::default()
        };

        // Construct a model, with a default color material, thereby transferring the mesh data to the GPU
        let model = Gm::new(Mesh::new(&context, &cpu_mesh), ColorMaterial::default());

        let scene_radius = 6.0;

        let mut control =
            OrbitControl::new(*camera.target(), 0.1 * scene_radius, 100.0 * scene_radius);

        Self {
            context,
            camera,
            model,
            control,
        }
    }

    pub fn frame(
        &mut self,
        mut frame_input: FrameInput<'_>,
        angle: f32,
    ) -> Option<egui_glow::glow::Framebuffer> {
        // Ensure the viewport matches the current window viewport which changes if the window is resized
        let mut redraw = self.camera.set_viewport(frame_input.viewport);

        // let mut redraw = frame_input.first_frame;
        // redraw |= camera.set_viewport(frame_input.viewport);
        // redraw |=
        self.control
            .handle_events(&mut self.camera, &mut frame_input.events);

        // Set the current transformation of the triangle
        self.model
            .set_transformation(Mat4::from_angle_y(radians(angle)));

        // Get the screen render target to be able to render something on the screen

        let cylinder = Gm::new(
            Mesh::new(&self.context, &CpuMesh::cylinder(1000)),
            PhysicalMaterial::new_transparent(
                &self.context,
                &CpuMaterial {
                    albedo: three_d::Srgba {
                        r: 0,
                        g: 255,
                        b: 0,
                        a: 200,
                    },
                    ..Default::default()
                },
            ),
        );
        // cylinder.set_transformation(Mat4::from_translation(vec3(1.3, 0.0, 0.0)) * Mat4::from_scale(0.2));

        let ambient = AmbientLight::new(&self.context, 0.4, three_d::Srgba::WHITE);
        let directional = DirectionalLight::new(
            &self.context,
            2.0,
            three_d::Srgba::WHITE,
            &vec3(-1.0, -1.0, -1.0),
        );

        // &cylinder

        frame_input
            .screen
            // Clear the color and depth of the screen render target
            .clear_partially(frame_input.scissor_box, ClearState::depth(1.0))
            // Render the triangle with the color material which uses the per vertex colors defined at construction
            .render_partially(
                frame_input.scissor_box,
                &self.camera,
                &[&self.model],
                &[&ambient, &directional],
            )
            .render_partially(
                frame_input.scissor_box,
                &self.camera,
                &[&cylinder],
                &[&ambient, &directional],
            );

        frame_input.screen.into_framebuffer() // Take back the screen fbo, we will continue to use it.
    }
}
