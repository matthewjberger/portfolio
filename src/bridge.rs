use leptos::prelude::*;
use protocol::{CANVAS_KEY, ClientMessage, MESSAGE_KEY, WorkerMessage};
use wasm_bindgen::prelude::*;
use wasm_bindgen::{JsCast, JsValue};
use web_sys::{MessageEvent, OffscreenCanvas, Worker, WorkerOptions, WorkerType};

use crate::state::PortfolioState;

/// The page side of the worker conversation. Data only; behavior is the free
/// functions below.
#[derive(Clone)]
pub struct Bridge {
    worker: Worker,
}

/// Spawns the worker, wires its `onmessage` to the state signals, sends `Init`
/// with the transferred canvas, and returns the bridge.
pub fn connect(
    offscreen: OffscreenCanvas,
    width: f32,
    height: f32,
    state: PortfolioState,
) -> Bridge {
    let options = WorkerOptions::new();
    options.set_type(WorkerType::Module);
    let worker =
        Worker::new_with_options("runtime/worker.js", &options).expect("failed to spawn worker");

    let onmessage = Closure::<dyn FnMut(MessageEvent)>::new(move |event: MessageEvent| {
        let Ok(message) = serde_wasm_bindgen::from_value::<WorkerMessage>(event.data()) else {
            return;
        };
        match message {
            WorkerMessage::Ready => state.ready.set(true),
            WorkerMessage::Stats { fps } => state.fps.set(fps),
            WorkerMessage::Game { status } => {
                let was_cleared = state.game_phase.get_untracked() == protocol::GamePhase::Cleared;
                state.game_phase.set(status.phase);
                state.game_level.set(status.level);
                state.game_score.set(status.score);
                state.game_shots_left.set(status.shots_left);
                state.game_shots_total.set(status.shots_total);
                state.game_targets_left.set(status.targets_left);
                state.game_targets_total.set(status.targets_total);
                state.game_combo.set(status.combo);
                if status.phase == protocol::GamePhase::Cleared && !was_cleared {
                    crate::state::record_game_best(status.level, status.score);
                    crate::state::unlock_game_level(status.level + 1);
                }
                if status.phase != protocol::GamePhase::Playing {
                    state.game_hits.set(Vec::new());
                }
            }
            WorkerMessage::GameHit { points, combo } => {
                state.game_hits.update(|hits| {
                    let id = hits.last().map(|(id, _)| id + 1).unwrap_or(0);
                    let text = if combo > 1 {
                        format!("+{points} x{combo}")
                    } else {
                        format!("+{points}")
                    };
                    hits.push((id, text));
                    if hits.len() > 3 {
                        let excess = hits.len() - 3;
                        hits.drain(0..excess);
                    }
                });
            }
        }
    });
    worker.set_onmessage(Some(onmessage.as_ref().unchecked_ref()));
    onmessage.forget();

    let bridge = Bridge { worker };
    send_init(&bridge, offscreen, width, height);
    bridge
}

/// Forwards a message to the worker inside the `{ message }` envelope.
pub fn send(bridge: &Bridge, message: &ClientMessage) {
    let envelope = js_sys::Object::new();
    let value = serde_wasm_bindgen::to_value(message).unwrap_or(JsValue::NULL);
    let _ = js_sys::Reflect::set(&envelope, &JsValue::from_str(MESSAGE_KEY), &value);
    let _ = bridge.worker.post_message(&envelope);
}

fn send_init(bridge: &Bridge, canvas: OffscreenCanvas, width: f32, height: f32) {
    let envelope = js_sys::Object::new();
    let value = serde_wasm_bindgen::to_value(&ClientMessage::Init { width, height })
        .unwrap_or(JsValue::NULL);
    let _ = js_sys::Reflect::set(&envelope, &JsValue::from_str(MESSAGE_KEY), &value);
    let _ = js_sys::Reflect::set(&envelope, &JsValue::from_str(CANVAS_KEY), &canvas);
    let transfer = js_sys::Array::of1(&canvas);
    let _ = bridge
        .worker
        .post_message_with_transfer(&envelope, &transfer);
}
