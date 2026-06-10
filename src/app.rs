use leptos::prelude::*;
use protocol::GamePhase;
use wasm_bindgen::JsValue;

use crate::bridge::Bridge;
use crate::components::game::Game;
use crate::components::nav::Nav;
use crate::components::sections::{
    CratesSection, EducationSection, ExperienceTimeline, Hero, Highlights, Projects, SkillsSection,
};
use crate::components::viewport::Viewport;
use crate::content;
use crate::state::{PortfolioState, SECTIONS};

/// Application root: the 3D viewport (or a CSS backdrop without WebGPU)
/// behind the scrolling portfolio sections, with the siege game chrome on top.
#[component]
pub fn App() -> impl IntoView {
    let webgpu = webgpu_supported();
    let state = PortfolioState::new(webgpu);
    let content = content::load();
    let bridge = StoredValue::new_local(None::<Bridge>);

    let _ = window_event_listener(leptos::ev::scroll, move |_| update_scroll_spy(state));
    crate::gamepad::start(bridge, state);

    let content_class = move || {
        if state.game_phase.get() == GamePhase::Idle {
            "relative z-10"
        } else {
            "relative z-10 invisible"
        }
    };

    view! {
        <Show
            when=move || state.webgpu
            fallback=|| view! { <div class="fixed inset-0 z-0 fallback-backdrop"></div> }
        >
            <Viewport bridge state />
        </Show>
        <div class=content_class>
            <Nav state content />
            <main>
                <Hero state content />
                <Highlights state content />
                <ExperienceTimeline state content />
                <Projects state content />
                <CratesSection state content />
                <SkillsSection state content />
                <EducationSection state content />
            </main>
        </div>
        <Show when=move || state.webgpu fallback=|| ()>
            <Game bridge state />
        </Show>
    }
}

/// Tracks which section currently fills the viewport (for the nav highlight)
/// and the furthest section reached (for the one-way reveal animations).
fn update_scroll_spy(state: PortfolioState) {
    let Some(window) = web_sys::window() else {
        return;
    };
    let Some(document) = window.document() else {
        return;
    };
    let midpoint = window
        .inner_height()
        .ok()
        .and_then(|value| value.as_f64())
        .unwrap_or(800.0)
        * 0.55;
    let mut current = 0;
    for (index, (id, _)) in SECTIONS.iter().enumerate() {
        if let Some(element) = document.get_element_by_id(id)
            && element.get_bounding_client_rect().top() <= midpoint
        {
            current = index;
        }
    }
    state.section.set(current);
    state
        .revealed
        .update(|revealed| *revealed = (*revealed).max(current));
}

fn webgpu_supported() -> bool {
    let Some(window) = web_sys::window() else {
        return false;
    };
    let Ok(navigator) = js_sys::Reflect::get(window.as_ref(), &JsValue::from_str("navigator"))
    else {
        return false;
    };
    js_sys::Reflect::get(&navigator, &JsValue::from_str("gpu"))
        .map(|gpu| !gpu.is_undefined() && !gpu.is_null())
        .unwrap_or(false)
}
