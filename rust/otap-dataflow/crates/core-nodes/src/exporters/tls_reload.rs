// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use std::rc::Rc;

use otel_arrow_dfe_otap::client_tls_reload::ClientTlsProvider;
use tokio::task::{JoinError, JoinHandle};
use tokio_util::sync::CancellationToken;

pub(super) struct LocalTlsReloadTask {
    cancel: CancellationToken,
    pub(super) handle: Option<JoinHandle<()>>,
}

impl LocalTlsReloadTask {
    pub(super) fn start(provider: Rc<ClientTlsProvider>) -> Self {
        let cancel = CancellationToken::new();
        let task_cancel = cancel.clone();
        let handle = tokio::task::spawn_local(async move {
            provider.run(task_cancel.cancelled()).await;
        });
        Self {
            cancel,
            handle: Some(handle),
        }
    }

    pub(super) async fn stop(&mut self) -> Result<(), JoinError> {
        self.cancel.cancel();
        if let Some(handle) = self.handle.take()
            && let Err(error) = handle.await
            && !error.is_cancelled()
        {
            return Err(error);
        }
        Ok(())
    }
}

impl Drop for LocalTlsReloadTask {
    fn drop(&mut self) {
        self.cancel.cancel();
        if let Some(handle) = self.handle.take() {
            handle.abort();
        }
    }
}
