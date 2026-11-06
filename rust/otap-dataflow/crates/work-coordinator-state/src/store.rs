// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use crate::{Error, Grant, Limits, OperationId, Outcome, Progress, ProgressUpdate};
use fs2::FileExt;
use rusqlite::{
    Connection, OpenFlags, OptionalExtension, Transaction, TransactionBehavior, params,
};
use sha2::{Digest, Sha256};
use std::{
    collections::HashSet,
    fs::{self, File, OpenOptions},
    io::{Read, Write},
    os::unix::fs::{DirBuilderExt, MetadataExt, OpenOptionsExt},
    path::Path,
};

const APPLICATION: i64 = 0x5743_5331;
const MAGIC: &[u8; 8] = b"DFEWC001";
const SCHEMA: &str = "
CREATE TABLE meta (id INTEGER PRIMARY KEY CHECK(id=1), namespace TEXT NOT NULL,
    identity BLOB NOT NULL CHECK(length(identity)=16), epoch INTEGER NOT NULL CHECK(epoch>=0),
    highwater INTEGER NOT NULL CHECK(highwater>=0),
    assignments INTEGER NOT NULL CHECK(assignments>=0),
    sources INTEGER NOT NULL CHECK(sources>=0),
    progress_bytes INTEGER NOT NULL CHECK(progress_bytes>=0));
CREATE TABLE grants (work TEXT PRIMARY KEY, term INTEGER NOT NULL CHECK(term>0),
    token BLOB NOT NULL, active INTEGER NOT NULL CHECK(active IN (0,1))) WITHOUT ROWID;
CREATE TABLE progress (source TEXT PRIMARY KEY, work TEXT NOT NULL,
    revision INTEGER NOT NULL CHECK(revision>0), bytes BLOB NOT NULL) WITHOUT ROWID;
CREATE TABLE receipts (epoch INTEGER NOT NULL, sequence INTEGER NOT NULL,
    digest BLOB NOT NULL CHECK(length(digest)=32), outcome INTEGER NOT NULL CHECK(outcome IN (0,1)),
    PRIMARY KEY(epoch, sequence)) WITHOUT ROWID;
PRAGMA application_id=1464030001;
PRAGMA user_version=1;
";

enum Command<'a> {
    Activate(&'a Grant),
    Retire(&'a Grant),
    Progress(&'a [ProgressUpdate]),
}

/// Exclusive, blocking local authority. Keep the namespace private and do not
/// rename or modify its artifacts while open. Drop closes SQLite before unlocking.
pub struct Store {
    connection: Connection,
    _lock: File,
    _directory: File,
    epoch: u64,
    limits: Limits,
    poisoned: bool,
    #[cfg(feature = "test-utils")]
    hook: Option<Box<dyn FnMut(crate::FaultPoint) + Send>>,
}

fn sql(error: rusqlite::Error) -> Error {
    Error::Storage(error.to_string())
}
fn number(value: u64) -> Result<i64, Error> {
    i64::try_from(value).map_err(|_| Error::Invalid("durable integer overflow"))
}
fn key(value: &str) -> bool {
    !value.is_empty() && value.len() <= 1024 && !value.contains('\0')
}
fn valid_grant(grant: &Grant) -> bool {
    key(&grant.work)
        && grant.term > 0
        && grant.term <= i64::MAX as u64
        && !grant.token.is_empty()
        && grant.token.len() <= 4096
}
fn private(path: &Path, directory: bool, maximum: u64) -> Result<(), Error> {
    let metadata = fs::symlink_metadata(path)?;
    let expected = if directory { 0o700 } else { 0o600 };
    if metadata.uid() != nix::unistd::geteuid().as_raw() {
        return Err(Error::Untrusted("artifact owner"));
    }
    if metadata.mode() & (if directory { 0o777 } else { 0o7777 }) != expected {
        return Err(Error::Untrusted("artifact permissions"));
    }
    if (directory && !metadata.is_dir())
        || (!directory
            && (!metadata.is_file() || metadata.nlink() != 1 || metadata.len() > maximum))
    {
        return Err(Error::Untrusted(
            "artifact type, owner, permissions, links, or size",
        ));
    }
    Ok(())
}

impl Store {
    /// Open beneath an existing private 0700 state root. The caller must protect
    /// the root's ancestors. Each successful exclusive open fences old grants in
    /// one durable epoch transaction and preserves all committed source progress.
    pub fn open(root: &Path, namespace: &str, limits: Limits) -> Result<Self, Error> {
        let limits = limits.validate()?;
        if !root.is_absolute() || !key(namespace) {
            return Err(Error::Invalid(
                "absolute state root and bounded namespace required",
            ));
        }
        private(root, true, 0)?;
        let digest = Sha256::digest(namespace.as_bytes());
        let encoded: String = digest
            .iter()
            .flat_map(|b| {
                [
                    char::from(b"0123456789abcdef"[(b >> 4) as usize]),
                    char::from(b"0123456789abcdef"[(b & 15) as usize]),
                ]
            })
            .collect();
        let path = root.join(format!("work-coordinator-v1-{encoded}"));
        match fs::DirBuilder::new().mode(0o700).create(&path) {
            Ok(()) => File::open(root)?.sync_all()?,
            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {}
            Err(error) => return Err(error.into()),
        }
        private(&path, true, 0)?;
        // An existing directory may be an interrupted mkdir whose parent link
        // was never synchronized. Close that publication window on every open.
        File::open(root)?.sync_all()?;
        let directory = OpenOptions::new()
            .read(true)
            .custom_flags(nix::libc::O_DIRECTORY | nix::libc::O_NOFOLLOW)
            .open(&path)?;
        let lock_path = path.join("ownership.lock");
        let lock = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(false)
            .mode(0o600)
            .custom_flags(nix::libc::O_NOFOLLOW)
            .open(&lock_path)?;
        private(&lock_path, false, 0)?;
        lock.try_lock_exclusive().map_err(|e| {
            if e.kind() == std::io::ErrorKind::WouldBlock {
                Error::Locked
            } else {
                e.into()
            }
        })?;
        let database = path.join("state.sqlite");
        let marker_path = path.join("initialized");
        let mut marker = None;
        for entry in fs::read_dir(&path)? {
            let entry = entry?;
            let name = entry.file_name();
            let maximum = match name.to_str() {
                Some("ownership.lock") => 0,
                Some("initialized") => 24,
                Some("state.sqlite") => limits.database_bytes,
                Some("state.sqlite-journal") => limits.database_bytes * 2,
                _ => return Err(Error::Untrusted("unexpected namespace artifact")),
            };
            private(&entry.path(), false, maximum)?;
        }
        if marker_path.exists() {
            let mut bytes = [0u8; 24];
            let mut file = OpenOptions::new()
                .read(true)
                .custom_flags(nix::libc::O_NOFOLLOW)
                .open(&marker_path)?;
            file.read_exact(&mut bytes)?;
            if &bytes[..8] != MAGIC {
                return Err(Error::Untrusted("initialization marker"));
            }
            marker = Some(bytes[8..].to_vec());
        }
        if !database.exists() {
            if marker.is_some() || path.join("state.sqlite-journal").exists() {
                return Err(Error::Untrusted("authoritative database missing"));
            }
            let _ = OpenOptions::new()
                .read(true)
                .write(true)
                .create_new(true)
                .mode(0o600)
                .open(&database)?;
        }
        let mut connection = Connection::open_with_flags(
            &database,
            OpenFlags::SQLITE_OPEN_READ_WRITE
                | OpenFlags::SQLITE_OPEN_NO_MUTEX
                | OpenFlags::SQLITE_OPEN_NOFOLLOW,
        )
        .map_err(sql)?;
        connection
            .busy_timeout(std::time::Duration::ZERO)
            .map_err(sql)?;
        let version: i64 = connection
            .pragma_query_value(None, "user_version", |r| r.get(0))
            .map_err(sql)?;
        let application: i64 = connection
            .pragma_query_value(None, "application_id", |r| r.get(0))
            .map_err(sql)?;
        if !((version == 0 && application == 0 && marker.is_none())
            || (version == 1 && application == APPLICATION))
        {
            return Err(Error::Untrusted("unsupported storage format"));
        }
        connection
            .execute_batch(
                "PRAGMA journal_mode=DELETE; PRAGMA synchronous=EXTRA;
            PRAGMA trusted_schema=OFF; PRAGMA temp_store=MEMORY; PRAGMA cache_size=-1024;
            PRAGMA mmap_size=0; PRAGMA page_size=4096;",
            )
            .map_err(sql)?;
        let page_size: u64 = connection
            .pragma_query_value(None, "page_size", |r| r.get(0))
            .map_err(sql)?;
        let sync: i64 = connection
            .pragma_query_value(None, "synchronous", |r| r.get(0))
            .map_err(sql)?;
        let journal: String = connection
            .pragma_query_value(None, "journal_mode", |r| r.get(0))
            .map_err(sql)?;
        if page_size != 4096 || sync != 3 || journal != "delete" {
            return Err(Error::Untrusted("unsupported SQLite durability settings"));
        }
        connection
            .pragma_update(None, "max_page_count", limits.database_bytes / 4096)
            .map_err(sql)?;
        let version: i64 = connection
            .pragma_query_value(None, "user_version", |r| r.get(0))
            .map_err(sql)?;
        if version == 0 && marker.is_none() {
            let tables: i64 = connection
                .query_row(
                    "SELECT count(*) FROM sqlite_schema WHERE name NOT LIKE 'sqlite_%'",
                    [],
                    |r| r.get(0),
                )
                .map_err(sql)?;
            if tables != 0 {
                return Err(Error::Untrusted("foreign database"));
            }
            let tx = connection.transaction().map_err(sql)?;
            tx.execute_batch(SCHEMA).map_err(sql)?;
            let _ = tx
                .execute(
                    "INSERT INTO meta VALUES(1, ?1, ?2, 0, 0, 0, 0, 0)",
                    params![namespace, uuid::Uuid::new_v4().as_bytes()],
                )
                .map_err(sql)?;
            tx.commit().map_err(sql)?;
        }
        let application: i64 = connection
            .pragma_query_value(None, "application_id", |r| r.get(0))
            .map_err(sql)?;
        let version: i64 = connection
            .pragma_query_value(None, "user_version", |r| r.get(0))
            .map_err(sql)?;
        if version != 1 || application != APPLICATION {
            return Err(Error::Untrusted("unsupported storage format"));
        }
        let check: String = connection
            .query_row("PRAGMA quick_check(1)", [], |r| r.get(0))
            .map_err(sql)?;
        if check != "ok" {
            return Err(Error::Untrusted("database integrity check failed"));
        }
        let (stored_namespace, identity, epoch, highwater): (String, Vec<u8>, u64, u64) =
            connection
                .query_row(
                    "SELECT namespace, identity, epoch, highwater FROM meta WHERE id=1",
                    [],
                    |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?)),
                )
                .map_err(sql)?;
        if stored_namespace != namespace
            || identity.len() != 16
            || marker.as_ref().is_some_and(|m| *m != identity)
        {
            return Err(Error::Untrusted("namespace identity changed"));
        }
        let (sources, bytes): (usize, usize) = connection
            .query_row(
                "SELECT count(*), coalesce(sum(length(bytes)), 0) FROM progress",
                [],
                |r| Ok((r.get(0)?, r.get(1)?)),
            )
            .map_err(sql)?;
        if sources > limits.sources || bytes > limits.progress_bytes {
            return Err(Error::Invalid(
                "persisted progress exceeds configured bounds",
            ));
        }
        let largest: usize = connection
            .query_row(
                "SELECT coalesce(max(length(bytes)), 0) FROM progress",
                [],
                |r| r.get(0),
            )
            .map_err(sql)?;
        if largest > limits.source_bytes {
            return Err(Error::Invalid("persisted source exceeds configured bounds"));
        }
        let recorded: (usize, usize, usize) = connection
            .query_row(
                "SELECT assignments, sources, progress_bytes FROM meta WHERE id=1",
                [],
                |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
            )
            .map_err(sql)?;
        let assignments: usize = connection
            .query_row("SELECT count(*) FROM grants", [], |r| r.get(0))
            .map_err(sql)?;
        if recorded != (assignments, sources, bytes) {
            return Err(Error::Untrusted(
                "persistent admission counters disagree with state",
            ));
        }
        let existing: usize = connection.query_row("SELECT (SELECT count(*) FROM grants) + (SELECT count(*) FROM receipts) + (SELECT count(*) FROM progress)", [], |r| r.get(0)).map_err(sql)?;
        if marker.is_none() && existing != 0 {
            return Err(Error::Untrusted("published state is missing its marker"));
        }
        let next_epoch = epoch
            .checked_add(1)
            .ok_or(Error::Invalid("authority epoch overflow"))?;
        let tx = connection
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(sql)?;
        let _ = tx.execute("DELETE FROM grants", []).map_err(sql)?;
        let _ = tx
            .execute(
                "DELETE FROM receipts WHERE epoch < ?1 OR sequence <= ?2",
                params![
                    number(epoch)?,
                    number(highwater.saturating_sub(limits.receipts as u64))?
                ],
            )
            .map_err(sql)?;
        let _ = tx
            .execute(
                "UPDATE meta SET epoch=?1, highwater=0, assignments=0 WHERE id=1",
                [number(next_epoch)?],
            )
            .map_err(sql)?;
        tx.commit().map_err(sql)?;
        if marker.is_none() {
            let mut file = OpenOptions::new()
                .write(true)
                .create_new(true)
                .mode(0o600)
                .custom_flags(nix::libc::O_NOFOLLOW)
                .open(&marker_path)?;
            file.write_all(MAGIC)?;
            file.write_all(&identity)?;
            file.sync_all()?;
        }
        directory.sync_all()?;
        Ok(Self {
            connection,
            _lock: lock,
            _directory: directory,
            epoch: next_epoch,
            limits,
            poisoned: false,
            #[cfg(feature = "test-utils")]
            hook: None,
        })
    }

    /// Epoch fenced into the same database before this store became available.
    #[must_use]
    pub fn epoch(&self) -> u64 {
        self.epoch
    }

    /// Inject process interruption at storage boundaries in explicit test builds.
    #[cfg(feature = "test-utils")]
    pub fn set_fault_hook(&mut self, hook: impl FnMut(crate::FaultPoint) + Send + 'static) {
        self.hook = Some(Box::new(hook));
    }

    /// Persist an exact grant before allowing its local installation.
    pub fn activate(&mut self, operation: OperationId, grant: &Grant) -> Result<Outcome, Error> {
        self.apply(operation, Command::Activate(grant))
    }
    /// Fence this exact grant. All subsequent source mutations reject it.
    pub fn retire(&mut self, operation: OperationId, grant: &Grant) -> Result<Outcome, Error> {
        self.apply(operation, Command::Retire(grant))
    }
    /// Commit all conditional updates or none, including when several assignments contribute.
    pub fn commit(
        &mut self,
        operation: OperationId,
        updates: &[ProgressUpdate],
    ) -> Result<Outcome, Error> {
        self.apply(operation, Command::Progress(updates))
    }
    /// Resolve a retained receipt, including one from the immediately preceding epoch.
    /// Historical success is never proof of current ownership.
    pub fn resolve(&mut self, operation: OperationId) -> Result<Outcome, Error> {
        if self.poisoned {
            return Err(Error::Poisoned);
        }
        let result = self
            .connection
            .query_row(
                "SELECT outcome FROM receipts WHERE epoch=?1 AND sequence=?2",
                params![number(operation.epoch)?, number(operation.sequence)?],
                |r| r.get::<_, i64>(0),
            )
            .optional()
            .map_err(sql);
        self.observe(result.map(|result| {
            result.map_or(Outcome::Unknown, |v| {
                if v == 1 {
                    Outcome::Applied
                } else {
                    Outcome::Rejected
                }
            })
        }))
    }
    /// Read one bounded opaque source record; grants never own checkpoint contents.
    pub fn progress(&mut self, source: &str) -> Result<Option<Progress>, Error> {
        if self.poisoned {
            return Err(Error::Poisoned);
        }
        if !key(source) {
            return Err(Error::Invalid("invalid source ID"));
        }
        let result = self
            .connection
            .query_row(
                "SELECT work, revision, bytes FROM progress WHERE source=?1",
                [source],
                |r| {
                    Ok(Progress {
                        work: r.get(0)?,
                        revision: r.get(1)?,
                        bytes: r.get(2)?,
                    })
                },
            )
            .optional()
            .map_err(sql);
        self.observe(result)
    }
    fn observe<T>(&mut self, result: Result<T, Error>) -> Result<T, Error> {
        if matches!(result, Err(Error::Storage(_))) {
            self.poisoned = true;
        }
        result
    }
    fn apply(&mut self, operation: OperationId, command: Command<'_>) -> Result<Outcome, Error> {
        if self.poisoned {
            return Err(Error::Poisoned);
        }
        let _ = number(operation.epoch)?;
        let _ = number(operation.sequence)?;
        if operation.sequence == 0 {
            return Err(Error::Invalid("zero operation sequence"));
        }
        if operation.epoch != self.epoch {
            return Ok(Outcome::Rejected);
        }
        let mut digest = Sha256::new();
        let mut field = |bytes: &[u8]| {
            digest.update((bytes.len() as u64).to_le_bytes());
            digest.update(bytes);
        };
        let mut grant_field = |grant: &Grant, kind: u8| -> Result<(), Error> {
            if !valid_grant(grant) || grant.epoch != operation.epoch {
                return Err(Error::Invalid("invalid exact grant"));
            }
            field(&[kind]);
            field(grant.work.as_bytes());
            field(&grant.term.to_le_bytes());
            field(&grant.token);
            Ok(())
        };
        match &command {
            Command::Activate(g) => grant_field(g, 1)?,
            Command::Retire(g) => grant_field(g, 2)?,
            Command::Progress(updates) => {
                let mut bytes = 0usize;
                let mut sources = HashSet::new();
                if updates.is_empty() || updates.len() > self.limits.transaction_items {
                    return Err(Error::Invalid("invalid progress transaction size"));
                }
                for update in *updates {
                    if !valid_grant(&update.grant)
                        || !key(&update.source)
                        || !sources.insert(&update.source)
                        || update.progress.len() > self.limits.source_bytes
                        || update.expected_revision >= i64::MAX as u64
                    {
                        return Err(Error::Invalid("invalid progress update"));
                    }
                    bytes = bytes
                        .saturating_add(
                            update.source.len()
                                + update.grant.work.len()
                                + update.grant.token.len(),
                        )
                        .saturating_add(update.progress.len())
                        .saturating_add(64);
                    if bytes > self.limits.transaction_bytes {
                        return Err(Error::Invalid("progress transaction byte limit"));
                    }
                }
                field(&[3]);
                for update in *updates {
                    field(&update.grant.epoch.to_le_bytes());
                    field(update.grant.work.as_bytes());
                    field(&update.grant.term.to_le_bytes());
                    field(&update.grant.token);
                    field(update.source.as_bytes());
                    field(&update.expected_revision.to_le_bytes());
                    field(&update.progress);
                }
            }
        }
        let digest = digest.finalize();
        let result = (|| {
            let tx = self
                .connection
                .transaction_with_behavior(TransactionBehavior::Immediate)
                .map_err(sql)?;
            let prior: Option<(Vec<u8>, i64)> = tx
                .query_row(
                    "SELECT digest, outcome FROM receipts WHERE epoch=?1 AND sequence=?2",
                    params![number(operation.epoch)?, number(operation.sequence)?],
                    |r| Ok((r.get(0)?, r.get(1)?)),
                )
                .optional()
                .map_err(sql)?;
            if let Some((prior, outcome)) = prior {
                return Ok(if prior.as_slice() != digest.as_slice() {
                    Outcome::Rejected
                } else if outcome == 1 {
                    Outcome::Applied
                } else {
                    Outcome::Rejected
                });
            }
            let highwater: u64 = tx
                .query_row("SELECT highwater FROM meta WHERE id=1", [], |r| r.get(0))
                .map_err(sql)?;
            let floor = highwater.saturating_sub(self.limits.receipts as u64);
            if operation.sequence <= floor {
                return Ok(Outcome::Unknown);
            }
            let applied = mutate(
                &tx,
                &command,
                self.epoch,
                self.limits,
                #[cfg(feature = "test-utils")]
                &mut self.hook,
            )?;
            let highwater = highwater.max(operation.sequence);
            let _ = tx
                .execute(
                    "UPDATE meta SET highwater=?1 WHERE id=1",
                    [number(highwater)?],
                )
                .map_err(sql)?;
            let _ = tx
                .execute(
                    "DELETE FROM receipts WHERE epoch=?1 AND sequence<=?2",
                    params![
                        number(self.epoch)?,
                        number(highwater.saturating_sub(self.limits.receipts as u64))?
                    ],
                )
                .map_err(sql)?;
            let _ = tx
                .execute(
                    "INSERT INTO receipts VALUES(?1, ?2, ?3, ?4)",
                    params![
                        number(operation.epoch)?,
                        number(operation.sequence)?,
                        digest.as_slice(),
                        i64::from(applied)
                    ],
                )
                .map_err(sql)?;
            #[cfg(feature = "test-utils")]
            if let Some(hook) = &mut self.hook {
                hook(crate::FaultPoint::BeforeCommit);
            }
            tx.commit().map_err(sql)?;
            #[cfg(feature = "test-utils")]
            if let Some(hook) = &mut self.hook {
                hook(crate::FaultPoint::AfterCommit);
            }
            Ok(if applied {
                Outcome::Applied
            } else {
                Outcome::Rejected
            })
        })();
        self.observe(result)
    }
}

fn current(tx: &Transaction<'_>, grant: &Grant, epoch: u64) -> Result<bool, Error> {
    if grant.epoch != epoch {
        return Ok(false);
    }
    let stored: Option<(u64, Vec<u8>, bool)> = tx
        .query_row(
            "SELECT term, token, active FROM grants WHERE work=?1",
            [&grant.work],
            |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
        )
        .optional()
        .map_err(sql)?;
    Ok(stored
        .is_some_and(|(term, token, active)| active && term == grant.term && token == grant.token))
}
fn mutate(
    tx: &Transaction<'_>,
    command: &Command<'_>,
    epoch: u64,
    limits: Limits,
    #[cfg(feature = "test-utils")] hook: &mut Option<Box<dyn FnMut(crate::FaultPoint) + Send>>,
) -> Result<bool, Error> {
    match command {
        Command::Activate(grant) => {
            let stored: Option<(u64, bool)> = tx
                .query_row(
                    "SELECT term, active FROM grants WHERE work=?1",
                    [&grant.work],
                    |r| Ok((r.get(0)?, r.get(1)?)),
                )
                .optional()
                .map_err(sql)?;
            let creating = stored.is_none();
            if let Some((term, active)) = stored {
                if active {
                    return current(tx, grant, epoch);
                }
                if grant.term <= term {
                    return Ok(false);
                }
            } else {
                let count: usize = tx
                    .query_row("SELECT assignments FROM meta WHERE id=1", [], |r| r.get(0))
                    .map_err(sql)?;
                if count >= limits.assignments {
                    return Ok(false);
                }
            }
            let _ = tx.execute("INSERT INTO grants VALUES(?1, ?2, ?3, 1) ON CONFLICT(work) DO UPDATE SET term=excluded.term, token=excluded.token, active=1", params![grant.work, number(grant.term)?, grant.token]).map_err(sql)?;
            if creating {
                let _ = tx
                    .execute("UPDATE meta SET assignments=assignments+1 WHERE id=1", [])
                    .map_err(sql)?;
            }
        }
        Command::Retire(grant) => {
            if !current(tx, grant, epoch)? {
                return Ok(false);
            }
            let _ = tx
                .execute("UPDATE grants SET active=0 WHERE work=?1", [&grant.work])
                .map_err(sql)?;
        }
        Command::Progress(updates) => {
            let (mut count, mut bytes): (usize, usize) = tx
                .query_row(
                    "SELECT sources, progress_bytes FROM meta WHERE id=1",
                    [],
                    |r| Ok((r.get(0)?, r.get(1)?)),
                )
                .map_err(sql)?;
            for update in *updates {
                if !current(tx, &update.grant, epoch)? {
                    return Ok(false);
                }
                let stored: Option<(String, u64, usize)> = tx
                    .query_row(
                        "SELECT work, revision, length(bytes) FROM progress WHERE source=?1",
                        [&update.source],
                        |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
                    )
                    .optional()
                    .map_err(sql)?;
                match stored {
                    Some((work, revision, size))
                        if work == update.grant.work && revision == update.expected_revision =>
                    {
                        bytes -= size
                    }
                    None if update.expected_revision == 0 => count += 1,
                    _ => return Ok(false),
                }
                bytes = bytes.saturating_add(update.progress.len());
            }
            if count > limits.sources || bytes > limits.progress_bytes {
                return Ok(false);
            }
            for (index, update) in updates.iter().enumerate() {
                let _ = index;
                let _ = tx.execute("INSERT INTO progress VALUES(?1, ?2, ?3, ?4) ON CONFLICT(source) DO UPDATE SET revision=excluded.revision, bytes=excluded.bytes", params![update.source, update.grant.work, number(update.expected_revision + 1)?, update.progress]).map_err(sql)?;
                #[cfg(feature = "test-utils")]
                if index == 0
                    && let Some(hook) = hook
                {
                    hook(crate::FaultPoint::AfterFirstUpdate);
                }
            }
            let _ = tx
                .execute(
                    "UPDATE meta SET sources=?1, progress_bytes=?2 WHERE id=1",
                    params![count, bytes],
                )
                .map_err(sql)?;
        }
    }
    Ok(true)
}
