# Changelog

## [Unreleased]
### Fixed
- [TQ-1] Silent event emission failure: replaced `let _ = emit(...)` with `tracing::debug!` on error
### Added
- `tracing` dependency for emit error logging
