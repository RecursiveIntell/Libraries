# Changelog

## [Unreleased]
### Fixed
- [ABQ-2] `mark_running` split-lock: combined three separate scheduling `Mutex` fields into single `SchedulingState` struct, held atomically with jobs lock
