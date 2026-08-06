Write-Host "1. Reverting the deadlock fix in mod.rs to block on sender.reserve().await..." -ForegroundColor Yellow

$filePath = "rust/otap-dataflow/crates/core-nodes/src/exporters/otap_exporter/mod.rs"
$content = [System.IO.File]::ReadAllText($filePath)

# Use here-strings to avoid any escape character issues in PowerShell
$oldBlock = @'
        let mut pending = Some((pdata, message));

        loop {
            let item = pending
                .take()
                .expect("stream enqueue loop must retain the pending batch");
            match sender.try_send(item) {
                Ok(()) => {
                    self.async_metrics
                        .stream_enqueue_duration_ns
                        .record(elapsed_nanos(enqueue_start));
                    return Ok(EnqueueResult::Done);
                }
                Err(tokio::sync::mpsc::error::TrySendError::Full(item)) => {
                    pending = Some(item);
                }
                Err(tokio::sync::mpsc::error::TrySendError::Closed(_item)) => {
                    self.async_metrics
                        .stream_enqueue_duration_ns
                        .record(elapsed_nanos(enqueue_start));
                    return Ok(EnqueueResult::Done);
                }
            }

            tokio::select! {
                permit = sender.reserve() => {
                    match permit {
                        Ok(permit) => {
                            permit.send(pending
                                .take()
                                .expect("stream enqueue reserve must retain the pending batch"));
                            self.async_metrics
                                .stream_enqueue_duration_ns
                                .record(elapsed_nanos(enqueue_start));
                            return Ok(EnqueueResult::Done);
                        }
                        Err(_) => {
                            self.async_metrics
                                .stream_enqueue_duration_ns
                                .record(elapsed_nanos(enqueue_start));
                            return Ok(EnqueueResult::Done);
                        }
                    }
                }
                metrics_update = pdata_metrics_rx.recv() => {
                    if let Some(update) = metrics_update {
                        self.handle_pdata_metrics_update(update, effect_handler).await?;
                    }
                }
                msg = msg_chan.recv_when(false) => {
                    let control_msg = msg.map_err(|e| Error::ExporterError {
                        exporter: effect_handler.exporter_id(),
                        kind: ExporterErrorKind::Other,
                        error: format!("Inbox receive failed: {e}"),
                        source_detail: "".to_owned(),
                    })?;
                    match control_msg {
                        Message::Control(NodeControlMsg::CollectTelemetry { mut metrics_reporter }) => {
                            self.export_latency_window
                                .report_into(&mut self.async_metrics);
                            _ = metrics_reporter.report(&mut self.pdata_metrics);
                            _ = metrics_reporter.report(&mut self.async_metrics);
                        }
                        Message::Control(NodeControlMsg::Shutdown { deadline, reason }) => {
                            if let Some((pdata, _)) = pending.take() {
                                effect_handler
                                    .notify_nack(NackMsg::new("exporter shutting down", pdata))
                                    .await?;
                            }
                            return Ok(EnqueueResult::Shutdown { deadline, reason });
                        }
                        // NACK any pdata that arrives while we are waiting for queue
                        // space — the exporter is shutting down so we won't forward it.
                        Message::PData(data) => {
                            effect_handler
                                .notify_nack(NackMsg::new("exporter shutting down", data))
                                .await?;
                        }
                        _ => {}
                    }
                }
            }
        }
'@

$newBlock = @'
        match sender.reserve().await {
            Ok(permit) => {
                permit.send((pdata, message));
                self.async_metrics
                    .stream_enqueue_duration_ns
                    .record(elapsed_nanos(enqueue_start));
                Ok(EnqueueResult::Done)
            }
            Err(_) => {
                self.async_metrics
                    .stream_enqueue_duration_ns
                    .record(elapsed_nanos(enqueue_start));
                Ok(EnqueueResult::Done)
            }
        }
'@

# Normalize line endings to avoid search/replace mismatch
$oldBlockNormalized = $oldBlock -replace "`r`n", "`n"
$newBlockNormalized = $newBlock -replace "`r`n", "`n"
$contentNormalized = $content -replace "`r`n", "`n"

if ($contentNormalized.Contains($oldBlockNormalized)) {
    $contentNormalized = $contentNormalized.Replace($oldBlockNormalized, $newBlockNormalized)
    # Write back with standard line endings
    [System.IO.File]::WriteAllText($filePath, ($contentNormalized -replace "`n", "`r`n"))
    Write-Host "Successfully replaced the select loop with the blocking reserve!" -ForegroundColor Green
} else {
    Write-Host "Warning: Could not find the expected select! loop in mod.rs. Is the file already reverted?" -ForegroundColor Red
}

Write-Host "`n2. Running deadlock test (Should FAIL/TIMEOUT now)..." -ForegroundColor Yellow
cargo test --manifest-path rust/otap-dataflow/Cargo.toml --package otap-df-core-nodes --lib exporters::otap_exporter::tests::test_otap_exporter_deadlock_on_full_queue_shutdown

Write-Host "`n3. Restoring mod.rs to the correct fixed version..." -ForegroundColor Yellow
git checkout -- $filePath

Write-Host "`n4. Running deadlock test again (Should PASS now)..." -ForegroundColor Yellow
cargo test --manifest-path rust/otap-dataflow/Cargo.toml --package otap-df-core-nodes --lib exporters::otap_exporter::tests::test_otap_exporter_deadlock_on_full_queue_shutdown
