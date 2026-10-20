use anyhow::{bail, Context};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use std::sync::{Mutex, OnceLock};

const FRIENDS_FILE: &str = "helium_client_friends.json";

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
struct Friend {
    #[serde(default)]
    uuid: Option<String>,
    name: String,
}

#[derive(Default)]
struct FriendStore {
    friends: Vec<Friend>,
}

static STORE: OnceLock<Result<Mutex<FriendStore>, String>> = OnceLock::new();

fn path() -> PathBuf {
    crate::config::base_dir().join(FRIENDS_FILE)
}

fn store() -> anyhow::Result<&'static Mutex<FriendStore>> {
    match STORE.get_or_init(|| {
        let path = path();
        let friends = match fs::read(&path) {
            Ok(bytes) => serde_json::from_slice(&bytes)
                .with_context(|| format!("parsing friend list {}", path.display()))
                .map_err(|error| error.to_string())?,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => Vec::new(),
            Err(error) => {
                return Err(format!("reading friend list {}: {error}", path.display()));
            }
        };
        Ok(Mutex::new(FriendStore { friends }))
    }) {
        Ok(store) => Ok(store),
        Err(error) => bail!("{error}"),
    }
}

fn normalize_uuid(uuid: Option<&str>) -> Option<String> {
    uuid.map(str::trim)
        .filter(|uuid| !uuid.is_empty())
        .map(str::to_ascii_lowercase)
}

fn normalize_name(name: &str) -> String {
    name.trim().to_ascii_lowercase()
}

fn matches(friend: &Friend, uuid: Option<&str>, name: &str) -> bool {
    let uuid = normalize_uuid(uuid);
    match (uuid.as_deref(), friend.uuid.as_deref()) {
        (Some(uuid), Some(saved_uuid)) => uuid == saved_uuid,
        (Some(_), None) => !name.is_empty() && normalize_name(&friend.name) == normalize_name(name),
        (None, _) => !name.is_empty() && normalize_name(&friend.name) == normalize_name(name),
    }
}

fn persist(friends: &[Friend]) -> anyhow::Result<()> {
    let path = path();
    let parent = path
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
        .unwrap_or_else(|| std::path::Path::new("."));
    fs::create_dir_all(parent)
        .with_context(|| format!("creating friend-list directory {}", parent.display()))?;

    let bytes = serde_json::to_vec_pretty(friends).context("serializing friend list")?;
    let temporary = path.with_extension("json.tmp");
    fs::write(&temporary, bytes)
        .with_context(|| format!("writing friend list {}", temporary.display()))?;
    if path.exists() {
        let backup = path.with_extension("json.bak");
        if backup.exists() {
            fs::remove_file(&backup).with_context(|| {
                format!("removing stale friend-list backup {}", backup.display())
            })?;
        }
        fs::rename(&path, &backup)
            .with_context(|| format!("backing up friend list {}", path.display()))?;
        if let Err(error) = fs::rename(&temporary, &path) {
            let _ = fs::rename(&backup, &path);
            return Err(error).with_context(|| {
                format!(
                    "replacing friend list {} with {}",
                    path.display(),
                    temporary.display()
                )
            });
        }
        if let Err(error) = fs::remove_file(&backup) {
            log::debug!(
                "could not remove friend-list backup {}: {error}",
                backup.display()
            );
        }
    } else {
        fs::rename(&temporary, &path).with_context(|| {
            format!(
                "creating friend list {} from {}",
                path.display(),
                temporary.display()
            )
        })?;
    }
    Ok(())
}

/// Add a friend, identified by UUID when available and by name as a fallback.
/// Returns `true` if the stored friend list changed.
pub fn add_friend(uuid: Option<&str>, name: &str) -> anyhow::Result<bool> {
    let uuid = normalize_uuid(uuid);
    let name = name.trim();
    if uuid.is_none() && name.is_empty() {
        bail!("a friend needs a UUID or a non-empty name");
    }

    let mut store = store()?
        .lock()
        .map_err(|_| anyhow::anyhow!("friend-list lock is poisoned"))?;
    let previous = store.friends.clone();
    if let Some(friend) = store.friends.iter_mut().find(|friend| {
        if let Some(uuid) = uuid.as_deref() {
            friend.uuid.as_deref() == Some(uuid)
                || (friend.uuid.is_none()
                    && !name.is_empty()
                    && normalize_name(&friend.name) == normalize_name(name))
        } else {
            !name.is_empty() && normalize_name(&friend.name) == normalize_name(name)
        }
    }) {
        let updated_uuid = uuid.clone().or_else(|| friend.uuid.clone());
        let changed = friend.uuid != updated_uuid || friend.name != name;
        if !changed {
            return Ok(false);
        }
        friend.uuid = updated_uuid;
        friend.name = name.to_owned();
    } else {
        store.friends.push(Friend {
            uuid,
            name: name.to_owned(),
        });
    }

    if let Err(error) = persist(&store.friends) {
        store.friends = previous;
        return Err(error);
    }
    Ok(true)
}

/// Remove a friend by UUID, falling back to a case-insensitive name match.
/// Returns `true` if a stored friend was removed.
pub fn remove_friend(uuid: Option<&str>, name: &str) -> anyhow::Result<bool> {
    let uuid = normalize_uuid(uuid);
    let name = name.trim();
    if uuid.is_none() && name.is_empty() {
        bail!("a friend needs a UUID or a non-empty name");
    }

    let mut store = store()?
        .lock()
        .map_err(|_| anyhow::anyhow!("friend-list lock is poisoned"))?;
    let Some(index) = store.friends.iter().position(|friend| {
        if let Some(uuid) = uuid.as_deref() {
            friend.uuid.as_deref() == Some(uuid)
                || (friend.uuid.is_none()
                    && !name.is_empty()
                    && normalize_name(&friend.name) == normalize_name(name))
        } else {
            !name.is_empty() && normalize_name(&friend.name) == normalize_name(name)
        }
    }) else {
        return Ok(false);
    };
    let removed = store.friends.remove(index);
    if let Err(error) = persist(&store.friends) {
        store.friends.insert(index, removed);
        return Err(error);
    }
    Ok(true)
}

/// Check friend status, preferring UUID matches and using name-only entries as
/// a migration/fallback path when the running game does not provide a UUID.
pub fn is_friend(uuid: Option<&str>, name: &str) -> anyhow::Result<bool> {
    if normalize_uuid(uuid).is_none() && name.trim().is_empty() {
        return Ok(false);
    }
    let store = store()?
        .lock()
        .map_err(|_| anyhow::anyhow!("friend-list lock is poisoned"))?;
    Ok(store
        .friends
        .iter()
        .any(|friend| matches(friend, uuid, name)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn uuid_matches_are_case_insensitive_and_prefer_uuid_over_other_names() {
        let friend = Friend {
            uuid: Some("abc".to_owned()),
            name: "Alex".to_owned(),
        };
        assert!(matches(&friend, Some("ABC"), "different-name"));
        assert!(!matches(&friend, Some("different"), "Alex"));
        assert!(matches(&friend, None, "Alex"));
    }

    #[test]
    fn name_only_entries_match_case_insensitively() {
        let friend = Friend {
            uuid: None,
            name: "Alex".to_owned(),
        };
        assert!(matches(&friend, Some("new-uuid"), "alex"));
        assert!(matches(&friend, None, "ALEX"));
    }
}
