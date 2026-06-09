use leptos::prelude::*;
use protocol::GamePhase;

use crate::content::Content;
use crate::state::PortfolioState;

const PANEL: &str = "rounded-2xl border border-white/15 bg-[#0c0e14]/90 backdrop-blur-md shadow-2xl shadow-black/40";
const CARD: &str = "rounded-xl hover:border-violet-400/40 transition-colors";
const CHIP: &str =
    "px-2 py-0.5 rounded-md bg-white/5 border border-white/10 text-[11px] text-white/75";

/// A full-height page section that fades and rises in once the scroll spy has
/// reached it.
#[component]
fn Section(
    index: usize,
    id: &'static str,
    state: PortfolioState,
    children: Children,
) -> impl IntoView {
    let class = move || {
        if state.revealed.get() >= index {
            "reveal revealed w-full max-w-5xl mx-auto"
        } else {
            "reveal w-full max-w-5xl mx-auto"
        }
    };
    view! {
        <section id=id class="min-h-screen flex items-center px-4 sm:px-8 py-24">
            <div class=class>{children()}</div>
        </section>
    }
}

fn heading(kicker: &'static str, title: &'static str) -> impl IntoView {
    view! {
        <div class="mb-6">
            <span class="text-[12px] uppercase tracking-[0.3em] text-violet-300">{kicker}</span>
            <h2 class="mt-1 text-[28px] sm:text-[34px] font-bold text-white">{title}</h2>
        </div>
    }
}

#[component]
pub fn Hero(state: PortfolioState, content: &'static Content) -> impl IntoView {
    let about = &content.about;
    let paragraphs = about
        .paragraphs
        .iter()
        .map(|paragraph| {
            view! { <p class="text-[15px] leading-relaxed text-white/85">{paragraph.as_str()}</p> }
        })
        .collect_view();

    view! {
        <section id="hero" class="min-h-screen flex items-center px-4 sm:px-8 pt-24 pb-16">
            <div class="w-full max-w-5xl mx-auto">
                <div class=format!("{PANEL} p-8 sm:p-12 max-w-[640px]")>
                    <div class="flex items-center gap-5">
                        <img
                            src=about.avatar.as_str()
                            alt="Matthew Berger"
                            class="w-20 h-20 sm:w-24 sm:h-24 rounded-2xl border border-white/15 object-cover"
                        />
                        <div>
                            <h1 class="text-[32px] sm:text-[42px] font-bold text-white leading-tight">
                                {about.name.as_str()}
                            </h1>
                            <p class="mt-1 text-[14px] text-violet-300/90">{about.tagline.as_str()}</p>
                        </div>
                    </div>
                    <div class="mt-6 space-y-4">{paragraphs}</div>
                    <div class="mt-7 flex flex-wrap items-center gap-2">
                        <a
                            class="px-4 py-2 rounded-lg bg-violet-500/85 hover:bg-violet-400 text-white text-[13px] font-semibold transition-colors"
                            href=about.resume.as_str()
                            target="_blank"
                            rel="noopener noreferrer"
                        >
                            "Resume"
                        </a>
                        <a
                            class="px-4 py-2 rounded-lg bg-white/10 hover:bg-white/20 text-white/90 text-[13px] font-semibold transition-colors"
                            href=about.github.as_str()
                            target="_blank"
                            rel="noopener noreferrer"
                        >
                            "GitHub"
                        </a>
                        <a
                            class="px-4 py-2 rounded-lg bg-white/10 hover:bg-white/20 text-white/90 text-[13px] font-semibold transition-colors"
                            href=about.linkedin.as_str()
                            target="_blank"
                            rel="noopener noreferrer"
                        >
                            "LinkedIn"
                        </a>
                        <a
                            class="px-4 py-2 rounded-lg bg-white/5 hover:bg-white/15 text-white/70 text-[13px] font-semibold transition-colors"
                            href=about.articles.as_str()
                            target="_blank"
                            rel="noopener noreferrer"
                        >
                            "Articles"
                        </a>
                        <Show when=move || state.webgpu && state.game_phase.get() == GamePhase::Idle fallback=|| ()>
                            <button
                                class="px-4 py-2 rounded-lg border border-violet-400/40 text-violet-300 hover:bg-violet-500/15 text-[13px] font-semibold transition-colors"
                                title="A physics game built with the engine rendering this page"
                                on:click=move |_| state.game_menu_open.set(true)
                            >
                                "▶ Nightshade Siege"
                            </button>
                        </Show>
                    </div>
                    <Show when=move || state.webgpu fallback=|| ()>
                        <p class="mt-6 text-[11px] text-white/60">
                            "The world behind this page is rendered live by the Nightshade engine in a web worker. Scroll to fly through it."
                        </p>
                    </Show>
                </div>
                <div class="mt-10 flex justify-center">
                    <a href="#highlights" class="text-white/40 hover:text-white/80 text-[22px] animate-bounce">
                        "↓"
                    </a>
                </div>
            </div>
        </section>
    }
}

#[component]
pub fn Highlights(state: PortfolioState, content: &'static Content) -> impl IntoView {
    let cards = content
        .highlights
        .iter()
        .map(|highlight| {
            view! {
                <div class=format!("{CARD} {PANEL} overflow-hidden flex flex-col")>
                    <img
                        src=format!("public/{}", highlight.image.trim_start_matches('/'))
                        alt=highlight.title.clone()
                        class="w-full h-40 object-cover border-b border-white/10"
                    />
                    <div class="p-5 flex flex-col gap-3 flex-1">
                        <h3 class="text-[17px] font-semibold text-white">{highlight.title.as_str()}</h3>
                        <p class="text-[13px] leading-relaxed text-white/80 flex-1">
                            {highlight.description.as_str()}
                        </p>
                        <div class="flex items-center gap-2">
                            <a
                                class="px-3 py-1.5 rounded-lg bg-white/10 hover:bg-white/20 text-white/90 text-[12px] font-semibold transition-colors"
                                href=highlight.link.as_str()
                                target="_blank"
                                rel="noopener noreferrer"
                            >
                                "Source"
                            </a>
                            <a
                                class="px-3 py-1.5 rounded-lg text-violet-300 hover:bg-violet-500/15 text-[12px] font-semibold transition-colors"
                                href=highlight.demo_link.as_str()
                                target="_blank"
                                rel="noopener noreferrer"
                            >
                                {highlight.demo_label.as_str()}
                            </a>
                        </div>
                    </div>
                </div>
            }
        })
        .collect_view();

    view! {
        <Section index=1 id="highlights" state=state>
            {heading("Featured work", "Highlights")}
            <div class="grid grid-cols-1 md:grid-cols-3 gap-4">{cards}</div>
        </Section>
    }
}

#[component]
pub fn ExperienceTimeline(state: PortfolioState, content: &'static Content) -> impl IntoView {
    let jobs = content
        .jobs
        .iter()
        .map(|job| {
            let expanded = RwSignal::new(false);
            let total = job.achievements.len();
            let achievements = move || {
                let count = if expanded.get() { total } else { 3.min(total) };
                job.achievements[..count]
                    .iter()
                    .map(|achievement| {
                        view! {
                            <li class="text-[13px] leading-relaxed text-white/85 pl-4 relative before:content-['▸'] before:absolute before:left-0 before:text-violet-400/70">
                                {achievement.as_str()}
                            </li>
                        }
                    })
                    .collect_view()
            };
            view! {
                <div class="relative pl-8">
                    <span class="absolute left-0 top-1.5 w-3 h-3 rounded-full bg-violet-400 shadow shadow-violet-500/50"></span>
                    <span class="absolute left-[5px] top-6 bottom-[-24px] w-px bg-white/10"></span>
                    <div class=format!("{PANEL} p-5")>
                        <div class="flex flex-wrap items-baseline gap-x-3 gap-y-1">
                            <h3 class="text-[16px] font-semibold text-white">{job.company.as_str()}</h3>
                            <span class="text-[12px] text-white/65 tabular-nums">{job.period.as_str()}</span>
                        </div>
                        <p class="mt-0.5 text-[13px] text-violet-300/90">{job.title.as_str()}</p>
                        <ul class="mt-3 space-y-2">{achievements}</ul>
                        <Show when=move || { total > 3 } fallback=|| ()>
                            <button
                                class="mt-3 text-[12px] text-white/65 hover:text-white transition-colors"
                                on:click=move |_| expanded.update(|open| *open = !*open)
                            >
                                {move || {
                                    if expanded.get() {
                                        "Show less".to_string()
                                    } else {
                                        format!("Show all {total} achievements")
                                    }
                                }}
                            </button>
                        </Show>
                    </div>
                </div>
            }
        })
        .collect_view();

    view! {
        <Section index=2 id="experience" state=state>
            {heading("Where I have shipped", "Experience")}
            <div class="space-y-6">{jobs}</div>
        </Section>
    }
}

#[component]
pub fn Projects(state: PortfolioState, content: &'static Content) -> impl IntoView {
    let filter = RwSignal::new("all".to_string());
    let query = RwSignal::new(String::new());

    let chips = [
        ("all", "All"),
        ("rust", "Rust"),
        ("go", "Go"),
        ("other", "Other"),
    ]
    .into_iter()
    .map(|(value, label)| {
        let class = move || {
            if filter.get() == value {
                "px-3 py-1.5 rounded-lg bg-violet-500/25 border border-violet-400/40 text-violet-200 text-[12px] font-semibold transition-colors"
            } else {
                "px-3 py-1.5 rounded-lg bg-white/5 border border-white/10 text-white/60 hover:text-white text-[12px] font-semibold transition-colors"
            }
        };
        view! {
            <button class=class on:click=move |_| filter.set(value.to_string())>
                {label}
            </button>
        }
    })
    .collect_view();

    let cards = move || {
        let active = filter.get();
        let needle = query.get().to_lowercase();
        content
            .projects
            .iter()
            .filter(|project| active == "all" || project.language == active)
            .filter(|project| {
                needle.is_empty()
                    || project.title.to_lowercase().contains(&needle)
                    || project.description.to_lowercase().contains(&needle)
                    || project
                        .technologies
                        .iter()
                        .any(|technology| technology.to_lowercase().contains(&needle))
            })
            .map(|project| {
                let technologies = project
                    .technologies
                    .iter()
                    .map(|technology| view! { <span class=CHIP>{technology.as_str()}</span> })
                    .collect_view();
                view! {
                    <a
                        class=format!("{CARD} {PANEL} p-5 flex flex-col gap-3")
                        href=project.link.as_str()
                        target="_blank"
                        rel="noopener noreferrer"
                    >
                        <h3 class="text-[15px] font-semibold text-white">{project.title.as_str()}</h3>
                        <p class="text-[12.5px] leading-relaxed text-white/80 flex-1">
                            {project.description.as_str()}
                        </p>
                        <div class="flex flex-wrap gap-1.5">{technologies}</div>
                    </a>
                }
            })
            .collect_view()
    };

    view! {
        <Section index=3 id="projects" state=state>
            {heading("Open source", "Projects")}
            <div class="mb-5 flex flex-wrap items-center gap-2">
                {chips}
                <input
                    type="text"
                    placeholder="Search projects…"
                    class="ml-auto w-full sm:w-56 px-3 py-1.5 rounded-lg bg-white/5 border border-white/10 text-[12px] text-white/85 placeholder:text-white/30 focus:outline-none focus:border-violet-400/50"
                    on:input=move |event| query.set(event_target_value(&event))
                    prop:value=query
                />
            </div>
            <div class="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 gap-4">{cards}</div>
        </Section>
    }
}

#[component]
pub fn CratesSection(state: PortfolioState, content: &'static Content) -> impl IntoView {
    let cards = content
        .crates
        .iter()
        .map(|item| {
            let technologies = item
                .technologies
                .iter()
                .map(|technology| view! { <span class=CHIP>{technology.as_str()}</span> })
                .collect_view();
            view! {
                <a
                    class=format!("{CARD} {PANEL} p-5 flex flex-col gap-3")
                    href=item.link.as_str()
                    target="_blank"
                    rel="noopener noreferrer"
                >
                    <h3 class="text-[15px] font-semibold text-white">{item.title.as_str()}</h3>
                    <p class="text-[12.5px] leading-relaxed text-white/80 flex-1">
                        {item.description.as_str()}
                    </p>
                    <div class="flex flex-wrap gap-1.5">{technologies}</div>
                </a>
            }
        })
        .collect_view();

    view! {
        <Section index=4 id="crates" state=state>
            {heading("280k+ downloads on crates.io", "Published Crates")}
            <div class="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 gap-4">{cards}</div>
        </Section>
    }
}

#[component]
pub fn SkillsSection(state: PortfolioState, content: &'static Content) -> impl IntoView {
    let top = content
        .skills
        .top_items
        .iter()
        .map(|skill| {
            view! {
                <span class="px-4 py-2 rounded-xl bg-violet-500/20 border border-violet-400/40 text-violet-200 text-[14px] font-semibold">
                    {skill.as_str()}
                </span>
            }
        })
        .collect_view();
    let rest = content
        .skills
        .items
        .iter()
        .map(|skill| {
            view! {
                <span class="px-3 py-1.5 rounded-lg bg-white/5 border border-white/10 text-white/85 text-[12px]">
                    {skill.as_str()}
                </span>
            }
        })
        .collect_view();

    view! {
        <Section index=5 id="skills" state=state>
            {heading("Shipped to production", "Skills")}
            <div class=format!("{PANEL} p-7")>
                <div class="flex flex-wrap gap-2">{top}</div>
                <div class="mt-5 flex flex-wrap gap-2">{rest}</div>
            </div>
        </Section>
    }
}

#[component]
pub fn EducationSection(state: PortfolioState, content: &'static Content) -> impl IntoView {
    let degrees = content
        .degrees
        .iter()
        .map(|degree| {
            view! {
                <div class=format!("{PANEL} p-6")>
                    <h3 class="text-[16px] font-semibold text-white">{degree.degree.as_str()}</h3>
                    <p class="mt-1 text-[13px] text-white/75">
                        {degree.institution.as_str()}
                        " · "
                        {degree.period.as_str()}
                    </p>
                    <p class="mt-2 text-[13px] text-white/70">{degree.description.as_str()}</p>
                </div>
            }
        })
        .collect_view();

    view! {
        <Section index=6 id="education" state=state>
            {heading("Background", "Education")}
            <div class="space-y-4 max-w-[560px]">{degrees}</div>
            <footer class="mt-16 pb-8 flex flex-wrap items-center gap-x-4 gap-y-2 text-[12px] text-white/65">
                <span>"© Matthew Berger"</span>
                <a
                    class="hover:text-white/80 transition-colors"
                    href=content.about.source.as_str()
                    target="_blank"
                    rel="noopener noreferrer"
                >
                    "Site source"
                </a>
                <a
                    class="hover:text-white/80 transition-colors"
                    href=content.about.sponsor.as_str()
                    target="_blank"
                    rel="noopener noreferrer"
                >
                    "Sponsor"
                </a>
                <Show when=move || state.webgpu fallback=|| ()>
                    <span class="tabular-nums">
                        {move || format!("Rendered by Nightshade · {:.0} fps", state.fps.get())}
                    </span>
                </Show>
            </footer>
        </Section>
    }
}
