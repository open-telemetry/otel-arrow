# Archived Design Documents

This directory contains historical design documents that have been superseded by implementation or consolidated into other documents.

## Archived Documents

### view-based-otlp-encoder-design.md
**Date**: December 17, 2025  
**Status**: Superseded by implementation  
**Reason**: Initial exploration of 2-byte padding optimization. We decided to use 4-byte padding for consistency with existing encoder. The view-based encoding approach was adopted, but the specific padding optimization was not.

### stateful-otlp-encoder-implementation.md
**Date**: December 17, 2025  
**Status**: Superseded by implementation  
**Reason**: Detailed implementation design document. Now that Phase 1 is complete, the actual code (`/crates/pdata/src/otlp/stateful_encoder.rs`) and the summary document (`../stateful-encoder-phase1-summary.md`) serve as the source of truth.

## Current Active Documents

See parent directory for current planning and implementation documentation:
- `custom-tracing-subscriber-plan.md` - Master plan
- `stateful-encoder-phase1-summary.md` - Phase 1 implementation summary
