use serde::Deserialize;

/// The hero copy and outbound links.
#[derive(Clone, Deserialize)]
pub struct About {
    pub name: String,
    pub tagline: String,
    pub avatar: String,
    pub resume: String,
    pub github: String,
    pub linkedin: String,
    pub articles: String,
    pub sponsor: String,
    pub source: String,
    pub paragraphs: Vec<String>,
}

#[derive(Clone, Deserialize)]
pub struct Job {
    pub title: String,
    pub company: String,
    pub period: String,
    pub achievements: Vec<String>,
}

#[derive(Clone, Deserialize)]
struct Experience {
    jobs: Vec<Job>,
}

#[derive(Clone, Deserialize)]
pub struct Degree {
    pub degree: String,
    pub institution: String,
    pub period: String,
    pub description: String,
}

#[derive(Clone, Deserialize)]
struct Education {
    degrees: Vec<Degree>,
}

#[derive(Clone, Deserialize)]
pub struct Highlight {
    pub title: String,
    pub description: String,
    pub link: String,
    pub image: String,
    pub demo_link: String,
    pub demo_label: String,
}

#[derive(Clone, Deserialize)]
struct Highlights {
    items: Vec<Highlight>,
}

#[derive(Clone, Deserialize)]
pub struct CrateItem {
    pub title: String,
    pub description: String,
    pub link: String,
    pub technologies: Vec<String>,
}

#[derive(Clone, Deserialize)]
struct Crates {
    items: Vec<CrateItem>,
}

#[derive(Clone, Deserialize)]
pub struct Project {
    pub title: String,
    pub language: String,
    pub description: String,
    pub link: String,
    pub technologies: Vec<String>,
}

#[derive(Clone, Deserialize)]
struct Projects {
    items: Vec<Project>,
}

#[derive(Clone, Deserialize)]
pub struct Skills {
    pub top_items: Vec<String>,
    pub items: Vec<String>,
}

/// Every portfolio fact, parsed once at startup from the TOML files embedded
/// at compile time. Components receive `&'static Content`, so content edits
/// never touch component code.
pub struct Content {
    pub about: About,
    pub jobs: Vec<Job>,
    pub degrees: Vec<Degree>,
    pub highlights: Vec<Highlight>,
    pub crates: Vec<CrateItem>,
    pub projects: Vec<Project>,
    pub skills: Skills,
}

/// Parses the embedded TOML data and leaks it for a `'static` borrow shared by
/// every component.
pub fn load() -> &'static Content {
    let about: About =
        toml::from_str(include_str!("../data/about.toml")).expect("about.toml is invalid");
    let experience: Experience = toml::from_str(include_str!("../data/experience.toml"))
        .expect("experience.toml is invalid");
    let education: Education =
        toml::from_str(include_str!("../data/education.toml")).expect("education.toml is invalid");
    let highlights: Highlights = toml::from_str(include_str!("../data/highlights.toml"))
        .expect("highlights.toml is invalid");
    let crates: Crates =
        toml::from_str(include_str!("../data/crates.toml")).expect("crates.toml is invalid");
    let projects: Projects =
        toml::from_str(include_str!("../data/projects.toml")).expect("projects.toml is invalid");
    let skills: Skills =
        toml::from_str(include_str!("../data/skills.toml")).expect("skills.toml is invalid");

    Box::leak(Box::new(Content {
        about,
        jobs: experience.jobs,
        degrees: education.degrees,
        highlights: highlights.items,
        crates: crates.items,
        projects: projects.items,
        skills,
    }))
}
