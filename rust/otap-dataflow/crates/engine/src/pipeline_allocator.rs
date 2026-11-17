// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Experimental Linux x86-64 Rust heap for the DFE runtime.
//!
//! Small blocks and optional medium-sized blocks use worker-owned spans.
//! Only the owner
//! touches a span's local state. Remote frees publish into a separate list,
//! which the owner consumes in batches. A retiring owner hands live spans to
//! the remote-free path, where the last outstanding block releases the mapping.

#![allow(unsafe_code)]

use nix::libc;
use std::alloc::{GlobalAlloc, Layout};
use std::cell::UnsafeCell;
use std::marker::PhantomData;
use std::ptr::{self, null_mut};
use std::rc::Rc;
use std::sync::atomic::{
    AtomicBool, AtomicUsize, Ordering::AcqRel, Ordering::Acquire, Ordering::Relaxed,
    Ordering::Release,
};
use std::time::{Duration, Instant};

const PAGE: usize = 4096;
const SPAN_BYTES: usize = 64 * 1024;
const SLAB_REGION_BYTES: usize = 2 * 1024 * 1024;
// Bit 0 belongs to the region header. Keep it independent of the slab pages
// so an entire relinquished slab can be discarded.
const SLAB_REGION_FREE: u32 = !1;
const _: () = assert!(SLAB_REGION_BYTES / SPAN_BYTES == u32::BITS as usize);
const CLASS_SIZES: [usize; 36] = [
    16, 32, 48, 64, 80, 96, 112, 128, 160, 192, 224, 256, 320, 384, 448, 512, 640, 768, 896, 1024,
    1280, 1536, 1792, 2048, 2560, 3072, 3584, 4096, 5120, 6144, 7168, 8192, 10240, 12288, 14336,
    16384,
];
const NUM_CLASSES: usize = CLASS_SIZES.len();
const MAX_SMALL: usize = CLASS_SIZES[NUM_CLASSES - 1];
const CLASS_LOOKUP: [u8; MAX_SMALL / 16] = {
    let mut classes = [0; MAX_SMALL / 16];
    let mut size = 16;
    let mut class = 0;
    while size <= MAX_SMALL {
        if CLASS_SIZES[class] < size {
            class += 1;
        }
        classes[size / 16 - 1] = class as u8;
        size += 16;
    }
    classes
};
const RECLAIM_SLAB_STRIDE: usize = 8 * 1024;
const RECLAIM_SLAB_CLASS: usize = CLASS_LOOKUP[RECLAIM_SLAB_STRIDE / 16 - 1] as usize;
#[cfg(test)]
const RECLAIM_SLAB_SLOTS: usize = (SPAN_BYTES - RECLAIM_SLAB_STRIDE) / RECLAIM_SLAB_STRIDE;
const PROBE_SPANS: usize = 8;
const MAP_CACHE_SLOTS: usize = 4;
const MIB: usize = 1024 * 1024;
const MAP_CACHE_BYTES: usize = 2 * MIB;
const MAX_MAP_CACHE_MIB: usize = 8;
const BIN_MIN_SHIFT: u32 = 14;
const BIN_MAX_SHIFT: u32 = 22;
const BIN_COUNT: usize = 4 * (BIN_MAX_SHIFT - BIN_MIN_SHIFT) as usize + 1;
const BINNED: usize = 1;
const RESERVE_HOT_BINS: usize = 1 << 1;
const GRADUAL_TRIM: usize = 1 << 2;
const MEDIUM_EXTENTS: usize = 1 << 3;
const EXTENT_BITMAP: usize = 1 << 4;
const EXTENT_PURGE: usize = 1 << 5;
const UPPER_EXTENT: usize = 1 << 6;
const SLAB_REGIONS: usize = 1 << 7;
const EXTENT_REUSE: usize = 1 << 8;
const EXTENT_REUSE_RESIDENT: usize = 1 << 9;
const EXTENT_POOL_MIB_SHIFT: u32 = 10;
const EXTENT_POOL_MIB_MASK: usize = 0b111 << EXTENT_POOL_MIB_SHIFT;
const FULLEST_EXTENTS: usize = 1 << 14;
const NODE_ACTIVITY: usize = 1 << 13;
const RECLAIM_SLAB_PAGES: usize = 1 << 25;
const CACHE_FLAGS: usize = BINNED
    | RECLAIM_SLAB_PAGES
    | RESERVE_HOT_BINS
    | GRADUAL_TRIM
    | MEDIUM_EXTENTS
    | EXTENT_BITMAP
    | EXTENT_PURGE
    | UPPER_EXTENT
    | SLAB_REGIONS
    | EXTENT_REUSE
    | EXTENT_REUSE_RESIDENT
    | EXTENT_POOL_MIB_MASK
    | FULLEST_EXTENTS
    | NODE_ACTIVITY;
const EXTENT_BYTES: usize = 512 * 1024;
const DEFAULT_EXTENT_POOL_MIB: usize = 2;
const MAX_EXTENT_POOL_MIB: usize = 4;
const MAX_EXTENT_POOL_SLOTS: usize = MAX_EXTENT_POOL_MIB * MIB / EXTENT_BYTES;
const EXTENT_PURGE_DELAY: Duration = Duration::from_secs(1);
const EXTENT_MAX_BIN_SHIFT: u32 = 16;
const EXTENT_MAX_CAPACITY: usize = 1 << EXTENT_MAX_BIN_SHIFT;
const EXTENT_BINS: usize = 4 * (EXTENT_MAX_BIN_SHIFT - BIN_MIN_SHIFT) as usize + 1;
// Mixed OTAP concentrates the remaining dedicated misses in this one bin.
const UPPER_EXTENT_FLOOR: usize = 128 * 1024;
const UPPER_EXTENT_CAPACITY: usize = 160 * 1024;
const EXTENT_20K_CLASS: usize = NUM_CLASSES + 1;
const EXTENT_20K_STRIDE: usize = bin_capacity(EXTENT_20K_CLASS - NUM_CLASSES);
const UPPER_EXTENT_CLASS: usize = NUM_CLASSES + EXTENT_BINS;
const HEAP_CLASSES: usize = UPPER_EXTENT_CLASS + 1;
// The saturation runs put nearly all mapping misses in this size range.
const RESERVABLE_BIN_MAX: usize = 64 * 1024;
const RESERVED_PER_BIN: u16 = 4;
const IDLE_KEEP_MAX: usize = 512 * 1024;
const IDLE_KEEP_TIME: Duration = Duration::from_secs(2);

// Heap flags and the extent pool limit occupy bits outside the dedicated
// mapping cache's byte budget.
static LARGE_CACHE_CONFIG: AtomicUsize = AtomicUsize::new(MAP_CACHE_BYTES | BINNED);

static NEXT_OWNER: AtomicUsize = AtomicUsize::new(1);
static MAPPED: AtomicUsize = AtomicUsize::new(0);
static PEAK_MAPPED: AtomicUsize = AtomicUsize::new(0);
static RETIRED_MAPPED: AtomicUsize = AtomicUsize::new(0);

#[derive(Clone, Copy, PartialEq, Eq)]
enum ExtentMode {
    Disabled,
    Linked,
    Bitmap,
    Purge,
}

impl ExtentMode {
    fn from_config(config: usize) -> Self {
        if config & MEDIUM_EXTENTS == 0 {
            Self::Disabled
        } else if config & EXTENT_PURGE != 0 {
            Self::Purge
        } else if config & EXTENT_BITMAP != 0 {
            Self::Bitmap
        } else {
            Self::Linked
        }
    }

    fn uses_bitmap(self) -> bool {
        matches!(self, Self::Bitmap | Self::Purge)
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum ExtentReuseMode {
    Off,
    Discard,
    Resident,
}

impl ExtentReuseMode {
    fn from_config(config: usize) -> Self {
        if config & EXTENT_REUSE_RESIDENT != 0 {
            Self::Resident
        } else if config & EXTENT_REUSE != 0 {
            Self::Discard
        } else {
            Self::Off
        }
    }
}

#[inline]
fn size_class(layout: Layout) -> Option<usize> {
    let size = layout.size().max(1);
    let align = layout.align();
    if size > MAX_SMALL || align > MAX_SMALL {
        return None;
    }
    let mut class = CLASS_LOOKUP[(size - 1) / 16] as usize;
    while CLASS_SIZES[class] & (align - 1) != 0 {
        class += 1;
    }
    Some(class)
}

#[inline]
fn span_bytes(index: usize) -> usize {
    if index < NUM_CLASSES {
        SPAN_BYTES
    } else {
        EXTENT_BYTES
    }
}

#[inline]
fn class_stride(index: usize) -> usize {
    if index < NUM_CLASSES {
        CLASS_SIZES[index]
    } else if index == UPPER_EXTENT_CLASS {
        UPPER_EXTENT_CAPACITY
    } else {
        bin_capacity(index - NUM_CLASSES)
    }
}

#[inline]
fn extent_class(alignment: usize, rounded_bytes: usize, upper: bool) -> Option<usize> {
    if alignment > PAGE {
        None
    } else if rounded_bytes <= EXTENT_MAX_CAPACITY {
        capacity_bin(rounded_bytes).map(|bin| NUM_CLASSES + bin)
    } else if upper && rounded_bytes > UPPER_EXTENT_FLOOR && rounded_bytes <= UPPER_EXTENT_CAPACITY
    {
        Some(UPPER_EXTENT_CLASS)
    } else {
        None
    }
}

fn map(len: usize) -> *mut u8 {
    // SAFETY: Anonymous, private mapping with no fixed address or file.
    let addr = unsafe {
        libc::mmap(
            null_mut(),
            len,
            libc::PROT_READ | libc::PROT_WRITE,
            libc::MAP_PRIVATE | libc::MAP_ANONYMOUS,
            -1,
            0,
        )
    };
    if addr == libc::MAP_FAILED {
        null_mut()
    } else {
        addr.cast()
    }
}

fn map_aligned(bytes: usize, alignment: usize) -> *mut u8 {
    let raw = map(bytes + alignment);
    if raw.is_null() {
        return raw;
    }
    let prefix = raw.align_offset(alignment);
    // SAFETY: The two page-aligned trims are outside the kept interval.
    unsafe {
        let aligned = raw.add(prefix);
        if prefix != 0 {
            unmap(raw, prefix);
        }
        unmap(aligned.add(bytes), alignment - prefix);
        aligned
    }
}

unsafe fn unmap(addr: *mut u8, len: usize) {
    // SAFETY: The caller owns this page-aligned interval and has ensured that
    // no live blocks or allocator operations still reference it.
    let result = unsafe { libc::munmap(addr.cast(), len) };
    if result != 0 {
        // A failed unmap would invalidate the allocator's ownership model.
        // Avoid unwinding or allocating while handling that failure.
        std::process::abort();
    }
}

fn add_mapping(len: usize) {
    let mapped = MAPPED.fetch_add(len, Relaxed) + len;
    let _ = PEAK_MAPPED.fetch_max(mapped, Relaxed);
}

unsafe fn release_mapping(addr: *mut u8, len: usize) {
    // SAFETY: Forwarded unique mapping ownership from the caller.
    unsafe { unmap(addr, len) };
    let _ = MAPPED.fetch_sub(len, Relaxed);
}

/// Whole, empty extent mappings available to any size class on this worker.
struct ExtentPool {
    mode: ExtentReuseMode,
    slots: [*mut Span; MAX_EXTENT_POOL_SLOTS],
    len: usize,
    limit: usize,
}

impl ExtentPool {
    const EMPTY: Self = Self {
        mode: ExtentReuseMode::Off,
        slots: [null_mut(); MAX_EXTENT_POOL_SLOTS],
        len: 0,
        limit: DEFAULT_EXTENT_POOL_MIB * MIB / EXTENT_BYTES,
    };

    fn configure(&mut self, config: usize) {
        self.mode = ExtentReuseMode::from_config(config);
        let mib = match (config & EXTENT_POOL_MIB_MASK) >> EXTENT_POOL_MIB_SHIFT {
            0 => DEFAULT_EXTENT_POOL_MIB,
            value => value,
        };
        self.limit = mib * MIB / EXTENT_BYTES;
    }

    fn take(&mut self, activity: &mut MappingActivity) -> *mut Span {
        if self.len == 0 {
            return null_mut();
        }
        self.len -= 1;
        activity.extent_reuse_hits += 1;
        std::mem::replace(&mut self.slots[self.len], null_mut())
    }

    unsafe fn release_empty(
        &mut self,
        span: *mut Span,
        idle: bool,
        activity: &mut MappingActivity,
    ) {
        // SAFETY: Only detached extents with no outstanding blocks enter the
        // pool. Discard may zero the header. Checkout rebuilds it.
        if !idle && self.mode != ExtentReuseMode::Off {
            if self.len >= self.limit {
                activity.extent_reuse_full_returns += 1;
            } else if self.mode == ExtentReuseMode::Resident
                || unsafe { libc::madvise(span.cast(), EXTENT_BYTES, libc::MADV_DONTNEED) } == 0
            {
                self.slots[self.len] = span;
                self.len += 1;
                return;
            }
        }
        // SAFETY: The caller detached this extent and owns its mapping.
        unsafe { release_mapping(span.cast(), EXTENT_BYTES) };
        activity.extent_releases += 1;
    }

    fn clear(&mut self, activity: &mut MappingActivity) {
        for slot in &mut self.slots[..std::mem::take(&mut self.len)] {
            // SAFETY: Pool slots exclusively own detached empty mappings.
            let span = std::mem::replace(slot, null_mut());
            unsafe { release_mapping(span.cast(), EXTENT_BYTES) };
            activity.extent_releases += 1;
        }
    }
}

struct SlabRegionLocal {
    prev: *mut SlabRegion,
    next: *mut SlabRegion,
    free: u32,
}

struct SlabRegion {
    local: UnsafeCell<SlabRegionLocal>,
    // One pin for the worker and one for each still-live slab it retires.
    // Active slab checkout and return only use owner-local state.
    retired_refs: AtomicUsize,
}

impl SlabRegion {
    unsafe fn local(region: *mut Self) -> *mut SlabRegionLocal {
        // SAFETY: The owner holds its reference and is the only local writer.
        unsafe { (*region).local.get() }
    }

    unsafe fn release_retired_ref(region: *mut Self) {
        // SAFETY: The caller holds a region reference. The final release
        // synchronizes with all earlier users before unmapping the header.
        if unsafe { (*region).retired_refs.fetch_sub(1, AcqRel) } == 1 {
            let _ = RETIRED_MAPPED.fetch_sub(SLAB_REGION_BYTES, Relaxed);
            unsafe { release_mapping(region.cast(), SLAB_REGION_BYTES) };
        }
    }
}

// Each region is on exactly one of these owner-only lists. Moving a region
// across lists means checkout never has to scan full regions.
struct SlabRegions {
    available: *mut SlabRegion,
    full: *mut SlabRegion,
    mapped: usize,
}

impl SlabRegions {
    const EMPTY: Self = Self {
        available: null_mut(),
        full: null_mut(),
        mapped: 0,
    };

    unsafe fn link(head: &mut *mut SlabRegion, region: *mut SlabRegion) {
        // SAFETY: This is an unlinked region and both entries belong to the owner.
        unsafe {
            let local = &mut *SlabRegion::local(region);
            local.prev = null_mut();
            local.next = *head;
            if !head.is_null() {
                (*SlabRegion::local(*head)).prev = region;
            }
            *head = region;
        }
    }

    unsafe fn unlink(head: &mut *mut SlabRegion, region: *mut SlabRegion) {
        // SAFETY: This region and its neighbors belong to this list and owner.
        unsafe {
            let local = &*SlabRegion::local(region);
            if local.prev.is_null() {
                *head = local.next;
            } else {
                (*SlabRegion::local(local.prev)).next = local.next;
            }
            if !local.next.is_null() {
                (*SlabRegion::local(local.next)).prev = local.prev;
            }
        }
    }

    fn take(&mut self, activity: &mut MappingActivity) -> (*mut u8, *mut SlabRegion) {
        if self.available.is_null() {
            let region = map_aligned(SLAB_REGION_BYTES, SLAB_REGION_BYTES).cast::<SlabRegion>();
            if region.is_null() {
                return (null_mut(), null_mut());
            }
            // SAFETY: Fresh aligned mapping owned exclusively by this thread.
            unsafe {
                region.write(SlabRegion {
                    local: UnsafeCell::new(SlabRegionLocal {
                        prev: null_mut(),
                        next: null_mut(),
                        free: SLAB_REGION_FREE,
                    }),
                    retired_refs: AtomicUsize::new(1),
                });
            }
            self.available = region;
            self.mapped += SLAB_REGION_BYTES;
            activity.slab_region_maps += 1;
            add_mapping(SLAB_REGION_BYTES);
        }
        let region = self.available;
        // SAFETY: The available list contains only owned regions with a free bit.
        unsafe {
            let free = &mut (*SlabRegion::local(region)).free;
            let slot = free.trailing_zeros() as usize;
            *free &= !(1 << slot);
            if *free == 0 {
                Self::unlink(&mut self.available, region);
                Self::link(&mut self.full, region);
            }
            (region.cast::<u8>().add(slot * SPAN_BYTES), region)
        }
    }

    unsafe fn release_span(
        &mut self,
        span: *mut Span,
        bytes: usize,
        activity: &mut MappingActivity,
    ) {
        // SAFETY: The slab is unlinked and has no outstanding blocks or frees.
        let region = unsafe { (*span).remote.region };
        if region.is_null() {
            unsafe { release_mapping(span.cast(), bytes) };
            return;
        }
        let slot = (span.addr() - region.addr()) / SPAN_BYTES;
        // SAFETY: The owner owns the region and the returned slot is in use.
        unsafe {
            let free = &mut (*SlabRegion::local(region)).free;
            let was_full = *free == 0;
            *free |= 1 << slot;
            if *free == SLAB_REGION_FREE {
                Self::unlink(&mut self.available, region);
                self.mapped -= SLAB_REGION_BYTES;
                activity.slab_region_releases += 1;
                release_mapping(region.cast(), SLAB_REGION_BYTES);
            } else {
                if was_full {
                    Self::unlink(&mut self.full, region);
                    Self::link(&mut self.available, region);
                }
                discard_slab(span);
            }
        }
    }

    fn retire(&mut self) {
        for mut region in [self.available, self.full] {
            while !region.is_null() {
                // SAFETY: All slabs have transferred their outstanding pins.
                // The worker's pin protects the links through this read.
                unsafe {
                    let next = (*SlabRegion::local(region)).next;
                    let _ = RETIRED_MAPPED.fetch_add(SLAB_REGION_BYTES, Relaxed);
                    SlabRegion::release_retired_ref(region);
                    region = next;
                }
            }
        }
        *self = Self::EMPTY;
    }
}

unsafe fn discard_slab(span: *mut Span) {
    // SAFETY: The caller owns the whole empty slab and its region is pinned.
    // A failed discard leaves reusable pages resident. Reuse never assumes zeroes.
    let _ = unsafe { libc::madvise(span.cast(), SPAN_BYTES, libc::MADV_DONTNEED) };
}

#[derive(Clone, Copy)]
struct FreeList {
    head: *mut u8,
    tail: *mut u8,
    count: usize,
    requested: usize,
}

impl FreeList {
    const EMPTY: Self = Self {
        head: null_mut(),
        tail: null_mut(),
        count: 0,
        requested: 0,
    };
}

struct RemoteState {
    pending: FreeList,
    // Zero means active. The retiring owner installs the number of still-live
    // blocks while holding the lock, after reconciling all published frees.
    retired_live: usize,
    returned_slots: u32,
}

#[repr(align(64))]
struct Remote {
    locked: AtomicBool,
    bitmap_slab: bool,
    state: UnsafeCell<RemoteState>,
    region: *mut SlabRegion,
}

impl Remote {
    const fn new(region: *mut SlabRegion) -> Self {
        Self {
            locked: AtomicBool::new(false),
            bitmap_slab: false,
            state: UnsafeCell::new(RemoteState {
                pending: FreeList::EMPTY,
                retired_live: 0,
                returned_slots: 0,
            }),
            region,
        }
    }

    unsafe fn with<R>(remote: *const Self, operation: impl FnOnce(&mut RemoteState) -> R) -> R {
        // SAFETY: The caller keeps the mapping live until the release store.
        // A raw pointer avoids keeping a borrow of the span past publication.
        unsafe {
            while (*remote)
                .locked
                .compare_exchange_weak(false, true, Acquire, Relaxed)
                .is_err()
            {
                while (*remote).locked.load(Relaxed) {
                    std::hint::spin_loop();
                }
            }
            let result = operation(&mut *(*remote).state.get());
            (*remote).locked.store(false, Release);
            result
        }
    }
}

// Bitmap spans reserve space before the first slot for metadata. Keep
// reader-visible geometry on a different cache line from the owner bitmap.
#[repr(C)]
struct SpanSlots {
    stride: usize,
    first_slot: usize,
    bitmap: bool,
    owner_slots: UnsafeCell<SlotBitmap>,
}

#[repr(align(64))]
struct SlotBitmap {
    free: u32,
    dirty_free: u32,
    available_prev: *mut Span,
    available_next: *mut Span,
    available_bucket: u32,
}

impl SpanSlots {
    fn new(stride: usize, first_slot: usize, span_bytes: usize, bitmap: bool) -> Self {
        let count = (span_bytes - first_slot) / stride;
        debug_assert!(count < u32::BITS as usize);
        Self {
            stride,
            first_slot,
            bitmap,
            owner_slots: UnsafeCell::new(SlotBitmap {
                free: (1 << count) - 1,
                dirty_free: 0,
                available_prev: null_mut(),
                available_next: null_mut(),
                available_bucket: 0,
            }),
        }
    }
}

impl SlotBitmap {
    fn release(&mut self, bits: u32) {
        self.free |= bits;
        self.dirty_free |= bits;
    }
}

struct SpanLocal {
    prev: *mut Span,
    next: *mut Span,
    head: *mut u8,
    bump: usize,
    live: usize,
    requested: usize,
}

#[repr(C)]
struct Span {
    owner: usize,
    class: usize,
    local: UnsafeCell<SpanLocal>,
    remote: Remote,
}

// Region ownership fits in the existing remote cache line and preserves
// the slab header size and block offsets in the standalone control.
const _: () = assert!(size_of::<Remote>() == 64);

impl Span {
    fn create(
        owner: usize,
        class: usize,
        stride: usize,
        bytes: usize,
        mode: ExtentMode,
        regions: Option<(&mut SlabRegions, &mut MappingActivity)>,
        reused_extent: *mut Self,
    ) -> *mut Self {
        let (span, align, region) = if bytes == EXTENT_BYTES {
            // The allocation prefix records the owning extent explicitly, so
            // the extent itself needs only the OS's normal page alignment.
            let span = if reused_extent.is_null() {
                map(bytes)
            } else {
                reused_extent.cast()
            };
            (span, PAGE, null_mut())
        } else {
            let (mut span, region) = regions
                .map(|(regions, activity)| regions.take(activity))
                .unwrap_or((null_mut(), null_mut()));
            if span.is_null() {
                span = map_aligned(SPAN_BYTES, SPAN_BYTES);
            }
            (span, 1 << stride.trailing_zeros(), region)
        };
        if span.is_null() {
            return null_mut();
        }
        let span = span.cast::<Self>();
        // SAFETY: The aligned span is uniquely owned and has room for the header.
        unsafe {
            span.write(Self {
                owner,
                class,
                local: UnsafeCell::new(SpanLocal {
                    prev: null_mut(),
                    next: null_mut(),
                    head: null_mut(),
                    bump: size_of::<Self>().next_multiple_of(align),
                    live: 0,
                    requested: 0,
                }),
                remote: Remote::new(region),
            });
            if bytes == EXTENT_BYTES {
                Self::span_slots(span).write(SpanSlots::new(
                    stride,
                    PAGE,
                    bytes,
                    mode.uses_bitmap(),
                ));
            }
        }
        if region.is_null() && reused_extent.is_null() {
            add_mapping(bytes);
        }
        span
    }

    #[inline]
    unsafe fn local(span: *mut Self) -> *mut SpanLocal {
        // SAFETY: Caller holds owner-thread access to this live header.
        unsafe { (*span).local.get() }
    }

    #[inline]
    unsafe fn span_slots(span: *mut Self) -> *mut SpanSlots {
        // SAFETY: The caller holds a slot span with a reserved header page
        // containing both aligned metadata structures.
        const {
            assert!(size_of::<Span>() + size_of::<SpanSlots>() <= PAGE);
            assert!(align_of::<Span>() == align_of::<SpanSlots>());
        };
        unsafe { span.add(1).cast() }
    }

    unsafe fn slot_bit(span: *mut Self, slot: *mut u8) -> u32 {
        // SAFETY: A live slot keeps the span's initialized header mapped.
        let slots = unsafe { &*Self::span_slots(span) };
        if !slots.bitmap {
            return 0;
        }
        1 << ((slot.addr() - span.addr() - slots.first_slot) / slots.stride)
    }

    unsafe fn purge_free_slots(span: *mut Self, calls: &mut u64, purged_bytes: &mut u64) {
        // SAFETY: The owner reconciled pending frees. Slot runs are page
        // aligned and their occupancy metadata lives in the reserved header.
        unsafe {
            let slots = Self::span_slots(span);
            let local = &mut *(*slots).owner_slots.get();
            let stride = (*slots).stride;
            let mut dirty = local.dirty_free;
            while dirty != 0 {
                let first = dirty.trailing_zeros();
                let count = (dirty >> first).trailing_ones();
                let mask = ((1 << count) - 1) << first;
                let start = span
                    .cast::<u8>()
                    .add((*slots).first_slot + first as usize * stride);
                let len = count as usize * stride;
                *calls += 1;
                if libc::madvise(start.cast(), len, libc::MADV_DONTNEED) == 0 {
                    local.dirty_free &= !mask;
                    *purged_bytes += len as u64;
                }
                dirty &= !mask;
            }
        }
    }

    #[inline]
    unsafe fn take_block(span: *mut Self, index: usize, requested: usize) -> *mut u8 {
        // SAFETY: This path runs on the sole owner of the span.
        let local = unsafe { &mut *Self::local(span) };
        let block = if !local.head.is_null() {
            // Only linked spans populate this list. Recycled blocks need
            // neither the class geometry nor the bump pointer.
            let block = local.head;
            // SAFETY: The free list contains initialized intrusive links.
            local.head = unsafe { block.cast::<*mut u8>().read() };
            block
        } else {
            // A slab already linked when the pipeline starts can still use
            // the original layout. Check the header of the actual span.
            let first_slot = match index {
                RECLAIM_SLAB_CLASS if unsafe { (*span).remote.bitmap_slab } => RECLAIM_SLAB_STRIDE,
                NUM_CLASSES.. if unsafe { (*Self::span_slots(span)).bitmap } => PAGE,
                _ => 0,
            };
            if first_slot != 0 {
                // SAFETY: Bitmap spans have initialized slot metadata and
                // each free bit identifies a valid, owner-visible slot.
                let meta = unsafe { &*Self::span_slots(span) };
                let slots = unsafe { &mut *meta.owner_slots.get() };
                let free = slots.free;
                if free == 0 {
                    return null_mut();
                }
                let slot = free.trailing_zeros();
                let mask = 1 << slot;
                slots.free = free & !mask;
                slots.dirty_free &= !mask;
                let stride = match index {
                    RECLAIM_SLAB_CLASS => RECLAIM_SLAB_STRIDE,
                    EXTENT_20K_CLASS => EXTENT_20K_STRIDE,
                    _ => meta.stride,
                };
                unsafe { span.cast::<u8>().add(first_slot + slot as usize * stride) }
            } else {
                let stride = class_stride(index);
                if local.bump + stride > span_bytes(index) {
                    return null_mut();
                }
                // SAFETY: bump is aligned for this class and lies inside the span.
                let block = unsafe { span.cast::<u8>().add(local.bump) };
                local.bump += stride;
                block
            }
        };
        local.live += 1;
        local.requested += requested;
        block
    }

    unsafe fn drain(span: *mut Self) -> usize {
        // SAFETY: The owner holds all local access. After detachment the remote
        // list is exclusively local and contains live-accounted blocks.
        let (pending, slots) = unsafe {
            Remote::with(ptr::addr_of!((*span).remote), |state| {
                (
                    std::mem::replace(&mut state.pending, FreeList::EMPTY),
                    std::mem::take(&mut state.returned_slots),
                )
            })
        };
        if pending.count != 0 {
            let local = unsafe { &mut *Self::local(span) };
            if slots != 0 {
                // SAFETY: Only a bitmap span publishes returned slot bits.
                unsafe { (*(*Self::span_slots(span)).owner_slots.get()).release(slots) };
            } else {
                unsafe { pending.tail.cast::<*mut u8>().write(local.head) };
                local.head = pending.head;
            }
            local.live -= pending.count;
            local.requested -= pending.requested;
        }
        pending.requested
    }

    unsafe fn remote_free(span: *mut Self, block: *mut u8, requested: usize, slot_bit: u32) {
        // SAFETY: The live block keeps the mapping valid through publication.
        // In the retired state, each caller holds one outstanding block.
        let release = unsafe {
            Remote::with(ptr::addr_of!((*span).remote), |state| {
                if state.retired_live != 0 {
                    state.retired_live -= 1;
                    return state.retired_live == 0;
                }
                let pending = &mut state.pending;
                if slot_bit != 0 {
                    state.returned_slots |= slot_bit;
                } else {
                    block.cast::<*mut u8>().write(pending.head);
                    if pending.count == 0 {
                        pending.tail = block;
                    }
                    pending.head = block;
                }
                pending.count += 1;
                pending.requested += requested;
                false
            })
        };
        if release {
            // SAFETY: This caller released the last retired block and the
            // owner relinquished the span before making retirement visible.
            unsafe {
                let region = (*span).remote.region;
                if region.is_null() {
                    let bytes = span_bytes((*span).class);
                    let _ = RETIRED_MAPPED.fetch_sub(bytes, Relaxed);
                    release_mapping(span.cast(), bytes);
                } else {
                    discard_slab(span);
                    SlabRegion::release_retired_ref(region);
                }
            }
        }
    }

    unsafe fn retire(span: *mut Self) {
        // SAFETY: The owner still holds this span until it publishes retirement.
        let (bytes, region) = unsafe { (span_bytes((*span).class), (*span).remote.region) };
        // SAFETY: Only the owner can retire a span. The remote lock serializes
        // the transition with frees already in flight.
        let remaining = unsafe {
            Remote::with(ptr::addr_of!((*span).remote), |state| {
                let live = (*Self::local(span)).live - state.pending.count;
                if live != 0 {
                    if region.is_null() {
                        let _ = RETIRED_MAPPED.fetch_add(bytes, Relaxed);
                    } else {
                        // The worker still pins this region while traversing its slabs.
                        let _ = (*region).retired_refs.fetch_add(1, Relaxed);
                    }
                    state.retired_live = live;
                }
                live
            })
        };
        if remaining == 0 {
            // SAFETY: All previously allocated blocks have been returned.
            unsafe {
                if region.is_null() {
                    release_mapping(span.cast(), bytes);
                } else {
                    discard_slab(span);
                }
            }
        }
        // Otherwise a remote free may already have unmapped the span.
    }
}

#[derive(Clone, Copy)]
struct Mapping {
    base: *mut u8,
    len: usize,
}

impl Mapping {
    const EMPTY: Self = Self {
        base: null_mut(),
        len: 0,
    };

    const EXTENT_TAG: usize = 1;

    // Dedicated mapping lengths are page multiples. An extent record uses
    // the low bit to distinguish its span pointer and payload offset.
    fn extent(span: *mut Span) -> Self {
        Self {
            base: span.cast(),
            len: Self::EXTENT_TAG,
        }
    }

    fn is_extent(self) -> bool {
        self.len & Self::EXTENT_TAG != 0
    }

    unsafe fn extent_slot(self, block: *mut u8) -> *mut u8 {
        // SAFETY: An extent record stores this block's offset from its slot.
        unsafe { block.sub(self.len & !Self::EXTENT_TAG) }
    }

    unsafe fn write_before(mut self, slot: *mut u8, alignment: usize) -> *mut u8 {
        // SAFETY: The caller reserves sufficient space in the slot or mapping
        // for both alignment padding and the allocation record.
        unsafe {
            let start = slot.add(size_of::<Self>());
            let block = start.add(start.align_offset(alignment));
            if self.is_extent() {
                self.len |= block.addr() - slot.addr();
            }
            block.cast::<Self>().sub(1).write(self);
            block
        }
    }
}

#[inline]
const fn bin_capacity(bin: usize) -> usize {
    (1 << (BIN_MIN_SHIFT + (bin / 4) as u32)) / 4 * (4 + bin % 4)
}

#[inline]
fn capacity_bin(len: usize) -> Option<usize> {
    if len > 1 << BIN_MAX_SHIFT {
        return None;
    }
    let len = len.max(1 << BIN_MIN_SHIFT);
    let shift = len.ilog2();
    let floor = 1 << shift;
    let quarter = floor / 4;
    Some(4 * (shift - BIN_MIN_SHIFT) as usize + (len - floor).div_ceil(quarter))
}

/// Configures the experimental cache before the pipeline workers start.
/// Exact mode reproduces the previous mapping cache for paired comparisons.
pub fn configure_large_cache_from_env() -> Result<(), &'static str> {
    let mode = match std::env::var("DFE_ALLOC_LARGE_CACHE") {
        Ok(value) if value == "exact" => 0,
        Ok(value) if value == "binned" => BINNED,
        Err(std::env::VarError::NotPresent) => BINNED,
        _ => return Err("DFE_ALLOC_LARGE_CACHE must be exact or binned"),
    };
    let budget = match std::env::var("DFE_ALLOC_LARGE_CACHE_MIB") {
        Ok(value) => {
            value
                .parse::<usize>()
                .ok()
                .filter(|mib| *mib <= MAX_MAP_CACHE_MIB)
                .ok_or("DFE_ALLOC_LARGE_CACHE_MIB must be an integer from 0 to 8")?
                * MIB
        }
        Err(std::env::VarError::NotPresent) => MAP_CACHE_BYTES,
        _ => return Err("DFE_ALLOC_LARGE_CACHE_MIB must be an integer from 0 to 8"),
    };
    let eviction = match std::env::var("DFE_ALLOC_LARGE_CACHE_EVICTION") {
        Ok(value) if value == "largest" => 0,
        Ok(value) if value == "reserve-hot" => RESERVE_HOT_BINS,
        Err(std::env::VarError::NotPresent) => 0,
        _ => return Err("DFE_ALLOC_LARGE_CACHE_EVICTION must be largest or reserve-hot"),
    };
    let trim = match std::env::var("DFE_ALLOC_LARGE_CACHE_TRIM") {
        Ok(value) if value == "immediate" => 0,
        Ok(value) if value == "gradual" => GRADUAL_TRIM,
        Err(std::env::VarError::NotPresent) => 0,
        _ => return Err("DFE_ALLOC_LARGE_CACHE_TRIM must be immediate or gradual"),
    };
    if mode == 0 && (eviction | trim) != 0 {
        return Err("reserve-hot eviction and gradual trim require the binned cache");
    }
    let extents = match std::env::var("DFE_ALLOC_MEDIUM_EXTENTS") {
        Ok(value) if value == "off" => 0,
        Ok(value) if value == "on" => MEDIUM_EXTENTS,
        Err(std::env::VarError::NotPresent) => 0,
        _ => return Err("DFE_ALLOC_MEDIUM_EXTENTS must be on or off"),
    };
    let slots = match std::env::var("DFE_ALLOC_MEDIUM_EXTENT_SLOTS") {
        Ok(value) if value == "list" => 0,
        Ok(value) if value == "bitmap" => EXTENT_BITMAP,
        Ok(value) if value == "bitmap-purge" => EXTENT_BITMAP | EXTENT_PURGE,
        Err(std::env::VarError::NotPresent) => 0,
        _ => return Err("DFE_ALLOC_MEDIUM_EXTENT_SLOTS must be list, bitmap, or bitmap-purge"),
    };
    if slots != 0 && extents == 0 {
        return Err("bitmap extent slots require DFE_ALLOC_MEDIUM_EXTENTS=on");
    }
    let upper_extent = match std::env::var("DFE_ALLOC_UPPER_EXTENT") {
        Ok(value) if value == "off" => 0,
        Ok(value) if value == "on" => UPPER_EXTENT,
        Err(std::env::VarError::NotPresent) => 0,
        _ => return Err("DFE_ALLOC_UPPER_EXTENT must be on or off"),
    };
    if upper_extent != 0 && extents == 0 {
        return Err("DFE_ALLOC_UPPER_EXTENT=on requires DFE_ALLOC_MEDIUM_EXTENTS=on");
    }
    let slab_regions = match std::env::var("DFE_ALLOC_SLAB_REGIONS") {
        Ok(value) if value == "off" => 0,
        Ok(value) if value == "on" => SLAB_REGIONS,
        Err(std::env::VarError::NotPresent) => 0,
        _ => return Err("DFE_ALLOC_SLAB_REGIONS must be on or off"),
    };
    let extent_reuse = match std::env::var("DFE_ALLOC_EXTENT_REUSE") {
        Ok(value) if value == "off" => 0,
        Ok(value) if value == "discard" => EXTENT_REUSE,
        Ok(value) if value == "resident" => EXTENT_REUSE | EXTENT_REUSE_RESIDENT,
        Err(std::env::VarError::NotPresent) => 0,
        _ => return Err("DFE_ALLOC_EXTENT_REUSE must be off, discard, or resident"),
    };
    if extent_reuse != 0 && extents == 0 {
        return Err("extent reuse requires DFE_ALLOC_MEDIUM_EXTENTS=on");
    }
    let extent_pool_mib = match std::env::var("DFE_ALLOC_EXTENT_REUSE_MIB") {
        Ok(value) => value
            .parse::<usize>()
            .ok()
            .filter(|mib| (1..=MAX_EXTENT_POOL_MIB).contains(mib))
            .ok_or("DFE_ALLOC_EXTENT_REUSE_MIB must be an integer from 1 to 4")?,
        Err(std::env::VarError::NotPresent) => 0,
        _ => return Err("DFE_ALLOC_EXTENT_REUSE_MIB must be an integer from 1 to 4"),
    };
    let fullest_extents = match std::env::var("DFE_ALLOC_FULLEST_EXTENTS") {
        Ok(value) if value == "on" && slots & EXTENT_BITMAP != 0 => FULLEST_EXTENTS,
        Ok(value) if value == "on" => {
            return Err("DFE_ALLOC_FULLEST_EXTENTS=on requires bitmap extent slots");
        }
        Ok(value) if value == "off" => 0,
        Err(std::env::VarError::NotPresent) => 0,
        _ => return Err("DFE_ALLOC_FULLEST_EXTENTS must be on or off"),
    };
    let reclaim_slab_pages = match std::env::var("DFE_ALLOC_RECLAIM_SLAB_PAGES") {
        Ok(value) if value == "on" => RECLAIM_SLAB_PAGES,
        Ok(value) if value == "off" => 0,
        Err(std::env::VarError::NotPresent) => 0,
        _ => return Err("DFE_ALLOC_RECLAIM_SLAB_PAGES must be on or off"),
    };
    let node_activity = match std::env::var("DFE_ALLOC_NODE_ACTIVITY") {
        Ok(value) if value == "on" => NODE_ACTIVITY,
        Ok(value) if value == "off" => 0,
        Err(std::env::VarError::NotPresent) => 0,
        _ => return Err("DFE_ALLOC_NODE_ACTIVITY must be on or off"),
    };
    LARGE_CACHE_CONFIG.store(
        budget
            | mode
            | eviction
            | trim
            | extents
            | slots
            | upper_extent
            | slab_regions
            | extent_reuse
            | fullest_extents
            | node_activity
            | reclaim_slab_pages
            | (extent_pool_mib << EXTENT_POOL_MIB_SHIFT),
        Relaxed,
    );
    Ok(())
}

pub(crate) const LARGE_SIZE_BANDS: usize = 3;

pub(crate) struct MappingActivity {
    pub(crate) slab_maps: u64,
    pub(crate) slab_releases: u64,
    pub(crate) slab_purge_calls: u64,
    pub(crate) slab_purged_bytes: u64,
    pub(crate) slab_region_maps: u64,
    pub(crate) slab_region_releases: u64,
    pub(crate) extent_maps: u64,
    pub(crate) extent_releases: u64,
    pub(crate) extent_reuse_hits: u64,
    pub(crate) extent_reuse_full_returns: u64,
    pub(crate) extent_purge_calls: u64,
    pub(crate) extent_purged_bytes: u64,
    pub(crate) large_maps: u64,
    pub(crate) large_releases: u64,
    pub(crate) hits: [u64; LARGE_SIZE_BANDS],
    pub(crate) misses: [u64; LARGE_SIZE_BANDS],
    pub(crate) realloc_copied: u64,
    pub(crate) large_realloc_in_place: u64,
}

impl MappingActivity {
    const EMPTY: Self = Self {
        slab_maps: 0,
        slab_releases: 0,
        slab_purge_calls: 0,
        slab_purged_bytes: 0,
        slab_region_maps: 0,
        slab_region_releases: 0,
        extent_maps: 0,
        extent_releases: 0,
        extent_reuse_hits: 0,
        extent_reuse_full_returns: 0,
        extent_purge_calls: 0,
        extent_purged_bytes: 0,
        large_maps: 0,
        large_releases: 0,
        hits: [0; LARGE_SIZE_BANDS],
        misses: [0; LARGE_SIZE_BANDS],
        realloc_copied: 0,
        large_realloc_in_place: 0,
    };
}

impl Default for MappingActivity {
    fn default() -> Self {
        Self::EMPTY
    }
}

#[inline]
fn size_band(len: usize) -> usize {
    usize::from(len >= 64 * 1024) + usize::from(len >= 256 * 1024)
}

#[derive(Clone, Copy)]
struct Class {
    head: *mut Span,
    active: *mut Span,
    scan: *mut Span,
    empty: *mut Span,
}

impl Class {
    const EMPTY: Self = Self {
        head: null_mut(),
        active: null_mut(),
        scan: null_mut(),
        empty: null_mut(),
    };

    unsafe fn link(&mut self, span: *mut Span) {
        // SAFETY: All linked spans belong to this thread and class.
        unsafe {
            (*Span::local(span)).next = self.head;
            if !self.head.is_null() {
                (*Span::local(self.head)).prev = span;
            }
        }
        self.head = span;
    }

    // Returns whether the empty span was unlinked for release.
    unsafe fn keep_or_unlink_empty(&mut self, span: *mut Span) -> bool {
        if self.empty.is_null() {
            self.empty = span;
        } else if self.empty != span {
            // SAFETY: The owner established that the linked span is empty.
            unsafe { self.unlink_empty(span) };
            return true;
        }
        false
    }

    unsafe fn unlink_empty(&mut self, span: *mut Span) {
        // SAFETY: This is a linked owner-only span with no outstanding blocks.
        // Update every local alias before releasing the mapping.
        unsafe {
            let local = &*Span::local(span);
            let (prev, next) = (local.prev, local.next);
            if prev.is_null() {
                self.head = next;
            } else {
                (*Span::local(prev)).next = next;
            }
            if !next.is_null() {
                (*Span::local(next)).prev = prev;
            }
            if self.empty == span {
                self.empty = null_mut();
            }
            if self.active == span {
                self.active = self.empty;
            }
            if self.scan == span {
                self.scan = next;
            }
        }
    }
}

// Only nonempty, nonactive bitmap extents with available slots are indexed.
// The bucket is the count of owner-visible free slots. Zero also marks an
// unlinked extent, so selecting the first occupied bucket finds the fullest.
struct AvailableExtents {
    heads: [*mut Span; u32::BITS as usize],
    occupied: u32,
}

impl AvailableExtents {
    const EMPTY: Self = Self {
        heads: [null_mut(); u32::BITS as usize],
        occupied: 0,
    };

    unsafe fn slots(span: *mut Span) -> *mut SlotBitmap {
        // SAFETY: Only the owner uses this index and all its entries are
        // live bitmap extents with initialized metadata.
        unsafe { (*Span::span_slots(span)).owner_slots.get() }
    }

    unsafe fn set_bucket(&mut self, span: *mut Span, bucket: u32) {
        // SAFETY: The owner can access every linked extent and its metadata.
        unsafe {
            let slots = &mut *Self::slots(span);
            let previous_bucket = slots.available_bucket;
            if previous_bucket == bucket {
                return;
            }
            if previous_bucket != 0 {
                let (prev, next) = (slots.available_prev, slots.available_next);
                if prev.is_null() {
                    self.heads[previous_bucket as usize] = next;
                    if next.is_null() {
                        self.occupied &= !(1 << previous_bucket);
                    }
                } else {
                    (*Self::slots(prev)).available_next = next;
                }
                if !next.is_null() {
                    (*Self::slots(next)).available_prev = prev;
                }
            }
            slots.available_bucket = bucket;
            slots.available_prev = null_mut();
            slots.available_next = null_mut();
            if bucket != 0 {
                let head = &mut self.heads[bucket as usize];
                slots.available_next = *head;
                if !head.is_null() {
                    (*Self::slots(*head)).available_prev = span;
                }
                *head = span;
                self.occupied |= 1 << bucket;
            }
        }
    }

    unsafe fn refresh(&mut self, span: *mut Span, live: usize) {
        // SAFETY: This nonactive bitmap extent remains linked to its owner.
        let free = if live == 0 {
            0
        } else {
            unsafe { (*Self::slots(span)).free.count_ones() }
        };
        unsafe { self.set_bucket(span, free) };
    }

    fn take_fullest(&mut self) -> Option<*mut Span> {
        if self.occupied == 0 {
            return None;
        }
        let span = self.heads[self.occupied.trailing_zeros() as usize];
        // SAFETY: An occupied bucket contains a linked, available extent.
        unsafe { self.set_bucket(span, 0) };
        Some(span)
    }
}

struct BinDemand {
    depth: [u16; BIN_COUNT],
    seen: u64,
    hot: u64,
    previous_hot: u64,
    surplus: u64,
}

impl BinDemand {
    const EMPTY: Self = Self {
        depth: [0; BIN_COUNT],
        seen: 0,
        hot: 0,
        previous_hot: 0,
        surplus: 0,
    };

    fn record_request(&mut self, bin: usize) {
        let mask = 1 << bin;
        self.hot |= self.seen & mask;
        self.seen |= mask;
    }

    fn insert(&mut self, bin: usize) {
        self.depth[bin] += 1;
        if self.depth[bin] > RESERVED_PER_BIN {
            self.surplus |= 1 << bin;
        }
    }

    fn remove(&mut self, bin: usize) {
        self.depth[bin] -= 1;
        if self.depth[bin] == RESERVED_PER_BIN {
            self.surplus &= !(1 << bin);
        }
    }

    fn age(&mut self) {
        self.previous_hot = std::mem::take(&mut self.hot);
        self.seen = 0;
    }

    fn eviction_candidates(&self, occupied: u64) -> u64 {
        let unreserved = occupied & (self.surplus | !(self.hot | self.previous_hot));
        if unreserved != 0 {
            unreserved
        } else {
            occupied
        }
    }
}

struct Heap {
    owner: usize,
    classes: [Class; HEAP_CLASSES],
    slab_regions: SlabRegions,
    use_slab_regions: bool,
    reclaim_slab_pages: bool,
    used_reclaim_slab: bool,
    slab_idle_since: Option<Instant>,
    cache: [Mapping; MAP_CACHE_SLOTS],
    // A cached mapping's first word is its intrusive next pointer. Only the
    // owning worker can access these lists and a byte cap bounds all entries.
    bins: [*mut u8; BIN_COUNT],
    occupied_bins: u64,
    demand: BinDemand,
    cache_config: usize,
    cached_bytes: usize,
    evict: usize,
    used_large: bool,
    extent_mode: ExtentMode,
    extent_pool: ExtentPool,
    fullest_extents: bool,
    available_extents: [AvailableExtents; EXTENT_BINS + 1],
    upper_extent: bool,
    used_extents: bool,
    extent_idle_since: Option<Instant>,
    idle_since: Option<Instant>,
    activity: MappingActivity,
}

impl Heap {
    const EMPTY: Self = Self {
        owner: 0,
        classes: [Class::EMPTY; HEAP_CLASSES],
        slab_regions: SlabRegions::EMPTY,
        use_slab_regions: false,
        reclaim_slab_pages: false,
        used_reclaim_slab: false,
        slab_idle_since: None,
        cache: [Mapping::EMPTY; MAP_CACHE_SLOTS],
        bins: [null_mut(); BIN_COUNT],
        occupied_bins: 0,
        demand: BinDemand::EMPTY,
        cache_config: MAP_CACHE_BYTES | BINNED,
        cached_bytes: 0,
        evict: 0,
        used_large: false,
        extent_mode: ExtentMode::Disabled,
        extent_pool: ExtentPool::EMPTY,
        fullest_extents: false,
        available_extents: [const { AvailableExtents::EMPTY }; EXTENT_BINS + 1],
        upper_extent: false,
        used_extents: false,
        extent_idle_since: None,
        idle_since: None,
        activity: MappingActivity::EMPTY,
    };

    fn owner(&mut self) -> usize {
        if self.owner == 0 {
            self.owner = NEXT_OWNER.fetch_add(1, Relaxed);
            if self.owner == 0 || self.owner == usize::MAX {
                std::process::abort();
            }
        }
        self.owner
    }

    #[inline]
    fn alloc_block(&mut self, index: usize, requested: usize) -> (*mut u8, *mut Span) {
        let bitmap_slab = index == RECLAIM_SLAB_CLASS && self.reclaim_slab_pages;
        if bitmap_slab {
            self.used_reclaim_slab = true;
        }
        let class = &mut self.classes[index];
        let span = class.active;
        if !span.is_null() {
            // SAFETY: This heap has exclusive access to its active span.
            let block = unsafe { Span::take_block(span, index, requested) };
            if !block.is_null() {
                if class.empty == span {
                    class.empty = null_mut();
                }
                return (block, span);
            }
        }
        self.refill_block(index, requested, bitmap_slab)
    }

    #[inline(never)]
    fn refill_block(
        &mut self,
        index: usize,
        requested: usize,
        bitmap_slab: bool,
    ) -> (*mut u8, *mut Span) {
        let owner = self.owner();
        let class = &mut self.classes[index];
        let bytes = span_bytes(index);
        // SAFETY: This heap owns all the linked spans in the class.
        unsafe {
            if self.fullest_extents
                && bytes == EXTENT_BYTES
                && let Some(available) = self.available_extents[index - NUM_CLASSES].take_fullest()
            {
                class.active = available;
                return (Span::take_block(available, index, requested), available);
            }
            if !class.empty.is_null() {
                let span = std::mem::replace(&mut class.empty, null_mut());
                class.active = span;
                return (Span::take_block(span, index, requested), span);
            }
            let mut probe = if class.scan.is_null() {
                class.head
            } else {
                class.scan
            };
            for _ in 0..PROBE_SPANS {
                if probe.is_null() {
                    break;
                }
                class.scan = (*Span::local(probe)).next;
                let _ = Span::drain(probe);
                let block = Span::take_block(probe, index, requested);
                if !block.is_null() {
                    class.active = probe;
                    return (block, probe);
                }
                probe = class.scan;
            }
            let reused = if bytes == EXTENT_BYTES {
                // The refill path owns the worker cache. An extent mapping
                // has the same geometry regardless of its previous class.
                self.extent_pool.take(&mut self.activity)
            } else {
                null_mut()
            };
            let regions = self
                .use_slab_regions
                .then_some((&mut self.slab_regions, &mut self.activity));
            let stride = class_stride(index);
            let span = Span::create(
                owner,
                index,
                stride,
                bytes,
                self.extent_mode,
                regions,
                reused,
            );
            if span.is_null() {
                return (null_mut(), null_mut());
            }
            if bytes == EXTENT_BYTES {
                self.activity.extent_maps += u64::from(reused.is_null());
            } else {
                self.activity.slab_maps += 1;
                if bitmap_slab {
                    (*span).remote.bitmap_slab = true;
                    Span::span_slots(span).write(SpanSlots::new(
                        stride,
                        RECLAIM_SLAB_STRIDE,
                        SPAN_BYTES,
                        true,
                    ));
                }
            }
            class.link(span);
            class.active = span;
            (Span::take_block(span, index, requested), span)
        }
    }

    unsafe fn free_block(
        &mut self,
        span: *mut Span,
        block: *mut u8,
        requested: usize,
        slot_bit: u32,
    ) {
        // SAFETY: This heap owns the live span. The caller relinquishes a
        // block that is sufficiently large and aligned for its free-list link.
        unsafe {
            let local = &mut *Span::local(span);
            if slot_bit != 0 {
                (*(*Span::span_slots(span)).owner_slots.get()).release(slot_bit);
            } else {
                block.cast::<*mut u8>().write(local.head);
                local.head = block;
            }
            local.live -= 1;
            local.requested -= requested;
            if slot_bit != 0 && self.fullest_extents {
                let index = (*span).class;
                if index >= NUM_CLASSES && self.classes[index].active != span {
                    self.available_extents[index - NUM_CLASSES].refresh(span, local.live);
                }
            }
            if local.live == 0 {
                let index = (*span).class;
                let bytes = span_bytes(index);
                if self.classes[index].keep_or_unlink_empty(span) {
                    if bytes == EXTENT_BYTES {
                        self.extent_pool
                            .release_empty(span, false, &mut self.activity);
                    } else {
                        self.slab_regions
                            .release_span(span, bytes, &mut self.activity);
                        self.activity.slab_releases += 1;
                    }
                }
            }
        }
    }

    fn refresh_large_cache_config(&mut self) {
        let config = LARGE_CACHE_CONFIG.load(Relaxed);
        if config != self.cache_config {
            self.drop_cached_mappings();
            self.demand = BinDemand::EMPTY;
            self.idle_since = None;
            self.cache_config = config;
        }
    }

    fn large_cache_budget(&self) -> usize {
        self.cache_config & !CACHE_FLAGS
    }

    fn take_large_mapping(&mut self, len: usize) -> Mapping {
        let budget = self.large_cache_budget();
        self.used_large = true;
        let band = size_band(len);
        let mut map_len = len;
        if self.cache_config & BINNED == 0 {
            if let Some(entry) = self.cache.iter_mut().find(|entry| entry.len == len) {
                self.cached_bytes -= len;
                self.activity.hits[band] += 1;
                return std::mem::replace(entry, Mapping::EMPTY);
            }
        } else if let Some(bin) = capacity_bin(len) {
            let rounded = bin_capacity(bin);
            if rounded <= budget / 2 {
                map_len = rounded;
                let reserve_hot = self.cache_config & RESERVE_HOT_BINS != 0;
                if reserve_hot && rounded <= RESERVABLE_BIN_MAX {
                    self.demand.record_request(bin);
                }
                let available = self.occupied_bins & (u64::MAX << bin);
                if available != 0 {
                    let best = available.trailing_zeros() as usize;
                    let capacity = bin_capacity(best);
                    if capacity <= len + len / 4 {
                        if reserve_hot && capacity <= RESERVABLE_BIN_MAX {
                            self.demand.hot |= 1 << best;
                        }
                        self.activity.hits[band] += 1;
                        return self.pop_bin(best);
                    }
                }
            }
        }

        self.activity.misses[band] += 1;
        let mapping = new_mapping(map_len);
        if !mapping.base.is_null() {
            self.activity.large_maps += 1;
        }
        mapping
    }

    fn store_large_mapping(&mut self, mapping: Mapping) {
        let budget = self.large_cache_budget();
        self.used_large = true;
        if mapping.len > budget / 2 {
            self.release_large_mapping(mapping);
            return;
        }

        if self.cache_config & BINNED == 0 {
            let index = self
                .cache
                .iter()
                .position(|entry| entry.len == 0)
                .unwrap_or(self.evict);
            let previous = self.cache[index];
            if self.cached_bytes - previous.len + mapping.len <= budget {
                self.cache[index] = mapping;
                self.evict = (index + 1) % MAP_CACHE_SLOTS;
                self.cached_bytes = self.cached_bytes - previous.len + mapping.len;
                if previous.len != 0 {
                    self.release_large_mapping(previous);
                }
                return;
            }
        } else if let Some(bin) = capacity_bin(mapping.len)
            && bin_capacity(bin) == mapping.len
        {
            self.trim_bins_to(budget - mapping.len);
            if self.cache_config & RESERVE_HOT_BINS != 0 {
                self.demand.insert(bin);
            }
            // SAFETY: The cache now exclusively owns the mapping. Its aligned
            // first word is available for an intrusive next pointer.
            unsafe { mapping.base.cast::<*mut u8>().write(self.bins[bin]) };
            self.bins[bin] = mapping.base;
            self.occupied_bins |= 1 << bin;
            self.cached_bytes += mapping.len;
            return;
        }
        // Bootstrap mappings can have a valid length outside the selected
        // bin layout. They remain reclaimable when the policy changes.
        self.release_large_mapping(mapping);
    }

    fn pop_bin(&mut self, bin: usize) -> Mapping {
        let base = self.bins[bin];
        let len = bin_capacity(bin);
        // SAFETY: Callers select a set bit in occupied_bins. The bin owns the
        // mapping and its first word is an initialized list link.
        let next = unsafe { base.cast::<*mut u8>().read() };
        self.bins[bin] = next;
        if next.is_null() {
            self.occupied_bins &= !(1 << bin);
        }
        self.cached_bytes -= len;
        if self.cache_config & RESERVE_HOT_BINS != 0 {
            self.demand.remove(bin);
        }
        Mapping { base, len }
    }

    fn trim_bins_to(&mut self, target: usize) {
        while self.cached_bytes > target {
            let candidates = if self.cache_config & RESERVE_HOT_BINS != 0 {
                self.demand.eviction_candidates(self.occupied_bins)
            } else {
                self.occupied_bins
            };
            let largest = (u64::BITS - candidates.leading_zeros() - 1) as usize;
            let evicted = self.pop_bin(largest);
            self.release_large_mapping(evicted);
        }
    }

    fn release_large_mapping(&mut self, mapping: Mapping) {
        self.activity.large_releases += 1;
        // SAFETY: Callers transfer sole ownership of an unused mapping.
        unsafe { release_mapping(mapping.base, mapping.len) };
    }

    fn drop_cached_mappings(&mut self) {
        while self.occupied_bins != 0 {
            let bin = self.occupied_bins.trailing_zeros() as usize;
            let mapping = self.pop_bin(bin);
            self.release_large_mapping(mapping);
        }
        for slot in 0..self.cache.len() {
            let mapping = std::mem::replace(&mut self.cache[slot], Mapping::EMPTY);
            if mapping.len != 0 {
                self.release_large_mapping(mapping);
            }
        }
        self.cached_bytes = 0;
    }

    fn trim_idle_cache(&mut self, clock: impl FnOnce() -> Instant) {
        if std::mem::replace(&mut self.used_large, false) || self.cached_bytes == 0 {
            self.idle_since = None;
            return;
        }
        if self.cache_config & GRADUAL_TRIM == 0 {
            self.drop_cached_mappings();
            return;
        }

        let now = clock();
        match self.idle_since {
            None => {
                self.idle_since = Some(now);
                self.trim_bins_to((self.large_cache_budget() / 4).min(IDLE_KEEP_MAX));
            }
            Some(since) if now.duration_since(since) >= IDLE_KEEP_TIME => {
                self.drop_cached_mappings();
                self.idle_since = None;
            }
            Some(_) => {}
        }
    }

    fn sample(&mut self) -> WorkerStats {
        self.sample_with_clock(Instant::now)
    }

    fn sample_with_clock(&mut self, clock: impl Fn() -> Instant) -> WorkerStats {
        self.trim_idle_cache(&clock);
        if self.cache_config & RESERVE_HOT_BINS != 0 {
            self.demand.age();
        }
        let mut result = WorkerStats {
            cached_large: self.cached_bytes,
            ..Default::default()
        };
        let extent_idle = !std::mem::replace(&mut self.used_extents, false);
        if extent_idle {
            self.extent_pool.clear(&mut self.activity);
        }
        let purge_extent_slots = if self.extent_mode != ExtentMode::Purge || !extent_idle {
            self.extent_idle_since = None;
            false
        } else {
            let now = clock();
            now.duration_since(*self.extent_idle_since.get_or_insert(now)) >= EXTENT_PURGE_DELAY
        };
        let purge_slab_slots =
            if !self.reclaim_slab_pages || std::mem::replace(&mut self.used_reclaim_slab, false) {
                self.slab_idle_since = None;
                false
            } else {
                let now = clock();
                now.duration_since(*self.slab_idle_since.get_or_insert(now)) >= EXTENT_PURGE_DELAY
            };
        for (index, class) in self.classes.iter_mut().enumerate() {
            let bytes = span_bytes(index);
            let extent = bytes == EXTENT_BYTES;
            let stride = class_stride(index);
            let mut span = class.head;
            while !span.is_null() {
                // SAFETY: Only this heap mutates its links. Save the next link
                // before an empty span can be released.
                unsafe {
                    let next = (*Span::local(span)).next;
                    let returned = Span::drain(span);
                    result.remote_requested += returned;
                    let local = &*Span::local(span);
                    if returned != 0 && extent && self.fullest_extents && class.active != span {
                        self.available_extents[index - NUM_CLASSES].refresh(span, local.live);
                    }
                    if local.live == 0 {
                        let released = if extent && extent_idle {
                            class.unlink_empty(span);
                            true
                        } else {
                            class.keep_or_unlink_empty(span)
                        };
                        if extent {
                            if released {
                                self.extent_pool.release_empty(
                                    span,
                                    extent_idle,
                                    &mut self.activity,
                                );
                            } else {
                                result.empty_extents += bytes;
                                result.extent_mapped += bytes;
                            }
                        } else if released {
                            self.slab_regions
                                .release_span(span, bytes, &mut self.activity);
                            self.activity.slab_releases += 1;
                        } else {
                            result.empty_mapped += bytes;
                            result.slab_mapped += bytes;
                        }
                    } else {
                        if extent {
                            result.extent_mapped += bytes;
                            result.extent_requested += local.requested;
                            result.extent_usable += local.live * stride;
                            if purge_extent_slots {
                                Span::purge_free_slots(
                                    span,
                                    &mut self.activity.extent_purge_calls,
                                    &mut self.activity.extent_purged_bytes,
                                );
                            }
                        } else {
                            result.slab_mapped += bytes;
                            result.slab_requested += local.requested;
                            result.slab_usable += local.live * stride;
                            if purge_slab_slots
                                && index == RECLAIM_SLAB_CLASS
                                && (*span).remote.bitmap_slab
                            {
                                Span::purge_free_slots(
                                    span,
                                    &mut self.activity.slab_purge_calls,
                                    &mut self.activity.slab_purged_bytes,
                                );
                            }
                        }
                        // The bitmap path can expose free slots even when the
                        // intrusive list is empty.
                        if class.active.is_null()
                            && (!local.head.is_null()
                                || ((extent
                                    || (index == RECLAIM_SLAB_CLASS
                                        && (*span).remote.bitmap_slab))
                                    && (*Span::span_slots(span)).bitmap
                                    && (*(*Span::span_slots(span)).owner_slots.get()).free != 0))
                        {
                            if extent && self.fullest_extents {
                                self.available_extents[index - NUM_CLASSES].set_bucket(span, 0);
                            }
                            class.active = span;
                        }
                    }
                    span = next;
                }
            }
        }
        result.slab_region_mapped = self.slab_regions.mapped;
        result.cached_extents = self.extent_pool.len * EXTENT_BYTES;
        result.activity = std::mem::take(&mut self.activity);
        result
    }

    fn retire(&mut self) {
        self.drop_cached_mappings();
        self.extent_pool.clear(&mut self.activity);
        self.extent_pool.mode = ExtentReuseMode::Off;
        self.demand = BinDemand::EMPTY;
        self.idle_since = None;
        self.used_large = false;
        self.extent_mode = ExtentMode::Disabled;
        self.fullest_extents = false;
        self.available_extents = [const { AvailableExtents::EMPTY }; EXTENT_BINS + 1];
        self.upper_extent = false;
        self.used_extents = false;
        self.extent_idle_since = None;
        self.use_slab_regions = false;
        self.reclaim_slab_pages = false;
        self.used_reclaim_slab = false;
        self.slab_idle_since = None;
        for class in &mut self.classes {
            let mut span = std::mem::replace(class, Class::EMPTY).head;
            while !span.is_null() {
                // SAFETY: Save the owner-local link before relinquishing
                // the span, since the last remote free can immediately unmap it.
                unsafe {
                    let next = (*Span::local(span)).next;
                    Span::retire(span);
                    span = next;
                }
            }
        }
        self.slab_regions.retire();
        self.owner = 0;
    }
}

fn new_mapping(len: usize) -> Mapping {
    let base = map(len);
    if !base.is_null() {
        add_mapping(len);
    }
    Mapping { base, len }
}

struct WorkerHeap(UnsafeCell<Heap>);

impl Drop for WorkerHeap {
    fn drop(&mut self) {
        self.0.get_mut().retire();
    }
}

thread_local! {
    static HEAP: WorkerHeap = const { WorkerHeap(UnsafeCell::new(Heap::EMPTY)) };
}

#[inline]
fn with_heap<T>(operation: impl FnOnce(&mut Heap) -> T) -> Option<T> {
    HEAP.try_with(|heap| {
        // SAFETY: Only this thread accesses the cell. Calls never invoke the
        // Rust global allocator, including during mapping and reclamation.
        operation(unsafe { &mut *heap.0.get() })
    })
    .ok()
}

/// Pipeline lifetime guard. Its teardown retires the worker's remaining spans.
pub struct PipelineScope(PhantomData<Rc<()>>);

impl Drop for PipelineScope {
    fn drop(&mut self) {
        let _ = with_heap(Heap::retire);
    }
}

/// Hooks the allocator into the lifetime of the pinned pipeline worker.
#[must_use]
pub fn enter_pipeline() -> PipelineScope {
    let config = LARGE_CACHE_CONFIG.load(Relaxed);
    let _ = with_heap(|heap| {
        heap.extent_mode = ExtentMode::from_config(config);
        heap.extent_pool.configure(config);
        heap.upper_extent = config & UPPER_EXTENT != 0;
        heap.use_slab_regions = config & SLAB_REGIONS != 0;
        heap.reclaim_slab_pages = config & RECLAIM_SLAB_PAGES != 0;
        heap.fullest_extents = config & FULLEST_EXTENTS != 0;
    });
    PipelineScope(PhantomData)
}

/// Allocator-accounted storage owned by the calling worker.
#[derive(Default)]
pub(crate) struct WorkerStats {
    pub(crate) slab_mapped: usize,
    pub(crate) slab_region_mapped: usize,
    pub(crate) slab_requested: usize,
    pub(crate) slab_usable: usize,
    pub(crate) empty_mapped: usize,
    pub(crate) extent_mapped: usize,
    pub(crate) cached_extents: usize,
    pub(crate) extent_requested: usize,
    pub(crate) extent_usable: usize,
    pub(crate) empty_extents: usize,
    pub(crate) remote_requested: usize,
    pub(crate) cached_large: usize,
    pub(crate) activity: MappingActivity,
}

/// Reconcile remote frees and sample the worker's allocator storage.
pub(crate) fn sample_worker() -> WorkerStats {
    with_heap(Heap::sample).unwrap_or_default()
}

pub(crate) fn node_activity_enabled() -> bool {
    LARGE_CACHE_CONFIG.load(Relaxed) & NODE_ACTIVITY != 0
}

/// Current, peak, and retired virtual mapping bytes across all threads.
#[must_use]
pub fn mapped_bytes() -> (u64, u64, u64) {
    (
        MAPPED.load(Relaxed) as u64,
        PEAK_MAPPED.load(Relaxed) as u64,
        RETIRED_MAPPED.load(Relaxed) as u64,
    )
}

fn large_layout(layout: Layout) -> Option<(usize, usize)> {
    let alignment = layout.align().max(align_of::<Mapping>());
    let len = layout
        .size()
        .checked_add(alignment - 1)?
        .checked_add(size_of::<Mapping>())?
        .checked_next_multiple_of(PAGE)?;
    if len > isize::MAX as usize {
        return None;
    }
    Some((alignment, len))
}

// SAFETY: The live allocation keeps the span header valid and its owner ID is
// immutable. Only that owner can access the local part of the span.
unsafe fn free_owned_block(span: *mut Span, block: *mut u8, requested: usize, slot_bit: u32) {
    let owner = unsafe { (*span).owner };
    let local = owner != 0
        && with_heap(|heap| {
            if heap.owner != owner {
                return false;
            }
            unsafe { heap.free_block(span, block, requested, slot_bit) };
            true
        })
        .unwrap_or(false);
    if !local {
        unsafe { Span::remote_free(span, block, requested, slot_bit) };
    }
}

unsafe fn resize_on_owner(span: *mut Span, old_size: usize, new_size: usize) -> bool {
    // SAFETY: The allocation keeps the span live. Access to the local state
    // is restricted to the matching worker.
    let owner = unsafe { (*span).owner };
    owner != 0
        && with_heap(|heap| {
            if heap.owner != owner {
                return false;
            }
            let local = unsafe { &mut *Span::local(span) };
            local.requested = local.requested - old_size + new_size;
            true
        })
        .unwrap_or(false)
}

/// Standalone Rust global allocator with worker-owned spans and Linux mappings.
pub struct PipelineAllocator;

impl PipelineAllocator {
    #[inline]
    unsafe fn allocate(&self, layout: Layout) -> *mut u8 {
        if let Some(class) = size_class(layout) {
            if let Some(block) = with_heap(|heap| heap.alloc_block(class, layout.size()).0) {
                return block;
            }
            // TLS is unavailable during teardown. Use a standalone retired
            // span with the same pointer classification as any other small block.
            let stride = CLASS_SIZES[class];
            let span = Span::create(
                0,
                class,
                stride,
                SPAN_BYTES,
                ExtentMode::Disabled,
                None,
                null_mut(),
            );
            if span.is_null() {
                return null_mut();
            }
            unsafe {
                let block = Span::take_block(span, class, layout.size());
                Span::retire(span);
                return block;
            }
        }
        let Some((alignment, len)) = large_layout(layout) else {
            return null_mut();
        };
        with_heap(|heap| {
            if heap.extent_mode != ExtentMode::Disabled
                && let Some(class) = extent_class(alignment, len, heap.upper_extent)
            {
                heap.used_extents = true;
                let (slot, span) = heap.alloc_block(class, layout.size());
                if !slot.is_null() {
                    // SAFETY: This slot's full size includes room for the
                    // record, alignment padding and requested payload.
                    return unsafe { Mapping::extent(span).write_before(slot, alignment) };
                }
            }
            heap.refresh_large_cache_config();
            let mapping = heap.take_large_mapping(len);
            // SAFETY: The live dedicated mapping is at least len bytes long.
            if mapping.base.is_null() {
                null_mut()
            } else {
                unsafe { mapping.write_before(mapping.base, alignment) }
            }
        })
        .unwrap_or_else(|| {
            let mapping = new_mapping(len);
            if mapping.base.is_null() {
                null_mut()
            } else {
                // SAFETY: The new dedicated mapping is len bytes long.
                unsafe { mapping.write_before(mapping.base, alignment) }
            }
        })
    }

    #[inline]
    unsafe fn deallocate(&self, block: *mut u8, layout: Layout) {
        if let Some(class) = size_class(layout) {
            let span = block
                .map_addr(|addr| addr & !(SPAN_BYTES - 1))
                .cast::<Span>();
            // SAFETY: Masking a live small block recovers its owning span.
            unsafe {
                let slot_bit = if class == RECLAIM_SLAB_CLASS && (*span).remote.bitmap_slab {
                    Span::slot_bit(span, block)
                } else {
                    0
                };
                free_owned_block(span, block, layout.size(), slot_bit);
            }
            return;
        }
        // SAFETY: Every non-small allocation initializes this aligned record.
        let mapping = unsafe { block.cast::<Mapping>().sub(1).read() };
        if mapping.is_extent() {
            // SAFETY: This live block and its prefix identify the extent and
            // the original slot to return to the owner's free list.
            unsafe {
                let span = mapping.base.cast();
                let slot = mapping.extent_slot(block);
                let slot_bit = Span::slot_bit(span, slot);
                free_owned_block(span, slot, layout.size(), slot_bit);
            };
        } else if with_heap(|heap| {
            heap.refresh_large_cache_config();
            heap.store_large_mapping(mapping);
        })
        .is_none()
        {
            unsafe { release_mapping(mapping.base, mapping.len) };
        }
    }
}

// SAFETY: Small blocks satisfy their class alignment and size and are routed
// by their aligned span base. Other blocks carry a mapping record that tags
// extent allocations. Only a thread with exclusive ownership of a block can
// free it. An owned span remains mapped through every outstanding block.
unsafe impl GlobalAlloc for PipelineAllocator {
    #[inline]
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        // SAFETY: The global allocator caller supplies a valid layout.
        let block = unsafe { self.allocate(layout) };
        if !block.is_null() {
            crate::node_allocation_metrics::allocated(layout.size());
        }
        block
    }

    #[inline]
    unsafe fn dealloc(&self, block: *mut u8, layout: Layout) {
        crate::node_allocation_metrics::freed(layout.size());
        // SAFETY: The caller owns this allocation and supplies its layout.
        unsafe { self.deallocate(block, layout) };
    }

    unsafe fn realloc(&self, block: *mut u8, old: Layout, size: usize) -> *mut u8 {
        // SAFETY: GlobalAlloc requires a valid new layout at the old alignment.
        let new = unsafe { Layout::from_size_align_unchecked(size, old.align()) };
        if let Some(index) = size_class(old) {
            if size_class(new) == Some(index) {
                let span = block
                    .map_addr(|addr| addr & !(SPAN_BYTES - 1))
                    .cast::<Span>();
                // SAFETY: The same small class still supports this layout.
                if unsafe { resize_on_owner(span, old.size(), size) } {
                    crate::node_allocation_metrics::reallocated(0);
                    return block;
                }
            }
        } else {
            // SAFETY: Every live non-small allocation stores this record.
            let mapping = unsafe { block.cast::<Mapping>().sub(1).read() };
            if mapping.is_extent() {
                let span = mapping.base.cast::<Span>();
                // SAFETY: The live block pins the extent and its immutable class.
                let class = unsafe { (*span).class };
                // Stay in the same class and keep the owner's requested-byte
                // accounting exact. A remote resize uses allocate-copy-free.
                if size_class(new).is_none()
                    && let Some((alignment, len)) = large_layout(new)
                    && extent_class(alignment, len, class == UPPER_EXTENT_CLASS) == Some(class)
                    && unsafe { resize_on_owner(span, old.size(), size) }
                {
                    crate::node_allocation_metrics::reallocated(0);
                    return block;
                }
            } else if size >= old.size()
                && LARGE_CACHE_CONFIG.load(Relaxed) & BINNED != 0
                && size_class(new).is_none()
            {
                let usable = mapping.len - (block.addr() - mapping.base.addr());
                if size <= usable {
                    let _ = with_heap(|heap| heap.activity.large_realloc_in_place += 1);
                    crate::node_allocation_metrics::reallocated(0);
                    return block;
                }
            }
        }
        // SAFETY: The old block stays live on failure. On success the two live
        // blocks are disjoint and the common prefix fits both layouts.
        let replacement = unsafe { self.allocate(new) };
        if !replacement.is_null() {
            unsafe {
                let copied = old.size().min(size);
                ptr::copy_nonoverlapping(block, replacement, copied);
                self.deallocate(block, old);
                crate::node_allocation_metrics::reallocated(copied);
                let _ = with_heap(|heap| heap.activity.realloc_copied += copied as u64);
            }
        }
        replacement
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct Block {
        ptr: *mut u8,
        layout: Layout,
    }

    // SAFETY: Ownership is unique and the test allocator accepts remote frees.
    unsafe impl Send for Block {}

    impl Block {
        fn new(size: usize, alignment: usize) -> Self {
            let layout = Layout::from_size_align(size, alignment).unwrap();
            // SAFETY: All test layouts are valid and nonzero.
            let ptr = unsafe { PipelineAllocator.alloc(layout) };
            assert!(!ptr.is_null());
            assert_eq!(ptr.addr() % alignment, 0);
            Self { ptr, layout }
        }
    }

    impl Drop for Block {
        fn drop(&mut self) {
            // SAFETY: This block owns the supplied allocation and layout.
            unsafe { PipelineAllocator.dealloc(self.ptr, self.layout) };
        }
    }

    fn enter_extent_worker() -> PipelineScope {
        enter_extent_worker_in(ExtentMode::Linked)
    }

    fn enter_extent_worker_in(mode: ExtentMode) -> PipelineScope {
        let scope = enter_pipeline();
        let _ = with_heap(|heap| heap.extent_mode = mode);
        scope
    }

    fn read_mapping(block: &Block) -> Mapping {
        // SAFETY: These tests pass only live non-small allocations.
        unsafe { block.ptr.cast::<Mapping>().sub(1).read() }
    }

    fn enter_slab_region_worker() -> PipelineScope {
        let scope = enter_pipeline();
        let _ = with_heap(|heap| heap.use_slab_regions = true);
        scope
    }

    fn enter_extent_reuse_worker(config: usize) -> PipelineScope {
        let scope = enter_extent_worker_in(ExtentMode::Purge);
        let _ = with_heap(|heap| {
            heap.upper_extent = true;
            heap.extent_pool.configure(config);
        });
        scope
    }

    /// Scenario: One class has an exhausted active extent, a nearer sparse
    /// extent with holes, and a fuller extent beyond the bounded search.
    /// Guarantees: Refill uses the fullest extent's last free slot first and
    /// then reuses an available sparse extent without mapping another one.
    #[test]
    fn bitmap_refill_prefers_the_fullest_extent_beyond_the_probe() {
        let _scope = enter_extent_reuse_worker(EXTENT_REUSE | EXTENT_REUSE_RESIDENT);
        let _ = with_heap(|heap| heap.fullest_extents = true);
        let size = 17 * 1024;
        let stride = 20 * 1024;
        let slots = (EXTENT_BYTES - PAGE) / stride;
        let mut blocks: Vec<Option<Block>> = (0..(PROBE_SPANS + 3) * slots)
            .map(|_| Some(Block::new(size, 1)))
            .collect();
        let fullest = read_mapping(blocks[0].as_ref().unwrap()).base;
        let sparse_start = (PROBE_SPANS + 1) * slots;
        let sparse = read_mapping(blocks[sparse_start].as_ref().unwrap()).base;
        let _ = blocks[0].take();
        for block in &mut blocks[sparse_start..sparse_start + slots - 1] {
            let _ = block.take();
        }
        let _ = sample_worker();
        let first = Block::new(size, 1);
        let second = Block::new(size, 1);
        assert_eq!(read_mapping(&first).base, fullest);
        assert_eq!(read_mapping(&second).base, sparse);
        assert_eq!(sample_worker().activity.extent_maps, 0);
    }

    /// Scenario: A worker samples remote frees that make one indexed extent
    /// sparser and fully empty another indexed extent.
    /// Guarantees: The first extent stays eligible, the second leaves the index,
    /// and a refill does not touch the emptied extent after it is pooled.
    #[test]
    fn bitmap_placement_reclassifies_remote_frees_before_reuse() {
        let _scope = enter_extent_reuse_worker(EXTENT_REUSE | EXTENT_REUSE_RESIDENT);
        let _ = with_heap(|heap| heap.fullest_extents = true);
        let size = 133_634;
        let mut blocks: Vec<_> = (0..12).map(|_| Some(Block::new(size, 1))).collect();
        let partial = read_mapping(blocks[0].as_ref().unwrap()).base;
        let kept = read_mapping(blocks[6].as_ref().unwrap()).base;
        drop((
            blocks[0].take(),
            blocks[3].take(),
            blocks[6].take(),
            blocks[7].take(),
            blocks[8].take(),
        ));
        let returned = (blocks[1].take(), blocks[4].take(), blocks[5].take());
        std::thread::spawn(move || drop(returned)).join().unwrap();
        let reconciled = sample_worker();
        assert_eq!(reconciled.remote_requested, 3 * size);
        assert_eq!(reconciled.empty_extents, EXTENT_BYTES);
        assert_eq!(reconciled.cached_extents, EXTENT_BYTES);
        let first = Block::new(size, 1);
        let second = Block::new(size, 1);
        let third = Block::new(size, 1);
        assert_eq!(read_mapping(&first).base, partial);
        assert_eq!(read_mapping(&second).base, partial);
        assert_eq!(read_mapping(&third).base, kept);
        drop((first, second, third, blocks));
        assert_eq!(sample_worker().activity.extent_maps, 0);
        assert_eq!(sample_worker().extent_mapped, 0);
    }

    /// Scenario: A worker releases more empty upper extents than its pool
    /// can hold, then allocates in a different extent class and becomes idle.
    /// Guarantees: The pool respects each configured cap, counts full returns,
    /// reassigns mappings across classes, and flushes at the first idle sample.
    #[test]
    fn extent_reuse_respects_budget_across_classes_and_flushes_when_idle() {
        let resident = EXTENT_REUSE | EXTENT_REUSE_RESIDENT;
        for (mode, configured_mib, expected_mib) in [
            (EXTENT_REUSE, 0, 2),
            (resident, 1, 1),
            (resident, 2, 2),
            (resident, 3, 3),
            (resident, 4, 4),
        ] {
            let _scope =
                enter_extent_reuse_worker(mode | (configured_mib << EXTENT_POOL_MIB_SHIFT));
            let pool_slots = expected_mib * MIB / EXTENT_BYTES;
            let extents = 11;
            let blocks: Vec<_> = (0..3 * extents).map(|_| Block::new(133_634, 1)).collect();
            let bases: Vec<_> = blocks.iter().map(|b| read_mapping(b).base).collect();
            for block in blocks {
                drop(block);
            }
            let full = sample_worker();
            let overflow = (extents - 1 - pool_slots) as u64;
            assert_eq!(full.activity.extent_maps, extents as u64);
            assert_eq!(full.activity.extent_releases, overflow);
            assert_eq!(full.activity.extent_reuse_full_returns, overflow);
            assert_eq!(full.cached_extents, expected_mib * MIB);
            assert_eq!(full.empty_extents, EXTENT_BYTES);

            let other_class = Block::new(32 * 1024, PAGE);
            let mapping = read_mapping(&other_class);
            assert!(mapping.is_extent());
            assert!(bases[3..3 * (pool_slots + 1)].contains(&mapping.base));
            let reused = sample_worker();
            assert_eq!(reused.activity.extent_maps, 0);
            assert_eq!(reused.activity.extent_reuse_hits, 1);
            assert_eq!(reused.cached_extents, (pool_slots - 1) * EXTENT_BYTES);
            // SAFETY: This byte is part of a live block owned by this thread.
            unsafe { other_class.ptr.write(42) };
            let idle = sample_worker();
            assert_eq!(idle.activity.extent_releases, pool_slots as u64);
            assert_eq!(idle.activity.extent_reuse_full_returns, 0);
            assert_eq!(idle.cached_extents, 0);
            assert_eq!(idle.extent_mapped, EXTENT_BYTES);
            // SAFETY: The block is still live and its first byte is initialized.
            assert_eq!(unsafe { other_class.ptr.read() }, 42);
            drop(other_class);
        }
    }

    /// Scenario: A worker retires with one live extent and one cached extent.
    /// Guarantees: Retirement unmaps the cache and the live block can still be
    /// read and freed on a different thread.
    #[test]
    fn extent_reuse_drops_cached_extents_on_retirement() {
        let scope = enter_extent_reuse_worker(EXTENT_REUSE | EXTENT_REUSE_RESIDENT);
        let mut blocks: Vec<_> = (0..9).map(|_| Block::new(133_634, 1)).collect();
        let live = blocks.pop().unwrap();
        drop(blocks);
        assert_eq!(sample_worker().cached_extents, EXTENT_BYTES);
        // SAFETY: The block is live and belongs to this thread.
        unsafe { live.ptr.write(13) };
        drop(scope);
        let retired = sample_worker();
        assert_eq!(retired.cached_extents, 0);
        assert_eq!(retired.activity.extent_releases, 1);
        std::thread::spawn(move || {
            // SAFETY: The retired extent stays mapped while this live block
            // owns its initialized first byte.
            assert_eq!(unsafe { live.ptr.read() }, 13);
            drop(live);
        })
        .join()
        .unwrap();
    }

    fn slab_of(block: &Block) -> *mut Span {
        block.ptr.map_addr(|addr| addr & !(SPAN_BYTES - 1)).cast()
    }

    /// Scenario: A worker enables 8 KiB slab reclaim while an original
    /// linked slab for that class still has live blocks.
    /// Guarantees: The active slab serves its returned and untouched slots
    /// in its original layout before refill creates a bitmap slab.
    #[test]
    fn active_slab_keeps_its_layout_when_reclaim_is_enabled() {
        let _scope = enter_slab_region_worker();
        let _ = with_heap(|heap| heap.reclaim_slab_pages = false);
        let first = Block::new(RECLAIM_SLAB_STRIDE, 8);
        let returned = Block::new(RECLAIM_SLAB_STRIDE, 8);
        let returned_address = returned.ptr.addr();
        drop(returned);

        let _ = with_heap(|heap| heap.reclaim_slab_pages = true);
        let reused = Block::new(RECLAIM_SLAB_STRIDE, 8);
        assert_eq!(reused.ptr.addr(), returned_address);
        let span = slab_of(&first);
        let rest: Vec<_> = (2..RECLAIM_SLAB_SLOTS)
            .map(|_| Block::new(RECLAIM_SLAB_STRIDE, 8))
            .collect();
        assert!(rest.iter().all(|block| slab_of(block) == span));
        let new = Block::new(RECLAIM_SLAB_STRIDE, 8);
        let new_span = slab_of(&new);
        assert_ne!(new_span, span);
        // SAFETY: Both slabs are pinned by their live blocks.
        unsafe {
            assert!(!(*span).remote.bitmap_slab);
            assert!((*new_span).remote.bitmap_slab);
        }
    }

    /// Scenario: A worker fills a region, maps a second, and releases a slab
    /// from the first region after retaining its class's one empty slab.
    /// Guarantees: The released slab can back a different size class without
    /// a new mapping and entirely empty regions are unmapped.
    #[test]
    fn slab_regions_reuse_released_slabs_across_classes() {
        let _scope = enter_slab_region_worker();
        let mut blocks: Vec<_> = (0..94).map(|_| Block::new(16 * 1024, 8)).collect();
        let returned = slab_of(&blocks[3]);
        let first = slab_of(&blocks[0]);
        let last = slab_of(&blocks[93]);
        assert_ne!(returned, first);
        // SAFETY: All three slabs are pinned by live blocks.
        unsafe {
            assert_eq!((*first).remote.region, (*returned).remote.region);
            assert_ne!((*first).remote.region, (*last).remote.region);
            assert_eq!((*first).remote.region.addr() % SLAB_REGION_BYTES, 0);
        }
        let full = sample_worker();
        assert_eq!(full.slab_region_mapped, 2 * SLAB_REGION_BYTES);
        assert_eq!(full.activity.slab_region_maps, 2);

        drop(blocks.drain(..6));
        let other_class = Block::new(8192, 64);
        assert_eq!(slab_of(&other_class), returned);
        let reused = sample_worker();
        assert_eq!(reused.activity.slab_region_maps, 0);

        drop(blocks);
        drop(other_class);
        let empty = sample_worker();
        assert_eq!(empty.slab_region_mapped, SLAB_REGION_BYTES);
        assert_eq!(empty.activity.slab_region_releases, 1);
        assert_eq!(empty.empty_mapped, 2 * SPAN_BYTES);
    }

    /// Scenario: Local and remote 8 KiB frees leave a partially occupied
    /// bitmap slab idle before the worker allocates from it again.
    /// Guarantees: Whole free slots can be purged without touching the
    /// adjacent live allocation, and the purged slots remain reusable.
    #[test]
    fn bitmap_slab_purges_locally_and_remotely_freed_pages() {
        let _scope = enter_slab_region_worker();
        let _ = with_heap(|heap| {
            heap.reclaim_slab_pages = true;
            heap.fullest_extents = true;
        });
        let mut blocks: Vec<_> = (0..RECLAIM_SLAB_SLOTS)
            .map(|_| Block::new(RECLAIM_SLAB_STRIDE, 8))
            .collect();
        let span = slab_of(&blocks[0]);
        assert!(blocks.iter().all(|block| slab_of(block) == span));
        // Fill another slab so the tested frees go through a nonactive span
        // even with fullest-first extent placement enabled.
        let _second_slab: Vec<_> = (0..RECLAIM_SLAB_SLOTS)
            .map(|_| Block::new(RECLAIM_SLAB_STRIDE, 8))
            .collect();
        // SAFETY: These live allocations each own a full slot.
        for block in &blocks {
            unsafe { block.ptr.write_bytes(37, RECLAIM_SLAB_STRIDE) };
        }
        let remote = blocks.remove(1);
        let local = blocks.remove(1);
        let first = remote.ptr.addr();
        let second = local.ptr.addr();
        std::thread::spawn(move || drop(remote)).join().unwrap();
        drop(local);
        let start = Instant::now();
        let sample = |now| with_heap(|heap| heap.sample_with_clock(|| now)).unwrap();
        assert_eq!(sample(start).remote_requested, RECLAIM_SLAB_STRIDE);
        assert_eq!(sample(start).activity.slab_purge_calls, 0);
        let idle = sample(start + EXTENT_PURGE_DELAY);
        assert_eq!(
            idle.activity.slab_purged_bytes,
            (2 * RECLAIM_SLAB_STRIDE) as u64
        );
        assert_eq!(idle.activity.slab_purge_calls, 1);
        assert_eq!(
            sample(start + EXTENT_PURGE_DELAY).activity.slab_purge_calls,
            0
        );

        let reused = Block::new(RECLAIM_SLAB_STRIDE, 8);
        let reused_next = Block::new(RECLAIM_SLAB_STRIDE, 8);
        assert_eq!(reused.ptr.addr(), first);
        assert_eq!(reused_next.ptr.addr(), second);
        // SAFETY: The other live slot was initialized above and remains owned.
        assert_eq!(unsafe { blocks[0].ptr.read() }, 37);
    }

    /// Scenario: The worker retires a region with blocks in two different
    /// slabs, and a different thread frees the first one.
    /// Guarantees: Each retired slab pins the region independently and
    /// releasing one leaves the other mapped and readable.
    #[test]
    fn slab_regions_keep_remote_blocks_alive_after_retirement() {
        let scope = enter_slab_region_worker();
        let first = Block::new(16 * 1024, 8);
        let last = Block::new(8192, 64);
        // SAFETY: The two live blocks pin their distinct slabs and region.
        let region = unsafe {
            assert_ne!(slab_of(&first), slab_of(&last));
            let region = (*slab_of(&last)).remote.region;
            assert!(!region.is_null());
            assert_eq!((*slab_of(&first)).remote.region, region);
            last.ptr.write(13);
            region
        };
        drop(scope);
        // SAFETY: Both retired slabs still have a live block.
        assert_eq!(unsafe { (*region).retired_refs.load(Acquire) }, 2);
        std::thread::spawn(move || drop(first)).join().unwrap();
        // SAFETY: Last still owns its slab and the first byte is initialized.
        unsafe {
            assert_eq!((*region).retired_refs.load(Acquire), 1);
            assert_eq!(last.ptr.read(), 13);
        }
        std::thread::spawn(move || drop(last)).join().unwrap();
    }

    /// Scenario: An opted-in worker allocates and grows buffers in the hot
    /// 160 KiB mapping bin while adjacent bins and over-aligned layouts are active.
    /// Guarantees: Only eligible layouts share three slots per extent. Resizing
    /// within that bin keeps the data in place and updates requested-byte accounting.
    #[test]
    fn upper_extent_pools_only_the_selected_bin() {
        let _scope = enter_extent_worker_in(ExtentMode::Purge);
        let size = 133_634;
        let dedicated = Block::new(size, 1);
        assert!(!read_mapping(&dedicated).is_extent());
        let _ = with_heap(|heap| heap.upper_extent = true);

        let mut first = Block::new(size, 1);
        let second = Block::new(size, PAGE);
        let third = Block::new(size, 8);
        let fourth = Block::new(size, 1);
        let base = read_mapping(&first).base;
        assert!(read_mapping(&first).is_extent());
        assert_eq!(read_mapping(&second).base, base);
        assert_eq!(read_mapping(&third).base, base);
        assert!(read_mapping(&fourth).is_extent());
        assert_ne!(read_mapping(&fourth).base, base);

        let below = Block::new(127 * 1024, 8);
        let above = Block::new(160 * 1024, 8);
        let over_aligned = Block::new(size, 2 * PAGE);
        assert!(!read_mapping(&below).is_extent());
        assert!(!read_mapping(&above).is_extent());
        assert!(!read_mapping(&over_aligned).is_extent());

        // SAFETY: The block is live with the exact recorded layout. The
        // enlarged layout fits the same slot and its first byte is initialized.
        unsafe {
            let original = first.ptr;
            first.ptr.write(42);
            let resized = PipelineAllocator.realloc(first.ptr, first.layout, size + 32);
            assert!(!resized.is_null());
            first.ptr = resized;
            first.layout = Layout::from_size_align(size + 32, 1).unwrap();
            assert_eq!(resized, original);
            assert_eq!(resized.read(), 42);
        }
        let stats = sample_worker();
        assert_eq!(stats.extent_requested, 4 * size + 32);
        assert_eq!(stats.extent_usable, 4 * UPPER_EXTENT_CAPACITY);
        assert_eq!(stats.extent_mapped, 2 * EXTENT_BYTES);
    }

    /// Scenario: twelve medium buffers of one class share an extent and
    /// another thread frees all but one of them.
    /// Guarantees: frees recover the original slots, reuse preserves page
    /// alignment, and an empty extent is unmapped once the worker idles.
    #[test]
    fn medium_extent_reuses_remote_slots_and_releases_when_idle() {
        for mode in [ExtentMode::Linked, ExtentMode::Bitmap] {
            check_extent_reuse_after_remote_free(mode);
        }
    }

    fn check_extent_reuse_after_remote_free(mode: ExtentMode) {
        let _scope = enter_extent_worker_in(mode);
        let anchor = Block::new(32 * 1024, PAGE);
        let blocks: Vec<_> = (0..11).map(|_| Block::new(32 * 1024, PAGE)).collect();
        let extent = read_mapping(&anchor);
        assert!(extent.is_extent());
        assert!(blocks.iter().all(|b| read_mapping(b).base == extent.base));
        let addresses: Vec<_> = blocks.iter().map(|b| b.ptr.addr()).collect();
        let initial = sample_worker();
        assert_eq!(initial.activity.extent_maps, 1);
        assert_eq!(initial.extent_mapped, EXTENT_BYTES);
        assert_eq!(initial.extent_requested, 12 * 32 * 1024);
        assert_eq!(initial.extent_usable, 12 * 40 * 1024);

        std::thread::spawn(move || drop(blocks)).join().unwrap();
        let reconciled = sample_worker();
        assert_eq!(reconciled.remote_requested, 11 * 32 * 1024);
        assert_eq!(reconciled.extent_requested, 32 * 1024);
        let reused = Block::new(32 * 1024, PAGE);
        assert!(addresses.contains(&reused.ptr.addr()));
        assert_eq!(read_mapping(&reused).base, extent.base);
        drop((anchor, reused));
        assert_eq!(sample_worker().empty_extents, EXTENT_BYTES);
        let idle = sample_worker();
        assert_eq!(idle.extent_mapped, 0);
        assert_eq!(idle.activity.extent_releases, 1);
    }

    /// Scenario: a dedicated allocation made outside a pipeline overlaps an
    /// extent-backed allocation whose layout changes while still owned.
    /// Guarantees: both layouts retain their routing and an extent resize
    /// within the same slot preserves its pointer and initialized data.
    #[test]
    fn medium_extent_routes_dedicated_and_resized_blocks() {
        let before_scope = Block::new(33 * 1024, 64);
        assert!(!read_mapping(&before_scope).is_extent());
        let _scope = enter_extent_worker();
        let mut block = Block::new(33 * 1024, 64);
        assert!(read_mapping(&block).is_extent());
        drop(before_scope);
        let old = block.layout;
        // SAFETY: The initialized bytes fit the old and both new layouts.
        unsafe {
            block.ptr.write_bytes(0x6d, old.size());
            let grown = PipelineAllocator.realloc(block.ptr, old, 36 * 1024);
            assert_eq!(grown, block.ptr);
            block.layout = Layout::from_size_align(36 * 1024, 64).unwrap();
            let dedicated = PipelineAllocator.realloc(grown, block.layout, 96 * 1024);
            assert!(!dedicated.is_null());
            block.ptr = dedicated;
            block.layout = Layout::from_size_align(96 * 1024, 64).unwrap();
            assert!(!read_mapping(&block).is_extent());
            assert!(
                std::slice::from_raw_parts(dedicated, old.size())
                    .iter()
                    .all(|byte| *byte == 0x6d)
            );
        }
    }

    /// Scenario: two extent-backed buffers outlive their allocating worker.
    /// Guarantees: retirement counts both outstanding slots and the first
    /// remote free preserves the other slot and its mapping.
    #[test]
    fn medium_extent_survives_owner_exit_until_last_remote_free() {
        for mode in [ExtentMode::Linked, ExtentMode::Bitmap] {
            check_retired_extent(mode);
        }
    }

    fn check_retired_extent(mode: ExtentMode) {
        let (first, last) = std::thread::spawn(move || {
            let _scope = enter_extent_worker_in(mode);
            (Block::new(40 * 1024, 64), Block::new(40 * 1024, 64))
        })
        .join()
        .unwrap();
        let extent = read_mapping(&last);
        assert!(extent.is_extent() && read_mapping(&first).base == extent.base);
        let span = extent.base.cast::<Span>();
        // SAFETY: The last block keeps the retired extent mapped.
        let remaining =
            || unsafe { Remote::with(ptr::addr_of!((*span).remote), |state| state.retired_live) };
        assert_eq!(remaining(), 2);
        drop(first);
        assert_eq!(remaining(), 1);
        drop(last);
    }

    /// Scenario: adjacent bitmap slots are freed locally and remotely while
    /// a third slot keeps the extent live through the idle-purge interval.
    /// Guarantees: the owner discards only those freed pages in one call,
    /// leaves the live bytes intact, and reuses a purged slot at its old address.
    #[test]
    fn bitmap_purges_only_reconciled_free_slots_after_idle() {
        let _scope = enter_extent_worker_in(ExtentMode::Purge);
        let size = 33 * 1024;
        let anchor = Block::new(size, 64);
        let first = Block::new(size, 64);
        let second = Block::new(size, 64);
        let freed = [first.ptr.addr(), second.ptr.addr()];
        // SAFETY: Each block is separately owned and has size writable bytes.
        unsafe {
            anchor.ptr.write_bytes(0x5a, size);
            first.ptr.write_bytes(0xaa, size);
            second.ptr.write_bytes(0xbb, size);
        }
        drop(first);
        std::thread::spawn(move || drop(second)).join().unwrap();

        let start = Instant::now();
        let sample = |time| with_heap(|heap| heap.sample_with_clock(|| time)).unwrap();
        let active = sample(start);
        assert_eq!(active.remote_requested, size);
        assert_eq!(active.activity.extent_purged_bytes, 0);
        assert_eq!(sample(start).activity.extent_purge_calls, 0);
        assert_eq!(
            sample(start + EXTENT_PURGE_DELAY / 2)
                .activity
                .extent_purge_calls,
            0
        );
        let purged = sample(start + EXTENT_PURGE_DELAY);
        assert_eq!(purged.extent_mapped, EXTENT_BYTES);
        assert_eq!(purged.extent_requested, size);
        assert_eq!(purged.activity.extent_purge_calls, 1);
        assert_eq!(purged.activity.extent_purged_bytes, 2 * 40 * 1024);
        assert_eq!(
            sample(start + 2 * EXTENT_PURGE_DELAY)
                .activity
                .extent_purge_calls,
            0
        );
        // SAFETY: The anchor remains live and its whole payload is initialized.
        assert!(
            unsafe { std::slice::from_raw_parts(anchor.ptr, size) }
                .iter()
                .all(|byte| *byte == 0x5a)
        );

        let reused = Block::new(size, 64);
        assert!(freed.contains(&reused.ptr.addr()));
        assert_eq!(read_mapping(&reused).base, read_mapping(&anchor).base);
    }

    /// Scenario: an active owner receives frees for multiple small blocks.
    /// Guarantees: reconciliation restores the slots and reports the exact
    /// live requested bytes, usable capacity and reclaimed requested bytes.
    #[test]
    fn reconciles_remote_frees_in_a_batch() {
        let _scope = enter_pipeline();
        let anchor = Block::new(42, 16);
        let blocks: Vec<_> = (0..100).map(|_| Block::new(42, 16)).collect();
        let initial = sample_worker();
        assert_eq!(initial.slab_mapped, SPAN_BYTES);
        assert_eq!(initial.slab_requested, 101 * 42);
        assert_eq!(initial.slab_usable, 101 * 48);
        std::thread::spawn(move || drop(blocks)).join().unwrap();
        let after = sample_worker();
        assert_eq!(after.remote_requested, 100 * 42);
        assert_eq!(after.slab_requested, 42);
        drop(anchor);
        assert_eq!(sample_worker().empty_mapped, SPAN_BYTES);
    }

    /// Scenario: two small blocks outlive their worker and its TLS heap.
    /// Guarantees: retirement records both live blocks, then a remote free
    /// leaves the other block readable until the last reference is dropped.
    #[test]
    fn remote_drop_after_owner_exit() {
        let (first, second) = std::thread::spawn(|| {
            let _scope = enter_pipeline();
            let first = Block::new(24, 16);
            let second = Block::new(24, 16);
            // SAFETY: This owned block has 24 writable bytes.
            unsafe { second.ptr.write_bytes(0x73, 24) };
            (first, second)
        })
        .join()
        .unwrap();
        let span = second
            .ptr
            .map_addr(|addr| addr & !(SPAN_BYTES - 1))
            .cast::<Span>();
        // SAFETY: Second remains live throughout, keeping the header valid.
        assert_eq!(
            unsafe { Remote::with(ptr::addr_of!((*span).remote), |state| state.retired_live) },
            2
        );
        drop(first);
        assert_eq!(
            unsafe { Remote::with(ptr::addr_of!((*span).remote), |state| state.retired_live) },
            1
        );
        assert_eq!(
            unsafe { std::slice::from_raw_parts(second.ptr, 24) },
            &[0x73; 24]
        );
        drop(second);
    }

    /// Scenario: remote workers return blocks while their owning heap retires
    /// with standalone or region-backed slabs.
    /// Guarantees: the racing frees complete across many spans, with each
    /// block surrendered exactly once.
    #[test]
    fn retirement_races_with_remote_free() {
        use std::sync::{Arc, Barrier};

        for iteration in 0..32 {
            let owner_scope = if iteration % 2 == 0 {
                enter_pipeline()
            } else {
                enter_slab_region_worker()
            };
            let gate = Arc::new(Barrier::new(5));
            let workers: Vec<_> = (0..4)
                .map(|_| {
                    let blocks: Vec<_> = (0..128).map(|_| Block::new(200, 64)).collect();
                    let gate = Arc::clone(&gate);
                    std::thread::spawn(move || {
                        let _ = gate.wait();
                        drop(blocks);
                    })
                })
                .collect();
            let _ = gate.wait();
            drop(owner_scope);
            for worker in workers {
                worker.join().unwrap();
            }
        }
    }

    /// Scenario: a highly aligned mapping is allocated, reused and grown.
    /// Guarantees: zeroed allocation clears retained bytes, growth preserves
    /// the original prefix and the large alignment survives reallocation.
    #[test]
    fn large_alignment_zeroing_and_growth() {
        let _scope = enter_pipeline();
        let layout = Layout::from_size_align(80 * 1024, 64 * 1024).unwrap();
        // SAFETY: Every checked block is uniquely owned and freed with its
        // layout. The zeroed call may reuse the first dirty mapping.
        unsafe {
            let dirty = PipelineAllocator.alloc(layout);
            assert!(!dirty.is_null());
            dirty.write_bytes(0xaa, layout.size());
            PipelineAllocator.dealloc(dirty, layout);
            let block = PipelineAllocator.alloc_zeroed(layout);
            assert!(!block.is_null());
            assert!(
                std::slice::from_raw_parts(block, layout.size())
                    .iter()
                    .all(|byte| *byte == 0)
            );
            block.write_bytes(0xbb, layout.size());
            let bigger = PipelineAllocator.realloc(block, layout, layout.size() * 3);
            assert!(!bigger.is_null());
            assert_eq!(bigger.addr() % layout.align(), 0);
            assert!(
                std::slice::from_raw_parts(bigger, layout.size())
                    .iter()
                    .all(|byte| *byte == 0xbb)
            );
            PipelineAllocator.dealloc(
                bigger,
                Layout::from_size_align(layout.size() * 3, layout.align()).unwrap(),
            );
        }
        assert!(sample_worker().cached_large <= MAP_CACHE_BYTES);
        assert_eq!(sample_worker().cached_large, 0);
    }

    /// Scenario: two distinct mapping sizes fit one capacity bin.
    /// Guarantees: the cache reuses the same mapping and counts one miss and
    /// one hit in the requested size band, then releases it when idle.
    #[test]
    fn reuses_larger_capacity_and_releases_when_idle() {
        let mut heap = Heap::EMPTY;
        let first = heap.take_large_mapping(72 * 1024);
        assert!(!first.base.is_null());
        heap.store_large_mapping(first);
        assert_eq!(heap.cached_bytes, 80 * 1024);
        let reused = heap.take_large_mapping(64 * 1024);
        assert_eq!(reused.base, first.base);
        assert_eq!(reused.len, first.len);
        heap.store_large_mapping(reused);
        let active = heap.sample();
        assert_eq!(active.activity.hits, [0, 1, 0]);
        assert_eq!(active.activity.misses, [0, 1, 0]);
        assert_eq!(active.activity.large_maps, 1);
        let idle = heap.sample();
        assert_eq!(idle.cached_large, 0);
        assert_eq!(idle.activity.large_releases, 1);
    }

    /// Scenario: a burst returns more mappings than the worker can cache.
    /// Guarantees: retained capacity stays within the budget and the cache
    /// can keep multiple mappings in the same size bin.
    #[test]
    fn evicts_mappings_to_enforce_budget() {
        let mut heap = Heap::EMPTY;
        let mappings = [(); 10].map(|()| heap.take_large_mapping(320 * 1024));
        assert!(mappings.iter().all(|mapping| !mapping.base.is_null()));
        for mapping in mappings {
            heap.store_large_mapping(mapping);
        }
        assert_eq!(heap.cached_bytes, 6 * 320 * 1024);
        assert_eq!(heap.activity.large_releases, 4);
        heap.drop_cached_mappings();
        assert_eq!(heap.cached_bytes, 0);
    }

    /// Scenario: reclaimed capacity for a busy bin competes with a cold bin.
    /// Guarantees: budget pressure evicts the cold mapping first and two
    /// samples without requests expire the busy bin's reservation.
    #[test]
    fn retains_recently_requested_bin_under_pressure() {
        let mut heap = Heap {
            cache_config: 128 * 1024 | BINNED | RESERVE_HOT_BINS,
            ..Heap::EMPTY
        };
        let hot = [(); 2].map(|()| heap.take_large_mapping(56 * 1024));
        let cold = heap.take_large_mapping(40 * 1024);
        let incoming = heap.take_large_mapping(28 * 1024);
        assert!(
            [hot[0], hot[1], cold, incoming]
                .iter()
                .all(|m| !m.base.is_null())
        );

        heap.store_large_mapping(hot[0]);
        heap.store_large_mapping(cold);
        heap.store_large_mapping(incoming);
        assert_eq!(heap.activity.large_releases, 0);
        let second_incoming = heap.take_large_mapping(20 * 1024);
        assert!(!second_incoming.base.is_null());
        heap.store_large_mapping(second_incoming);
        assert_eq!(heap.cached_bytes, (56 + 28 + 20) * 1024);
        assert_eq!(heap.activity.large_releases, 1);
        let reused = heap.take_large_mapping(56 * 1024);
        assert_eq!(reused.base, hot[0].base);
        heap.store_large_mapping(reused);
        heap.store_large_mapping(hot[1]);
        assert!(heap.cached_bytes <= heap.large_cache_budget());

        let hot_bit = 1 << capacity_bin(56 * 1024).unwrap();
        assert_ne!(heap.demand.hot & hot_bit, 0);
        heap.demand.age();
        assert_ne!(heap.demand.previous_hot & hot_bit, 0);
        heap.demand.age();
        assert_eq!(heap.demand.previous_hot & hot_bit, 0);
        heap.drop_cached_mappings();
    }

    /// Scenario: a burst leaves a populated binned cache before an idle period.
    /// Guarantees: the first quiet sample retains at most a quarter of the
    /// budget, reuse re-arms the timeout, and prolonged idle empties the cache.
    #[test]
    fn gradually_trims_and_releases_idle_mappings() {
        let mut heap = Heap {
            cache_config: MAP_CACHE_BYTES | BINNED | GRADUAL_TRIM,
            ..Heap::EMPTY
        };
        let mappings = [(); 12].map(|()| heap.take_large_mapping(56 * 1024));
        assert!(mappings.iter().all(|m| !m.base.is_null()));
        for mapping in mappings {
            heap.store_large_mapping(mapping);
        }
        let start = Instant::now();
        heap.trim_idle_cache(|| panic!("active sample does not read the clock"));
        assert_eq!(heap.cached_bytes, 12 * 56 * 1024);
        heap.trim_idle_cache(|| start);
        let kept = 9 * 56 * 1024;
        assert_eq!(heap.cached_bytes, kept);
        heap.trim_idle_cache(|| start + IDLE_KEEP_TIME / 2);
        assert_eq!(heap.cached_bytes, kept);

        let reused = heap.take_large_mapping(56 * 1024);
        assert_eq!(heap.activity.hits[0], 1);
        heap.store_large_mapping(reused);
        heap.trim_idle_cache(|| panic!("new activity re-arms the idle timer"));
        let second_idle = start + IDLE_KEEP_TIME;
        heap.trim_idle_cache(|| second_idle);
        assert_eq!(heap.cached_bytes, kept);
        heap.trim_idle_cache(|| second_idle + IDLE_KEEP_TIME);
        assert_eq!(heap.cached_bytes, 0);
        assert_eq!(heap.activity.large_releases, 12);
    }

    /// Scenario: a large block grows first within its mapping, then beyond it.
    /// Guarantees: the first growth preserves the pointer, the second
    /// preserves initialized bytes, and the two operations are counted.
    #[test]
    fn grows_large_mapping_in_place_when_capacity_allows() {
        let _scope = enter_pipeline();
        let first = Layout::from_size_align(70 * 1024, 64).unwrap();
        let middle = Layout::from_size_align(76 * 1024, 64).unwrap();
        let last = Layout::from_size_align(110 * 1024, 64).unwrap();
        // SAFETY: The test uniquely owns each live block and uses its current
        // layout. Only the initialized prefix is inspected.
        unsafe {
            let block = PipelineAllocator.alloc(first);
            assert!(!block.is_null());
            block.write_bytes(0x51, first.size());
            let grown = PipelineAllocator.realloc(block, first, middle.size());
            assert_eq!(grown, block);
            let replacement = PipelineAllocator.realloc(grown, middle, last.size());
            assert!(!replacement.is_null());
            assert_ne!(replacement, grown);
            assert!(
                std::slice::from_raw_parts(replacement, first.size())
                    .iter()
                    .all(|byte| *byte == 0x51)
            );
            PipelineAllocator.dealloc(replacement, last);
        }
        let stats = sample_worker();
        assert_eq!(stats.activity.large_realloc_in_place, 1);
        assert_eq!(stats.activity.realloc_copied, middle.size() as u64);
    }

    /// Scenario: many slabs of one class become empty after a burst.
    /// Guarantees: the worker keeps only one empty slab in that class.
    #[test]
    fn bounds_empty_spans_after_a_burst() {
        let _scope = enter_pipeline();
        let blocks: Vec<_> = (0..100).map(|_| Block::new(8192, 64)).collect();
        assert!(sample_worker().slab_mapped > SPAN_BYTES);
        drop(blocks);
        let after = sample_worker();
        assert_eq!(after.slab_mapped, SPAN_BYTES);
        assert_eq!(after.empty_mapped, SPAN_BYTES);
        assert_eq!(after.slab_requested, 0);
    }
}
