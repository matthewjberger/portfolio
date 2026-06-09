use std::collections::HashMap;

use leptos::html;
use leptos::prelude::*;
use protocol::{ClientMessage, GameCommand, GamePhase, TouchPhase};
use wasm_bindgen::JsCast;
use wasm_bindgen::prelude::*;
use web_sys::{HtmlCanvasElement, MouseEvent, PointerEvent, ResizeObserver, WheelEvent};

use crate::bridge::{self, Bridge, send};
use crate::state::PortfolioState;

#[derive(Clone, Copy, Default)]
struct DragState {
    button: Option<u8>,
    last_x: f32,
    last_y: f32,
    moved: f32,
}

/// Per-contact tracking for forwarded touches, used to tell a tap (fire) from
/// a drag (camera gesture) while the game is active.
#[derive(Clone, Copy)]
struct TouchTrack {
    last_x: f32,
    last_y: f32,
    moved: f32,
}

/// The render surface behind the page. Always reports scroll progress and the
/// normalized pointer glance to the worker; while the siege game is active it
/// also forwards raw pointer, wheel, and touch input so the engine camera can
/// orbit and taps fire cannonballs.
#[component]
pub fn Viewport(
    bridge: StoredValue<Option<Bridge>, LocalStorage>,
    state: PortfolioState,
) -> impl IntoView {
    let canvas_ref = NodeRef::<html::Canvas>::new();
    let drag = StoredValue::new(DragState::default());
    let touches = StoredValue::new(HashMap::<i32, TouchTrack>::new());

    Effect::new(move |_| {
        let Some(canvas) = canvas_ref.get() else {
            return;
        };
        if bridge.with_value(Option::is_some) {
            return;
        }
        let dpr = render_dpr() as f32;
        let rect = canvas.get_bounding_client_rect();
        let width = rect.width() as f32 * dpr;
        let height = rect.height() as f32 * dpr;
        canvas.set_width(width as u32);
        canvas.set_height(height as u32);
        let offscreen = canvas
            .transfer_control_to_offscreen()
            .expect("failed to transfer canvas to offscreen");
        let connected = bridge::connect(offscreen, width, height, state);
        attach_wheel(&canvas, bridge);
        observe_resize(canvas, connected.clone());
        send_reduced_motion(&connected, state);
        attach_scroll(bridge);
        attach_glance(bridge);
        bridge.set_value(Some(connected));
    });

    let on_pointerdown = move |event: PointerEvent| {
        if event.pointer_type() == "touch" {
            let id = event.pointer_id();
            touches.update_value(|map| {
                map.insert(
                    id,
                    TouchTrack {
                        last_x: event.client_x() as f32,
                        last_y: event.client_y() as f32,
                        moved: 0.0,
                    },
                );
            });
            if let Some(canvas) = canvas_ref.get() {
                let _ = canvas.set_pointer_capture(id);
                if let Some(bridge) = bridge.get_value() {
                    let (x, y) = physical(event.client_x(), event.client_y());
                    send(
                        &bridge,
                        &ClientMessage::Touch {
                            id: id as u64,
                            phase: TouchPhase::Started,
                            x,
                            y,
                        },
                    );
                }
            }
            return;
        }
        let button = event.button().max(0) as u8;
        drag.update_value(|state| {
            state.button = Some(button);
            state.last_x = event.client_x() as f32;
            state.last_y = event.client_y() as f32;
            state.moved = 0.0;
        });
        if let Some(canvas) = canvas_ref.get() {
            let _ = canvas.set_pointer_capture(event.pointer_id());
            if let Some(bridge) = bridge.get_value() {
                let (x, y) = physical(event.client_x(), event.client_y());
                send(&bridge, &ClientMessage::PointerMove { x, y });
                send(
                    &bridge,
                    &ClientMessage::PointerButton {
                        button,
                        pressed: true,
                    },
                );
            }
        }
    };

    let on_pointermove = move |event: PointerEvent| {
        if event.pointer_type() == "touch" {
            let id = event.pointer_id();
            touches.update_value(|map| {
                if let Some(track) = map.get_mut(&id) {
                    let x = event.client_x() as f32;
                    let y = event.client_y() as f32;
                    track.moved += (x - track.last_x).abs() + (y - track.last_y).abs();
                    track.last_x = x;
                    track.last_y = y;
                }
            });
            if let Some(bridge) = bridge.get_value() {
                let (x, y) = physical(event.client_x(), event.client_y());
                send(
                    &bridge,
                    &ClientMessage::Touch {
                        id: id as u64,
                        phase: TouchPhase::Moved,
                        x,
                        y,
                    },
                );
            }
            return;
        }
        drag.update_value(|state| {
            let x = event.client_x() as f32;
            let y = event.client_y() as f32;
            state.moved += (x - state.last_x).abs() + (y - state.last_y).abs();
            state.last_x = x;
            state.last_y = y;
        });
        if let Some(bridge) = bridge.get_value() {
            let (x, y) = physical(event.client_x(), event.client_y());
            send(&bridge, &ClientMessage::PointerMove { x, y });
        }
    };

    let on_pointerup = move |event: PointerEvent| {
        if event.pointer_type() == "touch" {
            let id = event.pointer_id();
            let (moved, count) = touches.with_value(|map| {
                (
                    map.get(&id).map(|track| track.moved).unwrap_or(0.0),
                    map.len(),
                )
            });
            touches.update_value(|map| {
                map.remove(&id);
            });
            if let Some(canvas) = canvas_ref.get() {
                let _ = canvas.release_pointer_capture(id);
                if let Some(bridge) = bridge.get_value() {
                    let (x, y) = physical(event.client_x(), event.client_y());
                    send(
                        &bridge,
                        &ClientMessage::Touch {
                            id: id as u64,
                            phase: TouchPhase::Ended,
                            x,
                            y,
                        },
                    );
                    if count == 1
                        && moved < 5.0
                        && state.game_phase.get_untracked() == GamePhase::Playing
                    {
                        send(
                            &bridge,
                            &ClientMessage::Game {
                                command: GameCommand::Fire { x, y },
                            },
                        );
                    }
                }
            }
            return;
        }
        let (button, moved) = drag.with_value(|state| (state.button, state.moved));
        drag.update_value(|state| state.button = None);
        if let Some(canvas) = canvas_ref.get() {
            let _ = canvas.release_pointer_capture(event.pointer_id());
            if let Some(bridge) = bridge.get_value() {
                let (x, y) = physical(event.client_x(), event.client_y());
                send(
                    &bridge,
                    &ClientMessage::PointerButton {
                        button: event.button().max(0) as u8,
                        pressed: false,
                    },
                );
                if button == Some(0)
                    && moved < 5.0
                    && state.game_phase.get_untracked() == GamePhase::Playing
                {
                    send(
                        &bridge,
                        &ClientMessage::Game {
                            command: GameCommand::Fire { x, y },
                        },
                    );
                }
            }
        }
    };

    let on_pointercancel = move |event: PointerEvent| {
        if event.pointer_type() != "touch" {
            return;
        }
        let id = event.pointer_id();
        touches.update_value(|map| {
            map.remove(&id);
        });
        if let Some(canvas) = canvas_ref.get() {
            let _ = canvas.release_pointer_capture(id);
            if let Some(bridge) = bridge.get_value() {
                let (x, y) = physical(event.client_x(), event.client_y());
                send(
                    &bridge,
                    &ClientMessage::Touch {
                        id: id as u64,
                        phase: TouchPhase::Cancelled,
                        x,
                        y,
                    },
                );
            }
        }
    };

    let on_contextmenu = move |event: MouseEvent| event.prevent_default();

    let canvas_class = move || {
        if state.game_phase.get() == GamePhase::Idle {
            "fixed inset-0 z-0 w-full h-full pointer-events-none"
        } else {
            "fixed inset-0 z-0 w-full h-full touch-none cursor-crosshair"
        }
    };

    view! {
        <canvas
            id="canvas"
            node_ref=canvas_ref
            class=canvas_class
            on:pointerdown=on_pointerdown
            on:pointermove=on_pointermove
            on:pointerup=on_pointerup
            on:pointercancel=on_pointercancel
            on:contextmenu=on_contextmenu
        ></canvas>
    }
}

const MAX_RENDER_DPR: f64 = 2.0;

fn render_dpr() -> f64 {
    web_sys::window()
        .unwrap()
        .device_pixel_ratio()
        .min(MAX_RENDER_DPR)
}

/// Maps a client-space pointer position to physical canvas pixels. The canvas
/// fills the window, so no element offset is involved.
fn physical(client_x: i32, client_y: i32) -> (f32, f32) {
    let dpr = render_dpr();
    (
        (client_x as f64 * dpr) as f32,
        (client_y as f64 * dpr) as f32,
    )
}

fn attach_wheel(canvas: &HtmlCanvasElement, bridge: StoredValue<Option<Bridge>, LocalStorage>) {
    let on_wheel = Closure::<dyn FnMut(WheelEvent)>::new(move |event: WheelEvent| {
        event.prevent_default();
        if let Some(bridge) = bridge.get_value() {
            send(
                &bridge,
                &ClientMessage::Wheel {
                    delta: event.delta_y() as f32,
                },
            );
        }
    });
    let options = web_sys::AddEventListenerOptions::new();
    options.set_passive(false);
    canvas
        .add_event_listener_with_callback_and_add_event_listener_options(
            "wheel",
            on_wheel.as_ref().unchecked_ref(),
            &options,
        )
        .expect("failed to add wheel listener");
    on_wheel.forget();
}

/// Streams the page's overall scroll progress (0..1) to the worker so the
/// camera tour can chase it, and keeps the scroll-spy section signal current.
fn attach_scroll(bridge: StoredValue<Option<Bridge>, LocalStorage>) {
    let send_progress = move || {
        let Some(window) = web_sys::window() else {
            return;
        };
        let Some(document) = window.document() else {
            return;
        };
        let Some(root) = document.document_element() else {
            return;
        };
        let scroll_top = root.scroll_top() as f32;
        let span = (root.scroll_height() - root.client_height()).max(1) as f32;
        if let Some(bridge) = bridge.get_value() {
            send(
                &bridge,
                &ClientMessage::Scroll {
                    progress: (scroll_top / span).clamp(0.0, 1.0),
                },
            );
        }
    };
    send_progress();
    let _ = window_event_listener(leptos::ev::scroll, move |_| send_progress());
}

/// Streams the normalized pointer position (-1..1, y up) for the parallax
/// glance and the floater nudge ray.
fn attach_glance(bridge: StoredValue<Option<Bridge>, LocalStorage>) {
    let _ = window_event_listener(leptos::ev::mousemove, move |event| {
        let Some(window) = web_sys::window() else {
            return;
        };
        let width = window
            .inner_width()
            .ok()
            .and_then(|value| value.as_f64())
            .unwrap_or(1.0)
            .max(1.0);
        let height = window
            .inner_height()
            .ok()
            .and_then(|value| value.as_f64())
            .unwrap_or(1.0)
            .max(1.0);
        let x = (event.client_x() as f64 / width) * 2.0 - 1.0;
        let y = 1.0 - (event.client_y() as f64 / height) * 2.0;
        if let Some(bridge) = bridge.get_value() {
            send(
                &bridge,
                &ClientMessage::Glance {
                    x: x as f32,
                    y: y as f32,
                },
            );
        }
    });
}

fn send_reduced_motion(bridge: &Bridge, state: PortfolioState) {
    let reduced = web_sys::window()
        .and_then(|window| {
            window
                .match_media("(prefers-reduced-motion: reduce)")
                .ok()
                .flatten()
        })
        .map(|query| query.matches())
        .unwrap_or(false);
    state.reduced_motion.set(reduced);
    if reduced {
        send(bridge, &ClientMessage::SetReducedMotion { enabled: true });
    }
}

fn observe_resize(canvas: HtmlCanvasElement, bridge: Bridge) {
    let resize_canvas = canvas.clone();
    let on_resize = Closure::<dyn FnMut()>::new(move || {
        let dpr = render_dpr() as f32;
        let rect = resize_canvas.get_bounding_client_rect();
        send(
            &bridge,
            &ClientMessage::Resize {
                width: rect.width() as f32 * dpr,
                height: rect.height() as f32 * dpr,
            },
        );
    });
    let observer = ResizeObserver::new(on_resize.as_ref().unchecked_ref())
        .expect("failed to create resize observer");
    observer.observe(&canvas);
    on_resize.forget();
    std::mem::forget(observer);
}
