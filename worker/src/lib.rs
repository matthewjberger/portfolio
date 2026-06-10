mod ecs;
mod state;
mod systems;

use std::cell::RefCell;
use std::rc::Rc;

use nightshade::prelude::winit::event::{
    ElementState as WinitElementState, MouseButton as WinitMouseButton,
};
use nightshade::prelude::*;
use nightshade::render::wgpu::create_wgpu_renderer;
use protocol::{CANVAS_KEY, ClientMessage, MESSAGE_KEY, WorkerMessage};
use wasm_bindgen::prelude::*;
use wasm_bindgen::{JsCast, JsValue};
use wasm_bindgen_futures::spawn_local;
use web_sys::{DedicatedWorkerGlobalScope, MessageEvent, OffscreenCanvas};

use crate::state::Portfolio;

type AppSlot = Rc<RefCell<Option<App>>>;

struct App {
    world: World,
    renderer: WgpuRenderer,
    state: Portfolio,
}

#[wasm_bindgen(start)]
pub fn start() {
    console_error_panic_hook::set_once();

    let scope: DedicatedWorkerGlobalScope = js_sys::global().unchecked_into();
    let app_slot: AppSlot = Rc::new(RefCell::new(None));

    let handler_scope = scope.clone();
    let onmessage = Closure::<dyn FnMut(MessageEvent)>::new(move |event: MessageEvent| {
        handle_message(&handler_scope, &app_slot, event);
    });
    scope.set_onmessage(Some(onmessage.as_ref().unchecked_ref()));
    onmessage.forget();
}

fn handle_message(scope: &DedicatedWorkerGlobalScope, app_slot: &AppSlot, event: MessageEvent) {
    let data = event.data();
    let Ok(payload) = js_sys::Reflect::get(&data, &JsValue::from_str(MESSAGE_KEY)) else {
        return;
    };
    let Ok(message) = serde_wasm_bindgen::from_value::<ClientMessage>(payload) else {
        return;
    };

    match message {
        ClientMessage::Init { width, height } => {
            let Some(canvas) = canvas_from(&data) else {
                return;
            };
            let scope = scope.clone();
            let app_slot = app_slot.clone();
            spawn_local(async move {
                let app = create_app(canvas, width, height).await;
                post(&WorkerMessage::Ready);
                *app_slot.borrow_mut() = Some(app);
                start_render_loop(scope, app_slot);
            });
        }
        ClientMessage::Resize { width, height } => {
            if let Some(app) = app_slot.borrow_mut().as_mut() {
                let physical_width = (width as u32).max(1);
                let physical_height = (height as u32).max(1);
                resize_offscreen(
                    &mut app.world,
                    &mut app.renderer,
                    physical_width,
                    physical_height,
                );
                app.world.resources.window.active_viewport_rect =
                    Some(nightshade::ecs::window::resources::ViewportRect {
                        x: 0.0,
                        y: 0.0,
                        width: physical_width as f32,
                        height: physical_height as f32,
                    });
            }
        }
        other => {
            if let Some(app) = app_slot.borrow_mut().as_mut() {
                apply_client_message(&mut app.world, &mut app.state, other);
            }
        }
    }
}

fn apply_client_message(world: &mut World, portfolio: &mut Portfolio, message: ClientMessage) {
    match message {
        ClientMessage::PointerMove { x, y } => {
            input_inject_cursor_moved(world, Vec2::new(x, y));
        }
        ClientMessage::PointerButton { button, pressed } => {
            let state = if pressed {
                WinitElementState::Pressed
            } else {
                WinitElementState::Released
            };
            input_inject_mouse_button(world, mouse_button(button), state);
        }
        ClientMessage::Wheel { delta } => {
            input_inject_mouse_wheel(world, Vec2::new(0.0, -delta / 100.0));
        }
        ClientMessage::Touch { id, phase, x, y } => {
            input_inject_touch(world, id, touch_phase(phase), Vec2::new(x, y));
        }
        ClientMessage::Scroll { progress } => {
            portfolio.portfolio.resources.tour.target_progress = progress.clamp(0.0, 1.0);
        }
        ClientMessage::Orbit { yaw, pitch, zoom } => {
            if let Some(camera) = world.resources.active_camera
                && let Some(orbit) = world.core.get_pan_orbit_camera_mut(camera)
            {
                orbit.target_yaw += yaw;
                orbit.target_pitch = (orbit.target_pitch + pitch).clamp(-1.4, 1.4);
                orbit.target_radius = (orbit.target_radius * (1.0 + zoom)).clamp(8.0, 60.0);
            }
        }
        ClientMessage::Glance { x, y } => {
            portfolio.portfolio.resources.tour.glance =
                Vec2::new(x.clamp(-1.0, 1.0), y.clamp(-1.0, 1.0));
        }
        ClientMessage::SetReducedMotion { enabled } => {
            portfolio.portfolio.resources.tour.reduced_motion = enabled;
        }
        ClientMessage::Game { command } => {
            systems::game::apply(&mut portfolio.portfolio, world, command);
        }
        ClientMessage::Init { .. } | ClientMessage::Resize { .. } => {}
    }
}

async fn create_app(canvas: OffscreenCanvas, width: f32, height: f32) -> App {
    let physical_width = (width as u32).max(1);
    let physical_height = (height as u32).max(1);

    let surface_target = wgpu::SurfaceTarget::OffscreenCanvas(canvas);
    let mut renderer = create_wgpu_renderer(surface_target, physical_width, physical_height)
        .await
        .expect("failed to create renderer from offscreen canvas");

    let mut world = World::default();
    let mut state = Portfolio::default();
    initialize_offscreen(
        &mut world,
        &mut state,
        &mut renderer,
        (physical_width, physical_height),
        1.0,
    );

    App {
        world,
        renderer,
        state,
    }
}

fn start_render_loop(_scope: DedicatedWorkerGlobalScope, app_slot: AppSlot) {
    let last_push = Rc::new(RefCell::new(0.0_f64));

    spawn_animation_frame_loop(move || {
        if let Some(app) = app_slot.borrow_mut().as_mut() {
            tick_offscreen(&mut app.world, &mut app.state, &mut app.renderer);
            let scope: DedicatedWorkerGlobalScope = js_sys::global().unchecked_into();
            if let Some(performance) = scope.performance() {
                let now = performance.now();
                let mut last = last_push.borrow_mut();
                if now - *last > 500.0 {
                    *last = now;
                    post(&WorkerMessage::Stats {
                        fps: app.world.resources.window.timing.frames_per_second,
                    });
                }
            }
        }
    });
}

fn mouse_button(button: u8) -> WinitMouseButton {
    match button {
        1 => WinitMouseButton::Middle,
        2 => WinitMouseButton::Right,
        _ => WinitMouseButton::Left,
    }
}

fn touch_phase(phase: protocol::TouchPhase) -> TouchPhase {
    match phase {
        protocol::TouchPhase::Started => TouchPhase::Started,
        protocol::TouchPhase::Moved => TouchPhase::Moved,
        protocol::TouchPhase::Ended => TouchPhase::Ended,
        protocol::TouchPhase::Cancelled => TouchPhase::Cancelled,
    }
}

fn canvas_from(data: &JsValue) -> Option<OffscreenCanvas> {
    js_sys::Reflect::get(data, &JsValue::from_str(CANVAS_KEY))
        .ok()
        .and_then(|value| value.dyn_into::<OffscreenCanvas>().ok())
}

pub(crate) fn post(message: &WorkerMessage) {
    let scope: DedicatedWorkerGlobalScope = js_sys::global().unchecked_into();
    if let Ok(value) = serde_wasm_bindgen::to_value(message) {
        let _ = scope.post_message(&value);
    }
}
