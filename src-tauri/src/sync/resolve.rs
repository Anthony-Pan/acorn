//! Pure conflict-resolution decisions for the pull path. No DB, no network —
//! the engine gathers the facts, this module decides, the engine applies.

use chrono::{DateTime, Utc};

/// What the engine should do with one pulled remote change, given the link's
/// local state and the account's conflict policy.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Resolution {
    /// Remote change is the echo of our own last push — record etag, skip.
    Echo,
    /// Only the remote side changed: apply it locally.
    ApplyRemote,
    /// Only the local side changed (or policy says so): keep local; the
    /// pending push will overwrite the remote.
    KeepLocal,
    /// Remote deleted and local has no unpushed edits: delete locally too.
    DeleteLocal,
    /// Remote deleted but local has unpushed edits: never silently drop a
    /// user edit — resurrect the item remotely as a fresh create.
    ResurrectRemote,
}

/// Per-account conflict policy (`sync_accounts.conflict_policy`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConflictPolicy {
    /// Last write wins by comparing timestamps.
    Lww,
    LocalWins,
    RemoteWins,
}

impl ConflictPolicy {
    pub fn parse(s: &str) -> Self {
        match s {
            "local_wins" => Self::LocalWins,
            "remote_wins" => Self::RemoteWins,
            _ => Self::Lww,
        }
    }
}

/// The facts about one link the engine hands to [`resolve`].
#[derive(Debug, Clone, Copy)]
pub struct LinkFacts<'a> {
    /// The link has local edits not yet pushed (`local_rev > synced_local_rev`).
    pub local_dirty: bool,
    /// Hash of the last content we pushed (`sync_links.content_hash`).
    pub synced_hash: Option<&'a str>,
    /// Local task's `updated_at`.
    pub local_updated: DateTime<Utc>,
}

/// The facts about the pulled remote change.
#[derive(Debug, Clone, Copy)]
pub struct RemoteFacts<'a> {
    pub deleted: bool,
    /// Hash of the pulled remote content (None when deleted).
    pub incoming_hash: Option<&'a str>,
    /// Remote `updated` timestamp, when the provider supplies one.
    pub remote_updated: Option<DateTime<Utc>>,
}

/// Decide what to do with one pulled change.
pub fn resolve(link: LinkFacts<'_>, remote: RemoteFacts<'_>, policy: ConflictPolicy) -> Resolution {
    if remote.deleted {
        return if link.local_dirty {
            Resolution::ResurrectRemote
        } else {
            Resolution::DeleteLocal
        };
    }

    // Content identical to what we last pushed → the echo of our own write.
    // This also covers "no remote change" reads (EventKit per-item verify).
    if remote.incoming_hash.is_some() && remote.incoming_hash == link.synced_hash {
        return Resolution::Echo;
    }

    // Remote genuinely changed. If local is clean, remote is the only writer.
    if !link.local_dirty {
        return Resolution::ApplyRemote;
    }

    // Both sides changed — a true conflict.
    match policy {
        ConflictPolicy::LocalWins => Resolution::KeepLocal,
        ConflictPolicy::RemoteWins => Resolution::ApplyRemote,
        ConflictPolicy::Lww => match remote.remote_updated {
            Some(remote_updated) if remote_updated > link.local_updated => Resolution::ApplyRemote,
            // Tie or unknown remote clock → keep the user's local edit.
            _ => Resolution::KeepLocal,
        },
    }
}

#[cfg(test)]
mod tests {
    use chrono::TimeZone;

    use super::*;

    fn at(hour: u32) -> DateTime<Utc> {
        Utc.with_ymd_and_hms(2026, 7, 15, hour, 0, 0).unwrap()
    }

    fn clean_link(hash: &str) -> LinkFacts<'_> {
        LinkFacts {
            local_dirty: false,
            synced_hash: Some(hash),
            local_updated: at(10),
        }
    }

    #[test]
    fn echo_of_own_push_is_skipped() {
        let link = clean_link("abc");
        let remote = RemoteFacts {
            deleted: false,
            incoming_hash: Some("abc"),
            remote_updated: Some(at(11)),
        };
        assert_eq!(resolve(link, remote, ConflictPolicy::Lww), Resolution::Echo);
    }

    #[test]
    fn remote_only_change_applies() {
        let link = clean_link("abc");
        let remote = RemoteFacts {
            deleted: false,
            incoming_hash: Some("xyz"),
            remote_updated: Some(at(11)),
        };
        assert_eq!(
            resolve(link, remote, ConflictPolicy::Lww),
            Resolution::ApplyRemote
        );
    }

    #[test]
    fn both_changed_lww_newer_remote_wins() {
        let link = LinkFacts {
            local_dirty: true,
            synced_hash: Some("abc"),
            local_updated: at(10),
        };
        let remote = RemoteFacts {
            deleted: false,
            incoming_hash: Some("xyz"),
            remote_updated: Some(at(12)),
        };
        assert_eq!(
            resolve(link, remote, ConflictPolicy::Lww),
            Resolution::ApplyRemote
        );
    }

    #[test]
    fn both_changed_lww_newer_local_wins() {
        let link = LinkFacts {
            local_dirty: true,
            synced_hash: Some("abc"),
            local_updated: at(12),
        };
        let remote = RemoteFacts {
            deleted: false,
            incoming_hash: Some("xyz"),
            remote_updated: Some(at(11)),
        };
        assert_eq!(
            resolve(link, remote, ConflictPolicy::Lww),
            Resolution::KeepLocal
        );
    }

    #[test]
    fn both_changed_lww_unknown_remote_clock_keeps_local() {
        let link = LinkFacts {
            local_dirty: true,
            synced_hash: Some("abc"),
            local_updated: at(10),
        };
        let remote = RemoteFacts {
            deleted: false,
            incoming_hash: Some("xyz"),
            remote_updated: None,
        };
        assert_eq!(
            resolve(link, remote, ConflictPolicy::Lww),
            Resolution::KeepLocal
        );
    }

    #[test]
    fn both_changed_policy_overrides_lww() {
        let link = LinkFacts {
            local_dirty: true,
            synced_hash: Some("abc"),
            local_updated: at(10),
        };
        let newer_remote = RemoteFacts {
            deleted: false,
            incoming_hash: Some("xyz"),
            remote_updated: Some(at(12)),
        };
        assert_eq!(
            resolve(link, newer_remote, ConflictPolicy::LocalWins),
            Resolution::KeepLocal
        );
        let older_remote = RemoteFacts {
            deleted: false,
            incoming_hash: Some("xyz"),
            remote_updated: Some(at(9)),
        };
        assert_eq!(
            resolve(link, older_remote, ConflictPolicy::RemoteWins),
            Resolution::ApplyRemote
        );
    }

    #[test]
    fn remote_delete_with_clean_local_deletes() {
        let link = clean_link("abc");
        let remote = RemoteFacts {
            deleted: true,
            incoming_hash: None,
            remote_updated: None,
        };
        assert_eq!(
            resolve(link, remote, ConflictPolicy::Lww),
            Resolution::DeleteLocal
        );
    }

    #[test]
    fn remote_delete_with_unpushed_edit_resurrects() {
        let link = LinkFacts {
            local_dirty: true,
            synced_hash: Some("abc"),
            local_updated: at(10),
        };
        let remote = RemoteFacts {
            deleted: true,
            incoming_hash: None,
            remote_updated: Some(at(12)),
        };
        // Even remote_wins must not silently drop an unpushed user edit.
        for policy in [
            ConflictPolicy::Lww,
            ConflictPolicy::LocalWins,
            ConflictPolicy::RemoteWins,
        ] {
            assert_eq!(resolve(link, remote, policy), Resolution::ResurrectRemote);
        }
    }

    #[test]
    fn policy_parse_defaults_to_lww() {
        assert_eq!(
            ConflictPolicy::parse("local_wins"),
            ConflictPolicy::LocalWins
        );
        assert_eq!(
            ConflictPolicy::parse("remote_wins"),
            ConflictPolicy::RemoteWins
        );
        assert_eq!(ConflictPolicy::parse("lww"), ConflictPolicy::Lww);
        assert_eq!(ConflictPolicy::parse("garbage"), ConflictPolicy::Lww);
    }
}
