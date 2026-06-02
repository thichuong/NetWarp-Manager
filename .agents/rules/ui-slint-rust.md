---
trigger: always_on
glob: "**/*.slint, src/callbacks.rs, src/polling.rs"
description: Rules for maintaining synchronization between Slint UI elements/properties/callbacks and Rust backend logic (callbacks and polling loops) in NetWarp-Manager (WiWarp).
---

# UI Slint & Rust Synchronization Rules

When modifying, adding, or removing UI elements in the Slint definition files, you **must** strictly adhere to the following rules to prevent application desyncs, compilation errors, and run-time callback failures.

## 1. Rule of Synchronization Check
Whenever a `.slint` file is modified (defining new components, properties, or callbacks), you **MUST** immediately inspect and update:
- `src/callbacks.rs`: Registers the interactive handlers (event callbacks) from Slint UI to Rust backend functions and async tokio tasks.
- `src/polling.rs`: Runs background loops that periodically query state (speeds, network interface, WARP state, pings) and push updates back to Slint UI properties.

Ensure all new/modified properties or callbacks are correctly implemented in these two files.

## 2. Requirement to Read Inline Comments
- Before editing any `.slint` file, you **MUST** read the guiding developer comments placed at the top of that specific file. These comments outline which properties/callbacks are managed by Rust and how they synchronize.
- Ensure any added comments are maintained and kept up-to-date with your changes.

## 3. Reference Mapping
- **Callbacks in Slint**: Registered in `src/callbacks.rs` inside the `register_callbacks` function.
- **Properties in Slint**:
  - Updated dynamically in `src/polling.rs` inside `start_polling_loops`.
  - Updated within event handlers in `src/callbacks.rs`.
- **Structs in Slint (`ui/structs.slint`)**: Shared data structures. Any changes here require corresponding modifications in the Rust struct definitions in `src/main.rs` (or where the Slint code generator compiles them) and their mappings in `src/callbacks.rs` and `src/polling.rs`.

Always run `cargo check` and `cargo clippy` after making UI changes to verify that the generated Rust bindings and your implementation are in perfect synchronization.

## 4. Idiomatic Rust & Cost-Effective Cloning Rules

To maintain high performance and clean memory usage, all Rust code interacting with the Slint UI must follow these cloning and allocation rules:
- **Use `slint::SharedString` for State Buffers**: When storing local state variables (such as active SSIDs or WARP states in `src/polling.rs`) that are only used for comparison or UI updates, store them as `slint::SharedString` rather than `String`. Cloning a `SharedString` is extremely cheap (reference-counted, zero-cost allocation).
- **Prefer Borrowed Parameters (`&str`/`&[&str]`)**: Backend utility functions (e.g., in `src/wifi.rs`, `src/warp.rs`, `src/net_utils.rs`) should accept `&str` or slices `&[&str]` instead of owned `String` or `Vec<String>`. This avoids redundant `.clone()` operations on the caller side.
- **Pre-format Log Messages**: When logging details inside event handlers or async tasks, format the log message *before* constructing Slint data structures or moving parameters into closures. This allows moving values directly into the closure and completely avoids cloning.
- **Direct Move in Closures**: Move local variables directly into event loop closures where possible rather than cloning them, if they are not reused in the outer scope during that iteration.

