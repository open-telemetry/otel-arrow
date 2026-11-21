// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Standalone Basic protocol example. For real controller/factory integration,
//! see controller/src/work_coordinator_tests.rs and docs/work-coordinator.md.

use async_trait::async_trait;
use otel_arrow_dfe_engine::work_coordinator::*;
use std::{
    collections::BTreeSet,
    rc::Rc,
    sync::{Arc, mpsc},
    thread,
    time::Duration,
};
use tokio::{runtime::Builder, sync::oneshot, task::LocalSet};

#[derive(Debug)]
struct Scrapes;
impl WorkProtocol for Scrapes {
    type Work = String;
    type ParticipantRequest = ();
    type CoordinatorNotice = ();
}

struct StaticTargets {
    _local: Rc<()>,
}
#[async_trait(?Send)]
impl WorkCoordinator<Scrapes> for StaticTargets {
    async fn run(
        self: Box<Self>,
        mut runtime: CoordinatorRuntime<Scrapes>,
    ) -> Result<(), CoordinatorError> {
        runtime
            .replace_catalog(
                (0..4)
                    .map(|i| WorkSpec {
                        id: WorkId(format!("target-{i}")),
                        work: format!("http://target-{i}:9090/metrics"),
                    })
                    .collect(),
            )
            .await?;
        runtime.ready()?;
        runtime.shutdown_requested().await;
        Ok(())
    }
}

async fn instance(
    mut participant: WorkParticipant<Scrapes>,
    completed: mpsc::Sender<Vec<String>>,
    finish: oneshot::Receiver<()>,
) -> Result<(), CoordinatorError> {
    // Local scraper state can remain !Send, even though the metadata endpoint
    // was transported to this thread during setup.
    let _local_scraper = Rc::new(());
    participant.ready_for_assignments().await?;
    let mut installed = BTreeSet::new();
    let mut executed = Vec::new();
    while executed.len() < 2 {
        let ParticipantEvent::Assignments(snapshot) = participant.recv().await? else {
            continue;
        };
        for assignment in &snapshot.assignments {
            match &assignment.phase {
                AssignmentPhase::Offered(offer) => participant.accept(offer).await?,
                AssignmentPhase::Active(grant) if installed.insert(grant.identity().term) => {
                    participant.activated(grant).await?;
                    let ticket = grant.try_admit()?;
                    // A real adapter scrapes assignment.work.work and publishes its
                    // metrics here, retaining the ticket through the send/completion.
                    executed.push(assignment.work.id.0.clone());
                    assert!(ticket.is_valid());
                    drop(ticket);
                }
                AssignmentPhase::Revoking(grant) => participant.released(grant).await?,
                _ => {}
            }
        }
    }
    completed
        .send(executed)
        .map_err(|e| CoordinatorError::Failed(e.to_string()))?;
    finish
        .await
        .map_err(|e| CoordinatorError::Failed(e.to_string()))?;
    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let declaration = CoordinatorDeclaration::new::<Scrapes, _>("targets", || {
        Box::new(StaticTargets {
            _local: Rc::new(()),
        })
    });
    let key = CoordinatorKey {
        group_id: "example".into(),
        pipeline_id: "scrapes".into(),
        node_id: "prometheus".into(),
        role: "targets".into(),
    };
    let sessions = (0..2)
        .map(|core_id| ParticipantSession {
            core_id,
            generation: 0,
            nonce: core_id as u64 + 1,
        })
        .collect();
    let (binding, task) = declaration.prepare(
        key,
        sessions,
        Arc::new(|error| {
            otel_arrow_dfe_telemetry::otel_error!("example.coordinator.failed", error = error);
        }),
    )?;
    let (ready, run) = task.into_parts();
    let host = thread::spawn(move || {
        Builder::new_current_thread()
            .enable_all()
            .build()
            .expect("host runtime")
            .block_on(LocalSet::new().run_until(run()))
    });
    ready
        .recv_timeout(Duration::from_secs(30))?
        .map_err(CoordinatorError::Failed)?;
    let (completed, results) = mpsc::channel();
    let mut threads = Vec::new();
    let mut releases = Vec::new();
    for core in 0..2 {
        let participant = binding.take::<Scrapes>(core, 0)?;
        let completed = completed.clone();
        let (release, finish) = oneshot::channel();
        releases.push(release);
        threads.push(thread::spawn(move || {
            Builder::new_current_thread()
                .enable_all()
                .build()
                .expect("instance runtime")
                .block_on(LocalSet::new().run_until(instance(participant, completed, finish)))
        }));
    }
    let mut all = BTreeSet::new();
    for _ in 0..2 {
        for target in results.recv_timeout(Duration::from_secs(30))? {
            assert!(all.insert(target), "one execution per target");
        }
    }
    assert_eq!(all.len(), 4);
    binding.begin_shutdown();
    for release in releases {
        let _ = release.send(());
    }
    for instance in threads {
        instance.join().expect("instance joined")?;
    }
    binding.stop();
    host.join().expect("host joined")?;
    Ok(())
}
