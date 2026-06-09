use std::collections::BTreeMap;
use std::sync::{Arc, Mutex};

use crate::ecs::PortfolioWorld;
use nightshade::prelude::{ehttp, serde_json, *};
use serde::Deserialize;

const HDRI_FILES: &str = "https://api.polyhaven.com/files/";
const SKYBOX: &str = "kloofendal_48d_partly_cloudy_puresky";
const RESOLUTION: u32 = 4;

#[derive(Deserialize)]
struct FileLink {
    url: String,
}

#[derive(Deserialize)]
struct HdriResolution {
    #[serde(default)]
    hdr: Option<FileLink>,
}

#[derive(Deserialize)]
struct HdriFiles {
    hdri: BTreeMap<String, HdriResolution>,
}

/// Kicks off the skybox download: resolve the pinned Polyhaven sky's 4k HDR
/// file and stash the bytes for `poll` to apply on the render thread.
pub fn fetch(portfolio: &PortfolioWorld) {
    let slot = Arc::clone(&portfolio.resources.sky.bytes);
    let files_url = format!("{HDRI_FILES}{SKYBOX}");
    ehttp::fetch(ehttp::Request::get(&files_url), move |result| {
        let url = result
            .ok()
            .filter(|response| response.ok)
            .and_then(|response| serde_json::from_slice::<HdriFiles>(&response.bytes).ok())
            .and_then(pick_hdr);
        if let Some(url) = url {
            download(url, slot);
        }
    });
}

/// Picks the exact requested resolution, else the highest available below it,
/// else the smallest.
fn pick_hdr(files: HdriFiles) -> Option<String> {
    let mut entries: Vec<(u32, String)> = files
        .hdri
        .into_iter()
        .filter_map(|(key, resolution)| {
            resolution.hdr.map(|link| {
                (
                    key.trim_end_matches(['k', 'K']).parse().unwrap_or(u32::MAX),
                    link.url,
                )
            })
        })
        .collect();
    entries.sort_by_key(|(value, _)| *value);
    if let Some(index) = entries.iter().position(|(value, _)| *value == RESOLUTION) {
        return Some(entries.swap_remove(index).1);
    }
    if let Some(index) = entries.iter().rposition(|(value, _)| *value <= RESOLUTION) {
        return Some(entries.swap_remove(index).1);
    }
    entries.into_iter().next().map(|(_, url)| url)
}

fn download(url: String, slot: Arc<Mutex<Option<Vec<u8>>>>) {
    ehttp::fetch(ehttp::Request::get(&url), move |result| {
        if let Ok(response) = result
            && response.ok
            && let Ok(mut guard) = slot.lock()
        {
            *guard = Some(response.bytes);
        }
    });
}

/// Applies the downloaded skybox once its bytes arrive: swap the procedural
/// atmosphere for the HDR sky and its image-based lighting.
pub fn poll(portfolio: &mut PortfolioWorld, world: &mut World) {
    let bytes = portfolio
        .resources
        .sky
        .bytes
        .lock()
        .ok()
        .and_then(|mut slot| slot.take());
    let Some(bytes) = bytes else {
        return;
    };
    world.resources.render_settings.atmosphere = Atmosphere::Hdr;
    load_hdr_skybox(world, bytes);
}
