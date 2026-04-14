// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

#![cfg_attr(not(target_os = "linux"), allow(dead_code, unused_imports))]

//! Linux perf-session support for the userevents receiver.

#[cfg(target_os = "linux")]
mod imp {
    #![allow(unsafe_code)]

    use std::collections::HashMap;
    use std::fs::{self, File};
    use std::io;
    use std::mem::size_of;
    use std::os::fd::{AsRawFd, FromRawFd, RawFd};
    use std::path::{Path, PathBuf};
    use std::sync::atomic::{Ordering, fence};
    use std::time::Instant;

    use memmap2::{MmapMut, MmapOptions};
    use nix::fcntl::{FcntlArg, OFlag, fcntl};
    use nix::libc;
    use tokio::io::unix::AsyncFd;

    use super::super::{DrainConfig, SessionConfig, SubscriptionConfig};

    const PERF_TYPE_TRACEPOINT: u32 = 2;
    const PERF_FLAG_FD_CLOEXEC: libc::c_ulong = 1 << 3;
    const PERF_FORMAT_ID: u64 = 1 << 2;
    const PERF_RECORD_LOST: u32 = 2;
    const PERF_RECORD_SAMPLE: u32 = 9;
    const PERF_SAMPLE_IDENTIFIER: u64 = 1 << 16;
    const PERF_SAMPLE_TID: u64 = 1 << 1;
    const PERF_SAMPLE_TIME: u64 = 1 << 2;
    const PERF_SAMPLE_CPU: u64 = 1 << 7;
    const PERF_SAMPLE_RAW: u64 = 1 << 10;
    const PERF_ATTR_FLAG_WATERMARK: u64 = 1 << 14;
    const PERF_EVENT_IOC_ENABLE: libc::c_ulong = ioctl_request_none(b'$', 0);
    const PERF_EVENT_IOC_SET_OUTPUT: libc::c_ulong = ioctl_request_none(b'$', 5);
    const PERF_EVENT_IOC_ID: libc::c_ulong = ioctl_request_read(b'$', 7, size_of::<u64>());
    const MMAP_DATA_HEAD_OFFSET: usize = 1024;
    const MMAP_DATA_TAIL_OFFSET: usize = 1032;
    const MMAP_DATA_OFFSET_OFFSET: usize = 1040;
    const MMAP_DATA_SIZE_OFFSET: usize = 1048;

    /// A raw event drained from the perf ring.
    #[derive(Debug, Clone, PartialEq, Eq)]
    pub(crate) struct RawUsereventsRecord {
        /// Tracepoint name from the tracefs subscription.
        pub tracepoint: String,
        /// Timestamp in Unix epoch nanoseconds.
        pub timestamp_unix_nano: u64,
        /// CPU that emitted the sample.
        pub cpu: u32,
        /// Process identifier.
        pub pid: i32,
        /// Thread identifier.
        pub tid: i32,
        /// Sample id resolved for the tracepoint.
        pub sample_id: u64,
        /// Raw payload bytes after the event payload offset.
        pub payload: Vec<u8>,
        /// Original payload size before encoding.
        pub payload_size: usize,
    }

    #[derive(Debug)]
    pub(crate) struct SessionDrain {
        /// Records drained from the perf ring.
        pub records: Vec<RawUsereventsRecord>,
        /// Number of lost samples reported by the kernel during this drain turn.
        pub lost_samples: u64,
    }

    #[derive(Debug)]
    pub(crate) enum SessionInitError {
        MissingTracepoint(String),
        InvalidTracepoint(String),
        Io(io::Error),
    }

    impl std::fmt::Display for SessionInitError {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            match self {
                Self::MissingTracepoint(name) => {
                    write!(f, "tracepoint `{name}` is not registered")
                }
                Self::InvalidTracepoint(name) => write!(f, "tracepoint `{name}` is invalid"),
                Self::Io(err) => err.fmt(f),
            }
        }
    }

    impl std::error::Error for SessionInitError {}

    impl From<io::Error> for SessionInitError {
        fn from(value: io::Error) -> Self {
            Self::Io(value)
        }
    }

    pub(crate) struct UsereventsSession {
        leader: AsyncFd<File>,
        member_files: Vec<File>,
        mmap: MmapMut,
        tracepoints_by_sample_id: HashMap<u64, TracepointMetadata>,
        monotonic_to_realtime_offset_ns: i128,
        scratch: Vec<u8>,
    }

    impl UsereventsSession {
        pub(crate) fn open(
            subscriptions: &[SubscriptionConfig],
            config: &SessionConfig,
            cpu_id: usize,
        ) -> Result<Self, SessionInitError> {
            if subscriptions.is_empty() {
                return Err(SessionInitError::InvalidTracepoint(
                    "at least one tracepoint subscription is required".to_owned(),
                ));
            }

            let tracefs_root = tracefs_root()?;
            let page_size = page_size();
            let buffer_size = round_up_buffer_size(page_size, config.per_cpu_buffer_size);
            let monotonic_to_realtime_offset_ns = monotonic_to_realtime_offset_ns()?;

            let mut resolved = Vec::with_capacity(subscriptions.len());
            for subscription in subscriptions {
                resolved.push(resolve_tracepoint(&tracefs_root, subscription)?);
            }

            let mut resolved = resolved.into_iter();
            let leader_tracepoint = resolved.next().ok_or_else(|| {
                SessionInitError::InvalidTracepoint(
                    "at least one resolved tracepoint is required".to_owned(),
                )
            })?;

            let leader_file = open_perf_fd(leader_tracepoint.id, cpu_id, config.wakeup_watermark)?;
            set_nonblocking(&leader_file)?;
            let leader_sample_id = read_perf_id(&leader_file)?;
            enable_event(leader_file.as_raw_fd())?;

            let mmap_len = page_size + buffer_size;
            let mmap = unsafe { MmapOptions::new().len(mmap_len).map_mut(&leader_file)? };

            let mut member_files = Vec::with_capacity(subscriptions.len());
            member_files.push(leader_file.try_clone()?);

            let mut tracepoints_by_sample_id = HashMap::with_capacity(subscriptions.len());
            let _ = tracepoints_by_sample_id.insert(
                leader_sample_id,
                TracepointMetadata {
                    tracepoint: leader_tracepoint.tracepoint,
                    payload_offset: leader_tracepoint.payload_offset,
                },
            );

            for tracepoint in resolved {
                let file = open_perf_fd(tracepoint.id, cpu_id, 0)?;
                set_output(file.as_raw_fd(), leader_file.as_raw_fd())?;
                let sample_id = read_perf_id(&file)?;
                enable_event(file.as_raw_fd())?;
                let _ = tracepoints_by_sample_id.insert(
                    sample_id,
                    TracepointMetadata {
                        tracepoint: tracepoint.tracepoint,
                        payload_offset: tracepoint.payload_offset,
                    },
                );
                member_files.push(file);
            }

            Ok(Self {
                leader: AsyncFd::new(leader_file)?,
                member_files,
                mmap,
                tracepoints_by_sample_id,
                monotonic_to_realtime_offset_ns,
                scratch: Vec::new(),
            })
        }

        pub(crate) async fn drain_ready(
            &mut self,
            config: &DrainConfig,
        ) -> io::Result<SessionDrain> {
            loop {
                let mut guard = self.leader.readable().await?;
                let drain = Self::drain_available(
                    &mut self.mmap,
                    &self.tracepoints_by_sample_id,
                    self.monotonic_to_realtime_offset_ns,
                    &mut self.scratch,
                    config,
                )?;
                if drain.records.is_empty() && drain.lost_samples == 0 {
                    guard.clear_ready();
                    continue;
                }
                return Ok(drain);
            }
        }

        pub(crate) fn subscription_count(&self) -> usize {
            self.member_files.len()
        }

        fn drain_available(
            mmap: &mut MmapMut,
            tracepoints_by_sample_id: &HashMap<u64, TracepointMetadata>,
            monotonic_to_realtime_offset_ns: i128,
            scratch: &mut Vec<u8>,
            config: &DrainConfig,
        ) -> io::Result<SessionDrain> {
            let page_size = page_size();
            let header_page = &mmap[..page_size];
            let data_offset = read_u64(header_page, MMAP_DATA_OFFSET_OFFSET)? as usize;
            let data_size = read_u64(header_page, MMAP_DATA_SIZE_OFFSET)? as usize;
            let ring = &mmap[data_offset..data_size + data_offset];

            let head = read_u64(header_page, MMAP_DATA_HEAD_OFFSET)?;
            fence(Ordering::Acquire);
            let mut tail = read_u64(header_page, MMAP_DATA_TAIL_OFFSET)?;
            if head.saturating_sub(tail) > data_size as u64 {
                tail = head.saturating_sub(data_size as u64);
            }

            let started = Instant::now();
            let mut drained_bytes = 0usize;
            let mut records = Vec::new();
            let mut lost_samples = 0u64;

            while tail < head {
                if records.len() >= config.max_records_per_turn
                    || drained_bytes >= config.max_bytes_per_turn
                    || started.elapsed() >= config.max_drain_ns
                {
                    break;
                }

                let mut header_bytes = [0u8; 8];
                copy_from_ring(ring, (tail as usize) & (data_size - 1), &mut header_bytes);
                let record_type =
                    u32::from_ne_bytes(header_bytes[0..4].try_into().unwrap_or([0; 4]));
                let record_size =
                    u16::from_ne_bytes(header_bytes[6..8].try_into().unwrap_or([0; 2])) as usize;
                if record_size == 0 || record_size > (head - tail) as usize {
                    break;
                }

                scratch.resize(record_size, 0);
                copy_from_ring(
                    ring,
                    (tail as usize) & (data_size - 1),
                    scratch.as_mut_slice(),
                );
                tail = tail.saturating_add(record_size as u64);
                drained_bytes = drained_bytes.saturating_add(record_size);

                match record_type {
                    PERF_RECORD_SAMPLE => {
                        if let Some(record) = parse_sample_record(
                            scratch,
                            tracepoints_by_sample_id,
                            monotonic_to_realtime_offset_ns,
                        )? {
                            records.push(record);
                        }
                    }
                    PERF_RECORD_LOST => {
                        lost_samples = lost_samples.saturating_add(parse_lost_record(scratch));
                    }
                    _ => {}
                }
            }

            fence(Ordering::Release);
            write_u64(&mut mmap[..page_size], MMAP_DATA_TAIL_OFFSET, tail)?;

            Ok(SessionDrain {
                records,
                lost_samples,
            })
        }
    }

    #[derive(Debug)]
    struct ResolvedTracepoint {
        tracepoint: String,
        id: u64,
        payload_offset: usize,
    }

    #[derive(Debug)]
    struct TracepointMetadata {
        tracepoint: String,
        payload_offset: usize,
    }

    #[repr(C)]
    #[derive(Clone, Copy, Debug, Default)]
    struct PerfEventAttr {
        attr_type: u32,
        size: u32,
        config: u64,
        sample_period: u64,
        sample_type: u64,
        read_format: u64,
        flags: u64,
        wakeup_events: u32,
        bp_type: u32,
        config1: u64,
        config2: u64,
        branch_sample_type: u64,
        sample_regs_user: u64,
        sample_stack_user: u32,
        clockid: i32,
        sample_regs_intr: u64,
    }

    const fn ioctl_request_none(ty: u8, nr: u8) -> libc::c_ulong {
        ((ty as libc::c_ulong) << 8) | nr as libc::c_ulong
    }

    const fn ioctl_request_read(ty: u8, nr: u8, size: usize) -> libc::c_ulong {
        (2u64 << 30)
            | ((size as libc::c_ulong) << 16)
            | ((ty as libc::c_ulong) << 8)
            | nr as libc::c_ulong
    }

    fn open_perf_fd(
        tracepoint_id: u64,
        cpu_id: usize,
        wakeup_watermark: usize,
    ) -> io::Result<File> {
        let attr = PerfEventAttr {
            attr_type: PERF_TYPE_TRACEPOINT,
            size: size_of::<PerfEventAttr>() as u32,
            config: tracepoint_id,
            sample_period: 1,
            sample_type: PERF_SAMPLE_IDENTIFIER
                | PERF_SAMPLE_TID
                | PERF_SAMPLE_TIME
                | PERF_SAMPLE_CPU
                | PERF_SAMPLE_RAW,
            read_format: PERF_FORMAT_ID,
            flags: PERF_ATTR_FLAG_WATERMARK,
            wakeup_events: wakeup_watermark as u32,
            ..PerfEventAttr::default()
        };

        let opened_fd = unsafe {
            libc::syscall(
                libc::SYS_perf_event_open,
                &attr as *const PerfEventAttr,
                -1,
                cpu_id as i32,
                -1,
                PERF_FLAG_FD_CLOEXEC,
            )
        };
        if opened_fd < 0 {
            return Err(io::Error::last_os_error());
        }
        let fd = RawFd::try_from(opened_fd).map_err(|_| io::Error::last_os_error())?;
        let file = unsafe { File::from_raw_fd(fd) };
        Ok(file)
    }

    fn set_nonblocking(file: &File) -> io::Result<()> {
        let current = fcntl(file, FcntlArg::F_GETFL).map_err(io::Error::other)?;
        let mut flags = OFlag::from_bits_truncate(current);
        flags.insert(OFlag::O_NONBLOCK);
        _ = fcntl(file, FcntlArg::F_SETFL(flags)).map_err(io::Error::other)?;
        Ok(())
    }

    fn set_output(fd: RawFd, leader_fd: RawFd) -> io::Result<()> {
        let request = PERF_EVENT_IOC_SET_OUTPUT;
        let rc = unsafe { libc::ioctl(fd, request, leader_fd) };
        if rc < 0 {
            return Err(io::Error::last_os_error());
        }
        Ok(())
    }

    fn enable_event(fd: RawFd) -> io::Result<()> {
        let request = PERF_EVENT_IOC_ENABLE;
        let rc = unsafe { libc::ioctl(fd, request, 0) };
        if rc < 0 {
            return Err(io::Error::last_os_error());
        }
        Ok(())
    }

    fn read_perf_id(file: &File) -> io::Result<u64> {
        let request = PERF_EVENT_IOC_ID;
        let mut id = 0u64;
        let rc = unsafe { libc::ioctl(file.as_raw_fd(), request, &mut id as *mut u64) };
        if rc < 0 {
            return Err(io::Error::last_os_error());
        }
        Ok(id)
    }

    fn tracefs_root() -> io::Result<PathBuf> {
        let primary = PathBuf::from("/sys/kernel/tracing");
        if primary.join("events").exists() {
            return Ok(primary);
        }

        let fallback = PathBuf::from("/sys/kernel/debug/tracing");
        if fallback.join("events").exists() {
            return Ok(fallback);
        }

        Err(io::Error::new(
            io::ErrorKind::NotFound,
            "tracefs root not found under /sys/kernel/tracing or /sys/kernel/debug/tracing",
        ))
    }

    fn resolve_tracepoint(
        tracefs_root: &Path,
        subscription: &SubscriptionConfig,
    ) -> Result<ResolvedTracepoint, SessionInitError> {
        let (system, event) = subscription
            .tracepoint
            .split_once(':')
            .ok_or_else(|| SessionInitError::InvalidTracepoint(subscription.tracepoint.clone()))?;

        if system != "user_events" {
            return Err(SessionInitError::InvalidTracepoint(
                subscription.tracepoint.clone(),
            ));
        }

        let tracepoint_dir = tracefs_root.join("events").join(system).join(event);
        if !tracepoint_dir.exists() {
            return Err(SessionInitError::MissingTracepoint(
                subscription.tracepoint.clone(),
            ));
        }

        let id = read_tracepoint_metadata(&tracepoint_dir, "id", &subscription.tracepoint)?
            .trim()
            .parse::<u64>()
            .map_err(|_| SessionInitError::InvalidTracepoint(subscription.tracepoint.clone()))?;

        let format = read_tracepoint_metadata(&tracepoint_dir, "format", &subscription.tracepoint)?;
        let parsed_format = parse_tracepoint_format(&format)
            .ok_or_else(|| SessionInitError::InvalidTracepoint(subscription.tracepoint.clone()))?;

        Ok(ResolvedTracepoint {
            tracepoint: subscription.tracepoint.clone(),
            id,
            payload_offset: parsed_format.payload_offset,
        })
    }

    fn read_tracepoint_metadata(
        tracepoint_dir: &Path,
        file_name: &str,
        tracepoint: &str,
    ) -> Result<String, SessionInitError> {
        let path = tracepoint_dir.join(file_name);
        fs::read_to_string(&path).map_err(|error| match error.kind() {
            io::ErrorKind::NotFound => SessionInitError::MissingTracepoint(tracepoint.to_owned()),
            io::ErrorKind::PermissionDenied => SessionInitError::Io(io::Error::new(
                io::ErrorKind::PermissionDenied,
                format!(
                    "tracepoint `{tracepoint}` is registered but `{}` is not readable; run df_engine with elevated privileges or relax tracefs read permissions",
                    path.display()
                ),
            )),
            _ => SessionInitError::Io(error),
        })
    }

    struct ParsedTracepointFormat {
        payload_offset: usize,
    }

    fn parse_tracepoint_format(contents: &str) -> Option<ParsedTracepointFormat> {
        let mut common_fields_end = 0usize;
        let mut eventheader_flags_offset = None;

        for line in contents.lines() {
            let line = line.trim();
            if !line.starts_with("field:") {
                continue;
            }

            let mut field_name = None;
            let mut offset = None;
            let mut size = None;
            for part in line.split(';') {
                let part = part.trim();
                if let Some(rest) = part.strip_prefix("field:") {
                    field_name = rest.split_whitespace().last().map(str::to_owned);
                } else if let Some(rest) = part.strip_prefix("offset:") {
                    offset = rest.trim().parse::<usize>().ok();
                } else if let Some(rest) = part.strip_prefix("size:") {
                    size = rest.trim().parse::<usize>().ok();
                }
            }

            let (field_name, offset, size) = match (field_name, offset, size) {
                (Some(field_name), Some(offset), Some(size)) => (field_name, offset, size),
                _ => continue,
            };

            if field_name.starts_with("common_") {
                common_fields_end = common_fields_end.max(offset.saturating_add(size));
            }
            if field_name == "eventheader_flags" {
                eventheader_flags_offset = Some(offset);
            }
        }

        Some(ParsedTracepointFormat {
            payload_offset: eventheader_flags_offset.unwrap_or(common_fields_end),
        })
    }

    fn parse_sample_record(
        record: &[u8],
        tracepoints_by_sample_id: &HashMap<u64, TracepointMetadata>,
        monotonic_to_realtime_offset_ns: i128,
    ) -> io::Result<Option<RawUsereventsRecord>> {
        if record.len() < 8 + 8 + 8 + 8 + 8 + 4 {
            return Ok(None);
        }

        let mut cursor = 8;
        let sample_id = read_u64_from(record, &mut cursor)?;
        let pid = read_u32_from(record, &mut cursor)? as i32;
        let tid = read_u32_from(record, &mut cursor)? as i32;
        let timestamp_unix_nano = perf_timestamp_to_unix_nano(
            read_u64_from(record, &mut cursor)?,
            monotonic_to_realtime_offset_ns,
        );
        let cpu = read_u32_from(record, &mut cursor)?;
        _ = read_u32_from(record, &mut cursor)?;
        let raw_size = read_u32_from(record, &mut cursor)? as usize;
        if record.len() < cursor.saturating_add(raw_size) {
            return Ok(None);
        }

        let metadata = match tracepoints_by_sample_id.get(&sample_id) {
            Some(metadata) => metadata,
            None => return Ok(None),
        };

        let raw = &record[cursor..cursor + raw_size];
        let payload = if raw.len() > metadata.payload_offset {
            raw[metadata.payload_offset..].to_vec()
        } else {
            raw.to_vec()
        };

        Ok(Some(RawUsereventsRecord {
            tracepoint: metadata.tracepoint.clone(),
            timestamp_unix_nano,
            cpu,
            pid,
            tid,
            sample_id,
            payload_size: payload.len(),
            payload,
        }))
    }

    fn parse_lost_record(record: &[u8]) -> u64 {
        if record.len() < 24 {
            return 1;
        }
        u64::from_ne_bytes(record[16..24].try_into().unwrap_or([0; 8]))
    }

    fn monotonic_to_realtime_offset_ns() -> io::Result<i128> {
        // Perf tracepoint sample timestamps on this path are monotonic-domain.
        // Convert them to wall clock once per session using a paired snapshot.
        let monotonic_before = clock_gettime_ns(libc::CLOCK_MONOTONIC)?;
        let realtime = clock_gettime_ns(libc::CLOCK_REALTIME)?;
        let monotonic_after = clock_gettime_ns(libc::CLOCK_MONOTONIC)?;
        let monotonic_midpoint = monotonic_before + (monotonic_after - monotonic_before) / 2;
        Ok(realtime as i128 - monotonic_midpoint as i128)
    }

    fn clock_gettime_ns(clock_id: libc::clockid_t) -> io::Result<u64> {
        let mut timespec = libc::timespec {
            tv_sec: 0,
            tv_nsec: 0,
        };
        let status = unsafe { libc::clock_gettime(clock_id, &mut timespec) };
        if status != 0 {
            return Err(io::Error::last_os_error());
        }
        if timespec.tv_sec < 0 || timespec.tv_nsec < 0 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "clock_gettime returned a negative timestamp",
            ));
        }
        Ok((timespec.tv_sec as u64)
            .saturating_mul(1_000_000_000)
            .saturating_add(timespec.tv_nsec as u64))
    }

    fn perf_timestamp_to_unix_nano(timestamp: u64, monotonic_to_realtime_offset_ns: i128) -> u64 {
        let unix_timestamp = (timestamp as i128).saturating_add(monotonic_to_realtime_offset_ns);
        if unix_timestamp <= 0 {
            0
        } else if unix_timestamp >= u64::MAX as i128 {
            u64::MAX
        } else {
            unix_timestamp as u64
        }
    }

    fn read_u64(buf: &[u8], offset: usize) -> io::Result<u64> {
        let bytes = buf
            .get(offset..offset + 8)
            .ok_or_else(|| io::Error::new(io::ErrorKind::UnexpectedEof, "short mmap header"))?;
        Ok(u64::from_ne_bytes(bytes.try_into().unwrap_or([0; 8])))
    }

    fn write_u64(buf: &mut [u8], offset: usize, value: u64) -> io::Result<()> {
        let bytes = buf
            .get_mut(offset..offset + 8)
            .ok_or_else(|| io::Error::new(io::ErrorKind::UnexpectedEof, "short mmap header"))?;
        bytes.copy_from_slice(&value.to_ne_bytes());
        Ok(())
    }

    fn read_u64_from(buf: &[u8], cursor: &mut usize) -> io::Result<u64> {
        let next = cursor.saturating_add(8);
        let value = buf
            .get(*cursor..next)
            .ok_or_else(|| io::Error::new(io::ErrorKind::UnexpectedEof, "short sample"))?;
        *cursor = next;
        Ok(u64::from_ne_bytes(value.try_into().unwrap_or([0; 8])))
    }

    fn read_u32_from(buf: &[u8], cursor: &mut usize) -> io::Result<u32> {
        let next = cursor.saturating_add(4);
        let value = buf
            .get(*cursor..next)
            .ok_or_else(|| io::Error::new(io::ErrorKind::UnexpectedEof, "short sample"))?;
        *cursor = next;
        Ok(u32::from_ne_bytes(value.try_into().unwrap_or([0; 4])))
    }

    fn copy_from_ring(ring: &[u8], offset: usize, dst: &mut [u8]) {
        let first = dst.len().min(ring.len().saturating_sub(offset));
        dst[..first].copy_from_slice(&ring[offset..offset + first]);
        if first < dst.len() {
            let second_len = dst.len() - first;
            dst[first..].copy_from_slice(&ring[..second_len]);
        }
    }

    fn round_up_buffer_size(page_size: usize, requested_size: usize) -> usize {
        page_size.max(requested_size).next_power_of_two()
    }

    fn page_size() -> usize {
        let sys_page_size = unsafe { libc::sysconf(libc::_SC_PAGESIZE) };
        if sys_page_size <= 0 {
            4096
        } else {
            sys_page_size as usize
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn parse_tracepoint_format_prefers_eventheader_offset() {
            let format = r#"
name: user_events:Example
ID: 123
format:
	field:unsigned short common_type;	offset:0;	size:2;	signed:0;
	field:unsigned char common_flags;	offset:2;	size:1;	signed:0;
	field:unsigned char common_preempt_count;	offset:3;	size:1;	signed:0;
	field:int common_pid;	offset:4;	size:4;	signed:1;
	field:unsigned char user_reg_index;	offset:8;	size:1;	signed:0;
	field:unsigned char eventheader_flags;	offset:9;	size:1;	signed:0;
            "#;
            let parsed = parse_tracepoint_format(format).expect("parsed format");
            assert_eq!(parsed.payload_offset, 9);
        }

        #[test]
        fn parse_sample_record_extracts_payload_after_offset() {
            let mut record = Vec::new();
            record.extend_from_slice(&PERF_RECORD_SAMPLE.to_ne_bytes());
            record.extend_from_slice(&0u16.to_ne_bytes());
            let raw = [1u8, 2, 3, 4, 5];
            let size = (8 + 8 + 8 + 8 + 8 + 4 + raw.len()) as u16;
            record.extend_from_slice(&size.to_ne_bytes());
            record.extend_from_slice(&7u64.to_ne_bytes());
            record.extend_from_slice(&11u32.to_ne_bytes());
            record.extend_from_slice(&22u32.to_ne_bytes());
            record.extend_from_slice(&33u64.to_ne_bytes());
            record.extend_from_slice(&44u32.to_ne_bytes());
            record.extend_from_slice(&0u32.to_ne_bytes());
            record.extend_from_slice(&(raw.len() as u32).to_ne_bytes());
            record.extend_from_slice(&raw);

            let mut tracepoints = HashMap::new();
            tracepoints.insert(
                7,
                TracepointMetadata {
                    tracepoint: "user_events:Example".to_owned(),
                    payload_offset: 2,
                },
            );

            let parsed = parse_sample_record(&record, &tracepoints, 1000)
                .expect("sample parsed")
                .expect("sample resolved");
            assert_eq!(parsed.payload, vec![3, 4, 5]);
            assert_eq!(parsed.cpu, 44);
            assert_eq!(parsed.pid, 11);
            assert_eq!(parsed.tid, 22);
            assert_eq!(parsed.timestamp_unix_nano, 1033);
        }

        #[test]
        fn perf_timestamp_to_unix_nano_saturates_at_zero() {
            assert_eq!(perf_timestamp_to_unix_nano(33, -100), 0);
        }
    }
}

#[cfg(not(target_os = "linux"))]
mod imp {
    use std::io;

    use super::super::{DrainConfig, SessionConfig, SubscriptionConfig};

    /// A raw event drained from the perf ring.
    #[derive(Debug, Clone, PartialEq, Eq)]
    pub(crate) struct RawUsereventsRecord {
        /// Concrete tracepoint name.
        pub tracepoint: String,
        /// Timestamp in Unix epoch nanoseconds.
        pub timestamp_unix_nano: u64,
        /// CPU that emitted the sample.
        pub cpu: u32,
        /// Process identifier.
        pub pid: i32,
        /// Thread identifier.
        pub tid: i32,
        /// Sample id resolved for the tracepoint.
        pub sample_id: u64,
        /// Raw payload bytes.
        pub payload: Vec<u8>,
        /// Original payload size.
        pub payload_size: usize,
    }

    #[derive(Debug)]
    pub(crate) struct SessionDrain {
        /// Records drained from the perf ring.
        pub records: Vec<RawUsereventsRecord>,
        /// Number of lost samples reported by the kernel during this drain turn.
        pub lost_samples: u64,
    }

    #[derive(Debug)]
    pub(crate) enum SessionInitError {
        Unsupported,
    }

    impl std::fmt::Display for SessionInitError {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            match self {
                Self::Unsupported => write!(f, "userevents sessions are supported only on Linux"),
            }
        }
    }

    impl std::error::Error for SessionInitError {}

    pub(crate) struct UsereventsSession;

    impl UsereventsSession {
        pub(crate) fn open(
            _subscriptions: &[SubscriptionConfig],
            _config: &SessionConfig,
            _cpu_id: usize,
        ) -> Result<Self, SessionInitError> {
            Err(SessionInitError::Unsupported)
        }

        pub(crate) async fn drain_ready(
            &mut self,
            _config: &DrainConfig,
        ) -> io::Result<SessionDrain> {
            Ok(SessionDrain {
                records: Vec::new(),
                lost_samples: 0,
            })
        }

        pub(crate) fn subscription_count(&self) -> usize {
            0
        }
    }
}

pub(super) use imp::{RawUsereventsRecord, SessionInitError, UsereventsSession};
