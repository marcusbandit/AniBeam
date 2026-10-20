//! Endpoint writes: a pack or a pasted URL into anirss, one endpoint out,
//! and the configured list back out as a pack. The writes go through
//! anirss's endpoint API, which an anirss lists as the `endpoint_writes`
//! feature; one without it is refused before anything is sent.

use std::sync::Arc;

use super::anirss::AnirssClient;
use super::*;

/// The feature `/health` lists when `POST /endpoints` and
/// `DELETE /endpoints/{name}` exist.
const ENDPOINT_WRITES: &str = "endpoint_writes";

/// The client, once anirss has said it takes endpoint writes.
async fn writable(core: &Arc<Core>) -> Result<AnirssClient, CoreError> {
    let client = ensure_client(core).await?;
    if !client
        .features()
        .await?
        .iter()
        .any(|f| f == ENDPOINT_WRITES)
    {
        let version = core.downloads.state().anirss.version.clone();
        return Err(service(
            anirss::SERVICE,
            "outdated",
            format!(
                "anirss {version} cannot add or remove endpoints. Update anirss to 0.6 or later."
            ),
        ));
    }
    Ok(client)
}

/// Adds a pack's endpoints in its order. The pack is checked before the
/// job starts, so a bad file fails at once and sends nothing.
pub fn import_pack(core: &Core, text: &str) -> Result<u64, CoreError> {
    let pack = pack::parse(text)?;
    start_import(core, pack.endpoints, pack.name)
}

/// One endpoint from a pasted URL.
pub fn add_url(core: &Core, url: &str, name: &str) -> Result<u64, CoreError> {
    let endpoint = pack::from_url(url, name)?;
    start_import(core, vec![endpoint], String::new())
}

fn start_import(
    core: &Core,
    endpoints: Vec<PackEndpoint>,
    pack_name: String,
) -> Result<u64, CoreError> {
    let owner = core.owner()?;
    Ok(owner
        .jobs
        .clone()
        .start(JobKind::ImportEndpoints, move |ctx| async move {
            let client = writable(&owner).await?;
            let total = endpoints.len() as u64;
            let (mut added, mut replaced) = (Vec::new(), Vec::new());
            for (i, ep) in endpoints.iter().enumerate() {
                ctx.progress(i as u64, Some(total), &format!("Adding {}", ep.name));
                if client.add_endpoint(ep).await? {
                    replaced.push(ep.name.clone());
                } else {
                    added.push(ep.name.clone());
                }
            }
            read_anirss(&owner, &client).await?;
            let status = status_async(&owner).await?;
            let mut message = match (added.len(), replaced.len()) {
                (_, 0) => format!("Added {}", list(&added)),
                (0, _) => format!("Replaced {}", list(&replaced)),
                _ => format!("Added {}, replaced {}", list(&added), list(&replaced)),
            };
            if !pack_name.is_empty() {
                message = format!("{message} from {pack_name}");
            }
            Ok(Finished {
                level: Level::Info,
                message,
                body: EventBody::DownloadsStatusChanged { status },
            })
        }))
}

fn list(names: &[String]) -> String {
    match names {
        [one] => format!("endpoint {one}"),
        many => format!("endpoints {}", many.join(", ")),
    }
}

pub fn remove(core: &Core, name: String) -> Result<u64, CoreError> {
    let name = name.trim().to_string();
    if name.is_empty() {
        return Err(CoreError::invalid("name", "no endpoint named"));
    }
    let owner = core.owner()?;
    Ok(owner
        .jobs
        .clone()
        .start(JobKind::RemoveEndpoint, move |_| async move {
            let client = writable(&owner).await?;
            client.remove_endpoint(&name).await?;
            read_anirss(&owner, &client).await?;
            let status = status_async(&owner).await?;
            Ok(Finished {
                level: Level::Info,
                message: format!("Removed endpoint {name}"),
                body: EventBody::DownloadsStatusChanged { status },
            })
        }))
}

/// The configured endpoints as a pack, read off anirss's own config file
/// so URLs come out whole: `GET /endpoints` masks tokens. A pack is a
/// file the owner keeps or hands on; a token in it is theirs to share.
/// A remote anirss keeps its file to itself, so it has no export.
pub fn export(core: &Core) -> Result<EndpointPack, CoreError> {
    let s = core.store.read(settings::load_stored)?;
    if s.anirss_mode == AnirssMode::Remote {
        return Err(CoreError::Unsupported {
            what: "exporting the endpoints of a remote anirss".into(),
        });
    }
    let path = settings::anirss_config_path()
        .ok_or_else(|| CoreError::internal("no home directory to find anirss's config in"))?;
    let text = std::fs::read_to_string(&path).map_err(|e| {
        CoreError::invalid(
            "config",
            format!(
                "anirss's config at {} could not be read: {e}",
                path.display()
            ),
        )
    })?;
    pack::from_anirss_config(&text, "AniBeam endpoints")
}
