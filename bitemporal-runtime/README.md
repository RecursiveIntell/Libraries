# bitemporal-runtime

Bitemporal truth primitives for append-supersede temporal data.

## Overview

This crate provides first-class support for bitemporal data modeling:

- **valid_time**: When a fact is true in the business domain
- **recorded_time**: When the system captured the fact
- **append-supersede**: Updates append new rows instead of mutating existing ones

## Key types

- `BitemporalRecord<T>` — a temporal record with `valid_time`, `recorded_time`, and domain value
- `SupersessionReceipt` — cryptographic receipt for supersession events
- `InMemoryDb` — in-memory store for testing

## Core functions

- `append_supersede()` — append a new record, emit receipts for superseded prior versions
- `as_of_query()` — query records valid at a given `valid_time` as of a given `recorded_time`
- `temporal_snapshot()` — retrieve full state as of a given `recorded_time`

## MSRV

Rust 1.75 (2021 edition).

## Dependencies

- `chrono` (serde)
- `serde` (derive)
- `sha2`
- `thiserror`