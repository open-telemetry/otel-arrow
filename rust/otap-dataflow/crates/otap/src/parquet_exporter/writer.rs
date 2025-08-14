// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use std::collections::HashMap;
use std::collections::hash_map::Entry;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

use arrow::array::RecordBatch;
use arrow::datatypes::SchemaRef;
use futures::TryStreamExt;
use futures::stream::FuturesUnordered;
use object_store::ObjectStore;
use otel_arrow_rust::otap::{OtapArrowRecords, child_payload_types};
use otel_arrow_rust::proto::opentelemetry::arrow::v1::ArrowPayloadType;
use parquet::arrow::AsyncArrowWriter;
use parquet::arrow::async_writer::ParquetObjectWriter;
use parquet::errors::ParquetError;
use parquet::file::properties::WriterProperties;

use super::config::WriterOptions;
use super::partition::PartitionAttribute;

pub struct WriteBatch<'a> {
    pub batch_id: i64,
    pub otap_batch: &'a OtapArrowRecords,
    pub partition_attributes: Option<&'a [PartitionAttribute]>,
}

impl<'a> WriteBatch<'a> {
    pub fn new(
        batch_id: i64,
        otap_batch: &'a OtapArrowRecords,
        partition_attributes: Option<&'a [PartitionAttribute]>,
    ) -> Self {
        Self {
            batch_id,
            otap_batch,
            partition_attributes,
        }
    }
}

/// `WriterManager` controls the parquet object writers for each table associated with some
/// payload type of the OTAP data. It is capable of writing partitioned files, and
/// controlling the approximate number of rows in each file.
pub struct WriterManager {
    object_store: Arc<dyn ObjectStore>,
    options: WriterOptions,

    // the current filename used for each path prefix. Prefixes in this case are based
    // on the payload type, and the partition attributes. There can be multiple files
    // written within the same prefix as files are flushed.
    curr_writer_for_prefix: HashMap<String, FileWriter>,

    // state for unflushed batches. used to track which OTAP batches have unflushed writes
    // for given types of payloads. This is used to determine whether a file should be flushed
    // based on whether some child payload type has unflushed writes.
    unflushed_batches_state: UnflushedBatchState,

    // These are files that have had all their batches written, but the parquet writer
    // has not yet been closed. We hold these files until all the child rows for some
    // batch have been flushed. This is to ensure the parent record does not become
    // visible until all its children are flushed.
    pending_file_flushes: Vec<FileWriter>,
}

impl WriterManager {
    pub fn new(object_store: Arc<dyn ObjectStore>, options: Option<WriterOptions>) -> Self {
        Self {
            object_store,
            options: options.unwrap_or_default(),
            curr_writer_for_prefix: HashMap::new(),
            unflushed_batches_state: UnflushedBatchState::new(),
            pending_file_flushes: Vec::new(),
        }
    }

    pub async fn write(&mut self, writes: &[WriteBatch<'_>]) -> Result<(), ParquetError> {
        for write in writes {
            // schedule the writes for each payload type for this signal
            for payload_type in write.otap_batch.allowed_payload_types() {
                if let Some(record_batch) = write.otap_batch.get(*payload_type) {
                    self.schedule_write_batch(
                        write.batch_id,
                        *payload_type,
                        record_batch,
                        write.partition_attributes,
                    )
                }
            }
        }

        // write the scheduled batches to the files
        self.write_scheduled().await?;

        // if we can determine after the write process that any files should be flushed
        // (e.g. if they've exceeded max size), we'll try to flush them immediately
        //
        // Note: the files might not actually get flushed if they have child rows that
        // aren't scheduled to be flushed. In this case, we won't continue appending to
        // these files, but we won't flush them until after the children are flushed.
        self.schedule_flushes();
        self.attempt_flush_scheduled().await?;

        Ok(())
    }

    /// Write all the scheduled writes to the files concurrently
    async fn write_scheduled(&mut self) -> Result<(), ParquetError> {
        _ = self
            .curr_writer_for_prefix
            .values_mut()
            .map(|fw| fw.write_scheduled())
            .collect::<FuturesUnordered<_>>()
            .try_collect::<Vec<_>>()
            .await?;

        Ok(())
    }

    fn schedule_write_batch(
        &mut self,
        batch_id: i64,
        payload_type: ArrowPayloadType,
        record_batch: &RecordBatch,
        partition_attributes: Option<&[PartitionAttribute]>,
    ) {
        let path_prefix = format!(
            "{}/{}",
            payload_type.as_str_name().to_lowercase(),
            compute_partition_path_prefix(partition_attributes),
        );

        // get the current writer for the path, or create a new one
        let mut new_file = false;
        let file_writer = match self.curr_writer_for_prefix.entry(path_prefix) {
            Entry::Occupied(e) => e.into_mut(),
            Entry::Vacant(e) => {
                new_file = true;
                let full_path = format!("{}/{}", e.key(), generate_filename());
                e.insert(FileWriter::new(
                    batch_id,
                    payload_type,
                    new_parquet_arrow_writer(
                        self.object_store.clone(),
                        record_batch.schema(),
                        full_path,
                    ),
                ))
            }
        };

        file_writer.schedule_write(record_batch);

        // track that we have written some unflushed data for this batch_id and payload_type
        if new_file {
            self.unflushed_batches_state
                .incr_unflushed_write(batch_id, payload_type);
        }
    }

    pub fn schedule_flushes(&mut self) {
        // collect the keys of writers that should flush
        let keys_to_flush: Vec<String> = self
            .curr_writer_for_prefix
            .iter()
            .filter_map(|(key, fw)| {
                if self.should_flush(fw) {
                    Some(key.clone())
                } else {
                    None
                }
            })
            .collect();

        // remove them from the map and move into scheduled_flushes
        for key in keys_to_flush {
            if let Some(writer) = self.curr_writer_for_prefix.remove(&key) {
                self.pending_file_flushes.push(writer);
            }
        }
    }

    fn should_flush(&self, file_writer: &FileWriter) -> bool {
        if let Some(target_rows_per_file) = self.options.target_rows_per_file {
            file_writer.rows_written >= target_rows_per_file
        } else {
            false // If no target rows per file is set, don't flush automatically
        }
    }

    /// This method attempts to flush all the scheduled writers. It tries to flush child
    /// payload types before parent records. The motivation is to ensure is to ensure that
    /// parent records do not become visible before their child payloads. E.g. we don't want
    /// Logs to become visible before LogAttrs.
    ///
    /// This isn't guaranteed to flush everything that is scheduled, because there may be
    /// cases where a parent record has children files manager by writers that are not scheduled
    /// to be flushed.
    ///
    /// To force everything to flush, you can call `flush_all` method, which will schedule
    /// all the current writers for flushing and then flush them all.
    async fn attempt_flush_scheduled(&mut self) -> Result<(), ParquetError> {
        let mut flushable = Vec::new();
        let mut requeue = Vec::new();

        loop {
            for file_writer in self.pending_file_flushes.drain(..) {
                if self
                    .unflushed_batches_state
                    .has_unflushed_child(file_writer.first_batch_id, file_writer.payload_type)
                {
                    requeue.push(file_writer);
                } else {
                    flushable.push(file_writer);
                }
            }

            self.pending_file_flushes.append(&mut requeue);
            if flushable.is_empty() {
                break;
            }

            for ctx in &flushable {
                self.unflushed_batches_state
                    .decr_unflushed_write(ctx.first_batch_id, ctx.payload_type);
            }

            _ = flushable
                .drain(..)
                .map(|fw| fw.writer.close())
                .collect::<FuturesUnordered<_>>()
                .try_collect::<Vec<_>>()
                .await?;
        }

        Ok(())
    }

    /// This method flushes all the current writers, ensuring that all files are closed and
    /// all data is written to the object store
    pub async fn flush_all(&mut self) -> Result<(), ParquetError> {
        for (_, writer) in self.curr_writer_for_prefix.drain() {
            self.pending_file_flushes.push(writer);
        }
        self.attempt_flush_scheduled().await
    }
}

/// generates a filename of the form: `part-<millis since epoch>-<random-uuid>.parquet`
fn generate_filename() -> String {
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("expect now to be after unix epoch")
        .as_millis();
    format!("part-{}-{}.parquet", timestamp, uuid::Uuid::new_v4())
}

/// Computes the partition path prefix based on the provided partition attributes.
/// This does hive-style partitioning, where the path segments are formatted as `key=value`.
fn compute_partition_path_prefix(partition_attributes: Option<&[PartitionAttribute]>) -> String {
    match partition_attributes {
        Some(attributes) => {
            let mut path_segments = Vec::with_capacity(attributes.len());
            for attr in attributes {
                path_segments.push(format!("{}={}", attr.key, attr.value));
            }
            path_segments.join("/")
        }
        None => "".to_string(),
    }
}

/// Creates a new Parquet Arrow writer for the given record batch's schema and full path.
fn new_parquet_arrow_writer(
    object_store: Arc<dyn ObjectStore>,
    schema: SchemaRef,
    full_path: String,
) -> AsyncArrowWriter<ParquetObjectWriter> {
    let object_writer = ParquetObjectWriter::new(object_store.clone(), full_path.clone().into());
    AsyncArrowWriter::try_new(object_writer, schema, Some(WriterProperties::default()))
        .expect("Failed to create AsyncArrowWriter")
}

struct FileWriter {
    first_batch_id: i64,
    payload_type: ArrowPayloadType,
    writer: AsyncArrowWriter<ParquetObjectWriter>,
    rows_written: usize,

    scheduled_batches: Vec<RecordBatch>,
}

impl FileWriter {
    fn new(
        batch_id: i64,
        payload_type: ArrowPayloadType,
        writer: AsyncArrowWriter<ParquetObjectWriter>,
    ) -> Self {
        Self {
            first_batch_id: batch_id,
            payload_type,
            writer,
            rows_written: 0,
            scheduled_batches: Vec::new(),
        }
    }

    fn schedule_write(&mut self, record_batch: &RecordBatch) {
        self.scheduled_batches.push(record_batch.clone());
    }

    async fn write_scheduled(&mut self) -> Result<(), ParquetError> {
        let drained_batches: Vec<_> = self.scheduled_batches.drain(..).collect();
        for batch in drained_batches {
            self.write(batch).await?;
        }

        Ok(())
    }

    async fn write(&mut self, record_batch: RecordBatch) -> Result<(), ParquetError> {
        self.writer.write(&record_batch).await?;
        self.rows_written += record_batch.num_rows();

        Ok(())
    }
}

struct UnflushedBatchState {
    // map keyed by (batch_id, payload_type) to values which are the number of files
    // currently being written that have not been flushed
    unflushed_writes: HashMap<(i64, ArrowPayloadType), usize>,
}

impl UnflushedBatchState {
    fn new() -> Self {
        Self {
            unflushed_writes: HashMap::new(),
        }
    }

    fn incr_unflushed_write(&mut self, batch_id: i64, payload_type: ArrowPayloadType) {
        let key = (batch_id, payload_type);
        let count = self.unflushed_writes.entry(key).or_insert(0);
        *count += 1;
    }

    fn decr_unflushed_write(&mut self, batch_id: i64, payload_type: ArrowPayloadType) {
        // Remove the unflushed write for the given batch_id and payload_type
        let key = (batch_id, payload_type);
        if let Some(count) = self.unflushed_writes.get_mut(&key) {
            if *count > 0 {
                *count -= 1;
            }
            if *count == 0 {
                _ = self.unflushed_writes.remove(&key);
            }
        }
    }

    fn has_unflushed_child(&self, batch_id: i64, payload_type: ArrowPayloadType) -> bool {
        child_payload_types(payload_type)
            .iter()
            .any(|child_payload_type| self.has_unflushed_write(batch_id, *child_payload_type))
    }

    fn has_unflushed_write(&self, batch_id: i64, payload_type: ArrowPayloadType) -> bool {
        self.unflushed_writes
            .get(&(batch_id, payload_type))
            .is_some_and(|&count| count > 0)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    use arrow::compute::concat_batches;
    use futures::StreamExt;
    use object_store::local::LocalFileSystem;
    use otel_arrow_rust::otap::from_record_messages;
    use otel_arrow_rust::{Consumer, proto::opentelemetry::arrow::v1::BatchArrowRecords};
    use parquet::arrow::ParquetRecordBatchStreamBuilder;
    use tokio::fs::File;

    use crate::fixtures::{SimpleDataGenOptions, create_simple_logs_arrow_record_batches};
    use crate::parquet_exporter::partition::PartitionAttributeValue;

    fn to_logs_record_batch(mut bar: BatchArrowRecords) -> OtapArrowRecords {
        let mut consumer = Consumer::default();
        let record_messages = consumer.consume_bar(&mut bar).unwrap();
        OtapArrowRecords::Logs(from_record_messages(record_messages))
    }

    #[tokio::test]
    async fn test_simple_single_batch_write_all_logs() {
        let temp_dir = tempfile::tempdir().unwrap();
        let path = temp_dir.path();
        let object_store = Arc::new(LocalFileSystem::new_with_prefix(path).unwrap());
        let mut writer = WriterManager::new(object_store, None);

        // write some batch:
        let otap_batch = to_logs_record_batch(create_simple_logs_arrow_record_batches(
            SimpleDataGenOptions::default(),
        ));
        writer
            .write(&[WriteBatch::new(0, &otap_batch, None)])
            .await
            .unwrap();

        // check that the files aren't flushed
        let mut files = Vec::new();
        let mut read_dir_stream = tokio::fs::read_dir(path).await.unwrap();
        while let Some(entry) = read_dir_stream.next_entry().await.unwrap() {
            files.push(entry)
        }

        assert!(files.is_empty());

        // flush the files
        writer.flush_all().await.unwrap();

        // check that we've written the file for each batch and it contains the correct content
        for payload_type in [
            ArrowPayloadType::Logs,
            ArrowPayloadType::LogAttrs,
            ArrowPayloadType::ResourceAttrs,
            ArrowPayloadType::ScopeAttrs,
        ] {
            let table_name = payload_type.as_str_name().to_lowercase();
            let mut files = Vec::new();
            let mut read_dir_stream =
                tokio::fs::read_dir(format!("{}/{}", path.to_string_lossy(), table_name))
                    .await
                    .unwrap();
            while let Some(entry) = read_dir_stream.next_entry().await.unwrap() {
                files.push(entry.path().to_string_lossy().to_string())
            }

            // we should have written one file
            assert_eq!(files.len(), 1);

            // read the file and ensure it's the equivalent data from the original batch
            let original_record_batch = otap_batch.get(payload_type).unwrap();
            let file = File::open(files[0].clone()).await.unwrap();
            let reader_builder = ParquetRecordBatchStreamBuilder::new(file).await.unwrap();
            let mut reader = reader_builder.build().unwrap();
            let read_batch = reader.next().await.unwrap().unwrap();
            assert_eq!(&read_batch, original_record_batch);

            // assert there's no extra data there
            assert!(reader.next().await.is_none())
        }
    }

    #[tokio::test]
    async fn test_simple_multi_batch_write_all_logs() {
        let temp_dir = tempfile::tempdir().unwrap();
        let path = temp_dir.path();
        let object_store = Arc::new(LocalFileSystem::new_with_prefix(path).unwrap());
        let mut writer = WriterManager::new(object_store, None);

        let batch1 = to_logs_record_batch(create_simple_logs_arrow_record_batches(
            SimpleDataGenOptions {
                id_offset: 0,
                ..Default::default()
            },
        ));

        let batch2 = to_logs_record_batch(create_simple_logs_arrow_record_batches(
            SimpleDataGenOptions {
                id_offset: 1,
                ..Default::default()
            },
        ));

        writer
            .write(&[
                WriteBatch::new(0, &batch1, None),
                WriteBatch::new(1, &batch2, None),
            ])
            .await
            .unwrap();

        writer.flush_all().await.unwrap();

        for payload_type in [
            ArrowPayloadType::Logs,
            ArrowPayloadType::LogAttrs,
            ArrowPayloadType::ResourceAttrs,
            ArrowPayloadType::ScopeAttrs,
        ] {
            let table_name = payload_type.as_str_name().to_lowercase();
            let mut files = Vec::new();
            let mut read_dir_stream =
                tokio::fs::read_dir(format!("{}/{}", path.to_string_lossy(), table_name))
                    .await
                    .unwrap();
            while let Some(entry) = read_dir_stream.next_entry().await.unwrap() {
                files.push(entry.path().to_string_lossy().to_string())
            }

            // we should have written one file
            assert_eq!(files.len(), 1);

            // read the file and ensure it's the equivalent data from the original batch
            let original_batch1 = batch1.get(payload_type).unwrap();
            let original_batch2 = batch2.get(payload_type).unwrap();
            let expected_batch = concat_batches(
                original_batch1.schema_ref(),
                vec![original_batch1, original_batch2],
            )
            .unwrap();
            let file = File::open(files[0].clone()).await.unwrap();
            let builder = ParquetRecordBatchStreamBuilder::new(file).await.unwrap();
            let mut reader = builder.build().unwrap();
            let read_batch = reader.next().await.unwrap().unwrap();
            assert_eq!(&read_batch, &expected_batch);

            // assert there's no extra data there
            assert!(reader.next().await.is_none())
        }
    }

    #[tokio::test]
    async fn test_partition_write_all_logs() {
        let temp_dir = tempfile::tempdir().unwrap();
        let path = temp_dir.path();
        let object_store = Arc::new(LocalFileSystem::new_with_prefix(path).unwrap());
        let mut writer = WriterManager::new(object_store, None);

        let partition1_batch = to_logs_record_batch(create_simple_logs_arrow_record_batches(
            SimpleDataGenOptions {
                id_offset: 0,
                ..Default::default()
            },
        ));

        let partition1_attrs = vec![
            PartitionAttribute {
                key: "key1".to_string(),
                value: PartitionAttributeValue::String("valA".to_string()),
            },
            PartitionAttribute {
                key: "key2".to_string(),
                value: PartitionAttributeValue::String("1".to_string()),
            },
        ];

        let partition2_batch = to_logs_record_batch(create_simple_logs_arrow_record_batches(
            SimpleDataGenOptions {
                id_offset: 1,
                ..Default::default()
            },
        ));

        let partition2_attrs = vec![
            PartitionAttribute {
                key: "key1".to_string(),
                value: PartitionAttributeValue::String("valB".to_string()),
            },
            PartitionAttribute {
                key: "key2".to_string(),
                value: PartitionAttributeValue::String("1".to_string()),
            },
        ];

        writer
            .write(&[
                WriteBatch::new(0, &partition1_batch, Some(partition1_attrs.as_slice())),
                WriteBatch::new(1, &partition2_batch, Some(partition2_attrs.as_slice())),
            ])
            .await
            .unwrap();

        // write all the files
        writer.flush_all().await.unwrap();

        let test_cases = vec![
            // original_batch, expected_partition_prefix
            (partition1_batch, "key1=valA/key2=1"),
            (partition2_batch, "key1=valB/key2=1"),
        ];

        for test_case in test_cases {
            let otap_batch = test_case.0;
            let partition_prefix = test_case.1;

            // check that we've written the file for each batch and it contains the correct content
            for payload_type in [
                ArrowPayloadType::Logs,
                ArrowPayloadType::LogAttrs,
                ArrowPayloadType::ResourceAttrs,
                ArrowPayloadType::ScopeAttrs,
            ] {
                let table_name = payload_type.as_str_name().to_lowercase();
                let dir = format!(
                    "{}/{}/{}",
                    path.to_string_lossy(),
                    table_name,
                    partition_prefix,
                );
                let mut files = Vec::new();
                let mut read_dir_stream = tokio::fs::read_dir(dir).await.unwrap();
                while let Some(entry) = read_dir_stream.next_entry().await.unwrap() {
                    files.push(entry.path().to_string_lossy().to_string());
                }

                // we should have written one file
                assert_eq!(files.len(), 1);

                // read the file and ensure it's the equivalent data from the original batch
                let original_record_batch = otap_batch.get(payload_type).unwrap();
                let file = File::open(files[0].clone()).await.unwrap();
                let builder = ParquetRecordBatchStreamBuilder::new(file).await.unwrap();
                let mut reader = builder.build().unwrap();
                let read_batch = reader.next().await.unwrap().unwrap();
                assert_eq!(&read_batch, original_record_batch);

                // assert there's no extra data there
                assert!(reader.next().await.is_none())
            }
        }
    }

    #[tokio::test]
    async fn test_auto_flushes_when_max_rows_exceeded() {
        let temp_dir = tempfile::tempdir().unwrap();
        let path = temp_dir.path();
        let object_store = Arc::new(LocalFileSystem::new_with_prefix(path).unwrap());
        let mut writer = WriterManager::new(
            object_store,
            Some(WriterOptions {
                target_rows_per_file: Some(2),
            }),
        );

        let batch1 = to_logs_record_batch(create_simple_logs_arrow_record_batches(
            SimpleDataGenOptions {
                id_offset: 0,
                ..Default::default()
            },
        ));

        let batch2 = to_logs_record_batch(create_simple_logs_arrow_record_batches(
            SimpleDataGenOptions {
                id_offset: 1,
                ..Default::default()
            },
        ));

        writer
            .write(&[
                WriteBatch::new(0, &batch1, None),
                WriteBatch::new(1, &batch2, None),
            ])
            .await
            .unwrap();

        for payload_type in [
            ArrowPayloadType::Logs,
            ArrowPayloadType::LogAttrs,
            ArrowPayloadType::ResourceAttrs,
            ArrowPayloadType::ScopeAttrs,
        ] {
            let table_name = payload_type.as_str_name().to_lowercase();
            let mut files = Vec::new();
            let mut read_dir_stream =
                tokio::fs::read_dir(format!("{}/{}", path.to_string_lossy(), table_name))
                    .await
                    .unwrap();
            while let Some(entry) = read_dir_stream.next_entry().await.unwrap() {
                files.push(entry.path().to_string_lossy().to_string())
            }

            // we should have written one file
            assert_eq!(files.len(), 1);

            // read the file and ensure it's the equivalent data from the original batch
            let original_batch1 = batch1.get(payload_type).unwrap();
            let original_batch2 = batch2.get(payload_type).unwrap();
            let expected_batch = concat_batches(
                original_batch1.schema_ref(),
                vec![original_batch1, original_batch2],
            )
            .unwrap();
            let file = File::open(files[0].clone()).await.unwrap();
            let builder = ParquetRecordBatchStreamBuilder::new(file).await.unwrap();
            let mut reader = builder.build().unwrap();
            let read_batch = reader.next().await.unwrap().unwrap();
            assert_eq!(&read_batch, &expected_batch);

            // assert there's no extra data there
            assert!(reader.next().await.is_none())
        }
    }

    #[tokio::test]
    async fn test_doesnt_autoflush_parent_batch_if_children_not_flushed() {
        let temp_dir = tempfile::tempdir().unwrap();
        let path = temp_dir.path();
        let object_store = Arc::new(LocalFileSystem::new_with_prefix(path).unwrap());
        let mut writer = WriterManager::new(
            object_store,
            Some(WriterOptions {
                target_rows_per_file: Some(2),
            }),
        );

        let batch1 = to_logs_record_batch(create_simple_logs_arrow_record_batches(
            SimpleDataGenOptions {
                id_offset: 0,
                ..Default::default()
            },
        ));

        let batch2 = to_logs_record_batch(create_simple_logs_arrow_record_batches(
            SimpleDataGenOptions {
                id_offset: 1,
                with_main_record_attrs: false,
                ..Default::default()
            },
        ));
        writer
            .write(&[
                WriteBatch::new(0, &batch1, None),
                WriteBatch::new(1, &batch2, None),
            ])
            .await
            .unwrap();

        // at this point resource & scope attributes should have flushed, but the
        // log attributes won't have (because there's only one buffered log record).
        // Also the logs won't have flushed, because there's one batch whose child
        // (log_attrs) hasn't flushed

        for payload_type in [ArrowPayloadType::Logs, ArrowPayloadType::LogAttrs] {
            let table_name = payload_type.as_str_name().to_lowercase();
            let table_exists =
                tokio::fs::try_exists(format!("{}/{}", path.to_string_lossy(), table_name))
                    .await
                    .unwrap();
            assert!(!table_exists);
        }

        for payload_type in [
            ArrowPayloadType::ResourceAttrs,
            ArrowPayloadType::ScopeAttrs,
        ] {
            let table_name = payload_type.as_str_name().to_lowercase();
            let mut files = Vec::new();
            let mut read_dir_stream =
                tokio::fs::read_dir(format!("{}/{}", path.to_string_lossy(), table_name))
                    .await
                    .unwrap();
            while let Some(entry) = read_dir_stream.next_entry().await.unwrap() {
                files.push(entry.path().to_string_lossy().to_string())
            }

            // we should have written one file
            assert_eq!(files.len(), 1);

            // read the file and ensure it's the equivalent data from the original batch
            let original_batch1 = batch1.get(payload_type).unwrap();
            let original_batch2 = batch2.get(payload_type).unwrap();
            let expected_batch = concat_batches(
                original_batch1.schema_ref(),
                vec![original_batch1, original_batch2],
            )
            .unwrap();
            let file = File::open(files[0].clone()).await.unwrap();
            let builder = ParquetRecordBatchStreamBuilder::new(file).await.unwrap();
            let mut reader = builder.build().unwrap();
            let read_batch = reader.next().await.unwrap().unwrap();
            assert_eq!(&read_batch, &expected_batch);

            // assert there's no extra data there
            assert!(reader.next().await.is_none())
        }
    }
}
