use leptos::prelude::*;
use protocol::GamePhase;

use crate::content::Content;
use crate::state::{PortfolioState, SECTIONS};

/// The fixed glass navigation bar: scroll-spy section links on the left,
/// outbound links and the game easter egg on the right.
#[component]
pub fn Nav(state: PortfolioState, content: &'static Content) -> impl IntoView {
    let links = SECTIONS
        .iter()
        .enumerate()
        .map(|(index, (id, label))| {
            let class = move || {
                if state.section.get() == index {
                    "hidden lg:inline px-2.5 py-1 rounded-md text-[12px] bg-white/10 text-white transition-colors"
                } else {
                    "hidden lg:inline px-2.5 py-1 rounded-md text-[12px] text-white/60 hover:text-white hover:bg-white/5 transition-colors"
                }
            };
            view! {
                <a href=format!("#{id}") class=class>
                    {*label}
                </a>
            }
        })
        .collect_view();

    view! {
        <nav class="fixed top-3 left-3 right-3 h-11 z-30 flex items-center gap-1 px-3 rounded-xl border border-white/10 bg-[#101218]/75 backdrop-blur-md shadow-lg shadow-black/40">
            <a
                href="#hero"
                class="shrink-0 text-[13px] font-semibold text-white/90 hover:text-white mr-2"
            >
                "Matthew Berger"
            </a>
            <div class="hidden lg:block shrink-0 w-px h-4 bg-white/10 mx-1"></div>
            {links}
            <div class="flex-1"></div>
            <Show when=move || state.webgpu && state.game_phase.get() == GamePhase::Idle fallback=|| ()>
                <button
                    class="shrink-0 px-2.5 py-1 rounded-md text-[12px] text-violet-300 hover:bg-violet-500/15 font-semibold transition-colors"
                    title="Play Nightshade Siege"
                    on:click=move |_| state.game_menu_open.set(true)
                >
                    "▶ Play"
                </button>
                <div class="shrink-0 w-px h-4 bg-white/10 mx-1"></div>
            </Show>
            <a
                class="shrink-0 px-2.5 py-1 rounded-md text-[12px] text-white/70 hover:bg-white/10 transition-colors"
                href=content.about.github.as_str()
                target="_blank"
                rel="noopener noreferrer"
            >
                "GitHub"
            </a>
            <a
                class="shrink-0 px-2.5 py-1 rounded-md text-[12px] text-white/70 hover:bg-white/10 transition-colors"
                href=content.about.linkedin.as_str()
                target="_blank"
                rel="noopener noreferrer"
            >
                "LinkedIn"
            </a>
            <a
                class="shrink-0 px-2.5 py-1 rounded-md text-[12px] text-white/70 hover:bg-white/10 transition-colors"
                href=content.about.resume.as_str()
                target="_blank"
                rel="noopener noreferrer"
            >
                "Resume"
            </a>
        </nav>
    }
}
