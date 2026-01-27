# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Overview

This is the Rust SDK for Hegel, a universal property-based testing framework. The SDK communicates with a Python server (powered by Hypothesis) via Unix sockets to generate test data.

## Build & Test Commands

```bash
just test                           # cargo test
just format                         # cargo fmt
just docs                           # cargo doc --open --all-features
cargo test test_name                # Run single test
cargo test --all-features           # Run tests including optional features
```

## Crate Structure

```
hegel-rust/
├── src/
│   ├── lib.rs          # Public API: hegel(), Hegel builder, assume(), note()
│   ├── embedded.rs     # Spawns hegel CLI, manages socket server
│   └── gen/            # Generator implementations
│       ├── mod.rs      # Generate trait, socket communication, thread-local state
│       ├── primitives.rs   # unit(), booleans(), just(), just_any()
│       ├── numeric.rs      # integers(), floats() with bounds
│       ├── strings.rs      # text(), from_regex()
│       ├── formats.rs      # emails(), urls(), dates(), ip_addresses(), etc.
│       ├── collections.rs  # vecs(), hashsets(), hashmaps()
│       ├── tuples.rs       # tuples(), tuples3()
│       ├── combinators.rs  # one_of!(), optional(), sampled_from(), BoxedGenerator
│       ├── fixed_dict.rs   # fixed_dicts() for JSON objects
│       ├── default.rs      # DefaultGenerator trait implementations
│       ├── macros.rs       # one_of!(), derive_generator!() macros
│       ├── binary.rs       # binary() for Vec<u8> generation
│       ├── random.rs       # randoms() for RNG generation (requires "rand" feature)
│       └── value.rs        # HegelValue wrapper for NaN/Infinity handling
├── hegel-derive/       # Proc macro crate for #[derive(Generate)]
│   └── src/lib.rs      # Derives generators for structs and enums
└── build.rs            # Auto-installs hegel CLI via uv if not on PATH
```

## Feature Flags

- **`rand`**: Enables `gen::randoms()` for generating `rand::RngCore` implementations

## Environment Variables

- `HEGEL_DEBUG=1`: Log all socket requests/responses to stderr

## Architecture

### How It Works

The SDK spawns the `hegel` CLI as a subprocess with `--client-mode`. The test binary creates a Unix socket server, and hegel connects for each test case. The build script (`build.rs`) automatically installs Python and hegel into cargo's `OUT_DIR/hegel` via uv if not found on PATH.

### Protocol

Each test case follows this handshake:
1. Hegel connects to the SDK's socket
2. Hegel sends: `{"is_last_run": bool}` (is_last_run=true on final replay for output)
3. SDK responds: `{"type": "handshake_ack"}`
4. SDK runs test, communicating via `generate`/`start_span`/`stop_span` commands
5. SDK sends result: `{"type": "test_result", "result": "pass"|"fail"|"reject", ...}`

### Thread-Local State

The SDK uses thread-local storage for:
- `IS_LAST_RUN`: Whether this is the final replay for counterexample output
- `CONNECTION`: The active socket connection with span depth tracking

### Generation Protocol

Generators implement the `Generate<T>` trait:
- `schema()`: Returns JSON Schema describing generated values (enables single-request composition)
- `generate()`: Produces a value, either via schema or compositional fallback

When `schema()` returns `Some`, the SDK sends one request. When `None` (after `map`/`filter`), it falls back to multiple requests with span grouping for shrinking.

### Span System

Spans (`start_span`/`stop_span`) group related generation calls, helping Hypothesis understand data structure for effective shrinking. Labels in `gen::labels` identify span types (LIST, TUPLE, ONE_OF, etc.).

## Key Patterns

### Adding a New Generator

1. Create a builder struct with configuration fields
2. Implement `Generate<T>` with `schema()` and `generate()`
3. Export a factory function from `gen/mod.rs`
4. If the generated type should work with `#[derive(Generate)]`, implement `DefaultGenerator`

### Derive Macro

`#[derive(Generate)]` creates a `<Type>Generator` struct with:
- `new()`: Uses `DefaultGenerator` for all fields
- `with_<field>(gen)`: Builder methods to customize field generators

For enums, it also creates `<Enum><Variant>Generator` for each data variant.
