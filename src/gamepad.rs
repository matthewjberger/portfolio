use std::cell::RefCell;
use std::rc::Rc;

use leptos::prelude::*;
use protocol::{ClientMessage, GameCommand, GamePhase};
use wasm_bindgen::JsCast;
use wasm_bindgen::prelude::*;
use web_sys::Gamepad;

use crate::bridge::{Bridge, send};
use crate::state::PortfolioState;

const DEADZONE: f64 = 0.2;
const BUTTON_A: u32 = 0;
const BUTTON_B: u32 = 1;
const BUTTON_LEFT_BUMPER: u32 = 4;
const BUTTON_RIGHT_BUMPER: u32 = 5;
const BUTTON_RIGHT_TRIGGER: u32 = 7;
const BUTTON_START: u32 = 9;
const BUTTON_DPAD_UP: u32 = 12;
const BUTTON_DPAD_DOWN: u32 = 13;
const BUTTON_DPAD_LEFT: u32 = 14;
const BUTTON_DPAD_RIGHT: u32 = 15;

const BROWSE_SCOPE: &str = "nav a, nav button, main a, main button, main input";
const OVERLAY_SCOPE: &str = "[data-pad-overlay] button";

const SCROLL_SPEED: f64 = 30.0;
const ORBIT_YAW_RATE: f32 = 0.045;
const ORBIT_PITCH_RATE: f32 = 0.03;
const ZOOM_RATE: f32 = 0.02;

type Buttons = Rc<RefCell<Vec<bool>>>;
type Frame = Rc<RefCell<Option<Closure<dyn FnMut()>>>>;

/// Polls the first connected gamepad every frame and routes it by mode: the
/// portfolio scrolls and moves DOM focus, overlays cycle their buttons, the
/// intro skips, and gameplay orbits, zooms, and fires at the screen center.
pub fn start(bridge: StoredValue<Option<Bridge>, LocalStorage>, state: PortfolioState) {
    let previous: Buttons = Rc::new(RefCell::new(Vec::new()));
    let frame: Frame = Rc::new(RefCell::new(None));
    let frame_handle = frame.clone();

    *frame.borrow_mut() = Some(Closure::<dyn FnMut()>::new(move || {
        poll(bridge, state, &previous);
        if let Some(window) = web_sys::window()
            && let Some(callback) = frame_handle.borrow().as_ref()
        {
            let _ = window.request_animation_frame(callback.as_ref().unchecked_ref());
        }
    }));

    if let Some(window) = web_sys::window()
        && let Some(callback) = frame.borrow().as_ref()
    {
        let _ = window.request_animation_frame(callback.as_ref().unchecked_ref());
    }
    std::mem::forget(frame);
}

fn poll(
    bridge: StoredValue<Option<Bridge>, LocalStorage>,
    state: PortfolioState,
    previous: &Buttons,
) {
    let Some(gamepad) = first_gamepad() else {
        previous.borrow_mut().clear();
        return;
    };

    let buttons: Vec<bool> = gamepad
        .buttons()
        .iter()
        .map(|button| {
            button
                .dyn_into::<web_sys::GamepadButton>()
                .map(|button| button.pressed())
                .unwrap_or(false)
        })
        .collect();
    let pressed = |index: u32| -> bool {
        let index = index as usize;
        buttons.get(index).copied().unwrap_or(false)
            && !previous.borrow().get(index).copied().unwrap_or(false)
    };
    let axes: Vec<f64> = gamepad
        .axes()
        .iter()
        .filter_map(|axis| axis.as_f64())
        .collect();
    let axis = |index: usize| -> f64 {
        let value = axes.get(index).copied().unwrap_or(0.0);
        if value.abs() > DEADZONE { value } else { 0.0 }
    };

    let any_input =
        buttons.iter().any(|&pressed| pressed) || axes.iter().any(|value| value.abs() > DEADZONE);
    if any_input {
        mark_pad_navigation();
    }

    let phase = state.game_phase.get_untracked();
    let menu_open = state.game_menu_open.get_untracked();

    if phase == GamePhase::Intro {
        if pressed(BUTTON_A) || pressed(BUTTON_B) || pressed(BUTTON_START) {
            send_game(bridge, GameCommand::SkipIntro);
        }
    } else if phase == GamePhase::Playing {
        let yaw = axis(0) as f32 * ORBIT_YAW_RATE;
        let pitch = -axis(1) as f32 * ORBIT_PITCH_RATE;
        let zoom = axis(3) as f32 * ZOOM_RATE;
        if (yaw != 0.0 || pitch != 0.0 || zoom != 0.0)
            && let Some(bridge) = bridge.get_value()
        {
            send(&bridge, &ClientMessage::Orbit { yaw, pitch, zoom });
        }
        if pressed(BUTTON_A) || pressed(BUTTON_RIGHT_TRIGGER) {
            fire_center(bridge);
        }
        if pressed(BUTTON_B) || pressed(BUTTON_START) {
            send_game(bridge, GameCommand::Exit);
            state.game_menu_open.set(true);
        }
    } else if menu_open || matches!(phase, GamePhase::Cleared | GamePhase::Failed) {
        if pressed(BUTTON_DPAD_DOWN) || pressed(BUTTON_DPAD_RIGHT) {
            move_focus(OVERLAY_SCOPE, 1);
        }
        if pressed(BUTTON_DPAD_UP) || pressed(BUTTON_DPAD_LEFT) {
            move_focus(OVERLAY_SCOPE, -1);
        }
        if pressed(BUTTON_A) {
            click_focused(OVERLAY_SCOPE);
        }
        if pressed(BUTTON_B) {
            click_cancel();
        }
    } else {
        let scroll = axis(1);
        if scroll != 0.0
            && let Some(window) = web_sys::window()
        {
            window.scroll_by_with_x_and_y(0.0, scroll * SCROLL_SPEED);
        }
        if pressed(BUTTON_DPAD_DOWN) || pressed(BUTTON_DPAD_RIGHT) {
            move_focus(BROWSE_SCOPE, 1);
        }
        if pressed(BUTTON_DPAD_UP) || pressed(BUTTON_DPAD_LEFT) {
            move_focus(BROWSE_SCOPE, -1);
        }
        if pressed(BUTTON_A) {
            click_focused(BROWSE_SCOPE);
        }
        if pressed(BUTTON_RIGHT_BUMPER) {
            jump_section(state, 1);
        }
        if pressed(BUTTON_LEFT_BUMPER) {
            jump_section(state, -1);
        }
        if pressed(BUTTON_START) && state.webgpu {
            state.game_menu_open.set(true);
        }
    }

    *previous.borrow_mut() = buttons;
}

fn first_gamepad() -> Option<Gamepad> {
    let window = web_sys::window()?;
    let gamepads = window.navigator().get_gamepads().ok()?;
    gamepads
        .iter()
        .find_map(|entry| entry.dyn_into::<Gamepad>().ok())
}

/// Flags the page as gamepad-navigated so the focus ring styles activate.
fn mark_pad_navigation() {
    if let Some(document) = web_sys::window().and_then(|window| window.document())
        && let Some(body) = document.body()
    {
        let _ = body.class_list().add_1("pad-nav");
    }
}

fn focusables(scope: &str) -> Vec<web_sys::HtmlElement> {
    let Some(document) = web_sys::window().and_then(|window| window.document()) else {
        return Vec::new();
    };
    let Ok(nodes) = document.query_selector_all(scope) else {
        return Vec::new();
    };
    let mut elements = Vec::new();
    for index in 0..nodes.length() {
        if let Some(node) = nodes.item(index)
            && let Ok(element) = node.dyn_into::<web_sys::HtmlElement>()
            && element.offset_parent().is_some()
        {
            elements.push(element);
        }
    }
    elements
}

fn active_index(elements: &[web_sys::HtmlElement]) -> Option<usize> {
    let active = web_sys::window()?.document()?.active_element()?;
    elements
        .iter()
        .position(|element| element.is_same_node(Some(active.as_ref())))
}

fn move_focus(scope: &str, step: i32) {
    let elements = focusables(scope);
    if elements.is_empty() {
        return;
    }
    let count = elements.len() as i32;
    let next = match active_index(&elements) {
        Some(index) => (index as i32 + step).rem_euclid(count),
        None => {
            if step > 0 {
                0
            } else {
                count - 1
            }
        }
    } as usize;
    let element = &elements[next];
    let _ = element.focus();
    element.scroll_into_view_with_scroll_into_view_options(&scroll_center());
}

fn scroll_center() -> web_sys::ScrollIntoViewOptions {
    let options = web_sys::ScrollIntoViewOptions::new();
    options.set_block(web_sys::ScrollLogicalPosition::Center);
    options
}

fn click_focused(scope: &str) {
    let elements = focusables(scope);
    if let Some(index) = active_index(&elements) {
        elements[index].click();
    } else if let Some(first) = elements.first() {
        let _ = first.focus();
    }
}

fn click_cancel() {
    if let Some(document) = web_sys::window().and_then(|window| window.document())
        && let Ok(Some(element)) = document.query_selector("[data-pad-cancel]")
        && let Ok(element) = element.dyn_into::<web_sys::HtmlElement>()
    {
        element.click();
    }
}

fn jump_section(state: PortfolioState, step: i32) {
    let Some(document) = web_sys::window().and_then(|window| window.document()) else {
        return;
    };
    let current = state.section.get_untracked() as i32;
    let target = (current + step).clamp(0, crate::state::SECTIONS.len() as i32 - 1) as usize;
    if let Some(element) = document.get_element_by_id(crate::state::SECTIONS[target].0) {
        element.scroll_into_view();
    }
}

fn fire_center(bridge: StoredValue<Option<Bridge>, LocalStorage>) {
    let Some(window) = web_sys::window() else {
        return;
    };
    let dpr = window.device_pixel_ratio().min(2.0);
    let width = window
        .inner_width()
        .ok()
        .and_then(|value| value.as_f64())
        .unwrap_or(0.0);
    let height = window
        .inner_height()
        .ok()
        .and_then(|value| value.as_f64())
        .unwrap_or(0.0);
    send_game(
        bridge,
        GameCommand::Fire {
            x: (width * 0.5 * dpr) as f32,
            y: (height * 0.5 * dpr) as f32,
        },
    );
}

fn send_game(bridge: StoredValue<Option<Bridge>, LocalStorage>, command: GameCommand) {
    if let Some(bridge) = bridge.get_value() {
        send(&bridge, &ClientMessage::Game { command });
    }
}
