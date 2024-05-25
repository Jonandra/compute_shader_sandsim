mod camera;
mod gui;
mod matter;
mod particle_simulator;
mod quad_pipeline;
mod render;
mod utils;
mod vertex;

use bevy::{
    input::mouse::MouseWheel,
    prelude::*,
    time::FixedTimestep,
    window::{close_on_esc, WindowMode},
};

use bevy_vulkano::{BevyVulkanoWindows, VulkanoWinitConfig, VulkanoWinitPlugin};
use vulkano::sync::GpuFuture;

use crate::{
    camera::OrthographicCamera,
    gui::user_interface,
    matter::MatterId,
    particle_simulator::CASimulator,
    render::FillScreenRenderPass,
    utils::{cursor_to_world, get_canvas_line, MousePos},
};

//CONSTANTS
//screen and camera constants
pub const WIDTH: f32 = 1024.0;
pub const HEIGHT: f32 = 1024.0;
pub const CLEAR_COLOR: [f32; 4] = [1.0; 4];
pub const CAMERA_MOVE_SPEED: f32 = 200.0;

//gpu multithreading constants
pub const CANVAS_SIZE_X: u32 = 1024;
pub const CANVAS_SIZE_Y: u32 = 1024;
pub const LOCAL_SIZE_X: u32 = 32;
pub const LOCAL_SIZE_Y: u32 = 32;
pub const NUM_WORK_GROUPS_X: u32 = CANVAS_SIZE_X / LOCAL_SIZE_X;
pub const NUM_WORK_GROUPS_Y: u32 = CANVAS_SIZE_Y / LOCAL_SIZE_Y;

//game constants
pub const SIM_FPS: f64 = 60.0;

// Creates our simulation and render pipelines
fn setup(mut commands: Commands, vulkano_windows: NonSend<BevyVulkanoWindows>) {
    let (primary_window_renderer, _gui) = vulkano_windows.get_primary_window_renderer().unwrap();
    // Create our render pass
    let fill_screen = FillScreenRenderPass::new(
        primary_window_renderer.graphics_queue(),
        primary_window_renderer.swapchain_format(),
    );
    // Create simple orthographic camera
    let mut camera = OrthographicCamera::default();
    camera.zoom_to_fit_vertical_pixels(CANVAS_SIZE_Y, HEIGHT as u32);

    let simulator = CASimulator::new(primary_window_renderer.compute_queue());

    // Insert resources
    commands.insert_resource(DynamicSettings::default());

    commands.insert_resource(PreviousMousePos(None));
    commands.insert_resource(CurrentMousePos(None));

    commands.insert_resource(simulator);
    commands.insert_resource(camera);

    commands.insert_resource(fill_screen);
}

// Render the simulation
fn render(
    mut vulkano_windows: NonSendMut<BevyVulkanoWindows>,
    mut fill_screen: ResMut<FillScreenRenderPass>,
    simulator: Res<CASimulator>,
    camera: Res<OrthographicCamera>,
) {
    // Access our window renderer and gui
    let (window_renderer, gui) = vulkano_windows.get_primary_window_renderer_mut().unwrap();
    // Start frame
    let before: Box<dyn GpuFuture> = match window_renderer.acquire() {
        Err(e) => {
            bevy::log::error!("Failed to start frame: {}", e);
            return;
        }
        Ok(f) => f,
    };
    let canvas_image = simulator.color_image();
    // Access the final window image (this is the current GPU image which changes between frames)
    let final_image = window_renderer.swapchain_image_view();
    let after_images = fill_screen.draw(
        before,
        *camera,
        canvas_image,
        final_image.clone(),
        CLEAR_COLOR,
        false,
        true,
    );
    // Draw GUI using egui_winit_window's GUI draw pipeline
    let after_gui = gui.draw_on_image(after_images, final_image);

    // Finish Frame
    window_renderer.present(after_gui, true);
}

fn main() {
    //bevy initialization
    //this takes care of window initialization, input, game core loop etc
    App::new()
        .insert_non_send_resource(VulkanoWinitConfig::default())
        .insert_resource(WindowDescriptor {
            width: WIDTH,
            height: HEIGHT,
            title: "ParticleSimulator".to_string(),
            present_mode: bevy::window::PresentMode::Immediate,
            resizable: true,
            mode: WindowMode::Windowed,
            ..WindowDescriptor::default()
        })
        .add_plugin(bevy::core::CorePlugin)
        .add_plugin(bevy::log::LogPlugin)
        .add_plugin(bevy::time::TimePlugin)
        .add_plugin(bevy::diagnostic::DiagnosticsPlugin)
        .add_plugin(bevy::diagnostic::FrameTimeDiagnosticsPlugin)
        .add_plugin(bevy::input::InputPlugin)
        .add_plugin(VulkanoWinitPlugin)
        .add_system(close_on_esc)
        .add_startup_system(setup)
        .add_system(user_interface)
        .add_system(input_actions)
        .add_system(update_camera)
        .add_system(update_mouse)
        .add_system(draw_matter)
        .add_system_set_to_stage(
            CoreStage::Update,
            SystemSet::new()
                .with_run_criteria(FixedTimestep::steps_per_second(SIM_FPS))
                .with_system(simulate),
        )
        .add_system_to_stage(CoreStage::PostUpdate, render)
        .run();
}

// Step simulation
fn simulate(mut sim_pipeline: ResMut<CASimulator>, settings: Res<DynamicSettings>) {
    sim_pipeline.step(1, settings.is_paused);
}

// Update camera (if window is resized)
fn update_camera(windows: Res<Windows>, mut camera: ResMut<OrthographicCamera>) {
    let window = windows.get_primary().unwrap();
    camera.update(window.width(), window.height());
}

// Input actions for camera movement, zoom and pausing
fn input_actions(
    time: Res<Time>,
    mut camera: ResMut<OrthographicCamera>,
    keyboard_input: Res<Input<KeyCode>>,
    mut mouse_input_events: EventReader<MouseWheel>,
) {
    // Move camera with arrows and WASD
    let up = keyboard_input.pressed(KeyCode::W) || keyboard_input.pressed(KeyCode::Up);
    let down = keyboard_input.pressed(KeyCode::S) || keyboard_input.pressed(KeyCode::Down);
    let left = keyboard_input.pressed(KeyCode::A) || keyboard_input.pressed(KeyCode::Left);
    let right = keyboard_input.pressed(KeyCode::D) || keyboard_input.pressed(KeyCode::Right);

    let x_axis = -(right as i8) + left as i8;
    let y_axis = -(up as i8) + down as i8;

    let mut move_delta = Vec2::new(x_axis as f32, y_axis as f32);
    if move_delta != Vec2::ZERO {
        move_delta /= move_delta.length();
        camera.pos += move_delta * time.delta_seconds() * CAMERA_MOVE_SPEED;
    }

    // Zoom camera with mouse scroll
    for e in mouse_input_events.iter() {
        if e.y < 0.0 {
            camera.scale *= 1.05;
        } else {
            camera.scale *= 1.0 / 1.05;
        }
    }
}

// Mouse position from last frame
#[derive(Debug, Copy, Clone)]
pub struct PreviousMousePos(pub Option<MousePos>);

// Mouse position now
#[derive(Debug, Copy, Clone)]
pub struct CurrentMousePos(pub Option<MousePos>);

// Update mouse position
fn update_mouse(
    windows: Res<Windows>,
    mut _prev: ResMut<PreviousMousePos>,
    mut _current: ResMut<CurrentMousePos>,
    camera: Res<OrthographicCamera>,
) {
    _prev.0 = _current.0;
    let primary = windows.get_primary().unwrap();
    if primary.cursor_position().is_some() {
        _current.0 = Some(MousePos {
            world: cursor_to_world(primary, camera.pos, camera.scale),
        });
    }
}

//draw to canvas

fn draw_matter(
    mut simulator: ResMut<CASimulator>,
    prev: Res<PreviousMousePos>,
    current: Res<CurrentMousePos>,
    mouse_button_input: Res<Input<MouseButton>>,
    settings: Res<DynamicSettings>,
) {
    if let Some(current) = current.0 {
        if mouse_button_input.pressed(MouseButton::Left) {
            let line = get_canvas_line(prev.0, current);
            // Draw
            simulator.draw_matter(&line, settings.brush_radius, settings.draw_matter);
        }
    }
}

//Drawing settings
pub struct DynamicSettings {
    pub brush_radius: f32,
    pub draw_matter: MatterId,
    pub is_paused: bool,
}

impl Default for DynamicSettings {
    fn default() -> Self {
        Self {
            brush_radius: 4.0,
            draw_matter: MatterId::Sand,
            is_paused: false,
        }
    }
}
