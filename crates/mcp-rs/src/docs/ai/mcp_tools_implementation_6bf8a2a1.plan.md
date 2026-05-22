---
name: MCP Tools Implementation
overview: Implement AI-callable tools for MCP-RS in 4 testable phases, starting with foundational file access tools and progressing to development workflow and system inspection tools.
todos:
  - id: phase-1-file-tools
    content: "Phase 1: Implement read_file, list_directory, grep_file tools"
    status: completed
  - id: phase-2-search-tools
    content: "Phase 2: Implement grep_project, file_stats, find_files tools"
    status: completed
  - id: phase-3-cargo-tools
    content: "Phase 3: Implement cargo_check, cargo_test, cargo_build tools"
    status: completed
  - id: phase-4-system-tools
    content: "Phase 4: Implement read_env, check_command, system_info, read_toml tools"
    status: completed
  - id: integration-tests
    content: Create comprehensive test script for all tools
    status: completed
isProject: false
---

# MCP-RS AI Tools Implementation Plan

## Current State

- Single `say_hello` tool exists in [src/tools/hello.rs](src/tools/hello.rs)
- Basic Tool trait with Input/Output types in [src/tool.rs](src/tool.rs)
- Dependencies: serde, serde_json, thiserror

## AI Review Key Insights

From [src/docs/ai/mcp-review-readme.md](src/docs/ai/mcp-review-readme.md):

- File access is "the most important MCP tool class"
- Prefer enums over free strings for AI-friendliness
- Explicit error variants over sentinel values
- Deterministic, grounded outputs enable AI reasoning

---

## Phase 1: Foundational File Access Tools

**Goal**: Enable AI to safely read and search files (grounded reasoning)

### Tools to Implement

**1.1 `read_file**` - Read file contents with optional line range

- Input: `path: String`, `start_line: Option<u32>`, `end_line: Option<u32>`
- Output: `success: bool`, `content: Option<String>`, `line_count: Option<u32>`, `error: Option<String>`

**1.2 `list_directory**` - List directory contents

- Input: `path: String`, `recursive: bool`, `pattern: Option<String>`
- Output: `entries: Vec<Entry>`, `error: Option<String>`
- Entry: `name`, `path`, `is_dir`, `size`

**1.3 `grep_file**` - Search within a single file

- Input: `path: String`, `pattern: String`, `case_sensitive: bool`
- Output: `matches: Vec<Match>`, `total_matches: u32`
- Match: `line_number`, `content`, `column`

### Files to Create/Modify

- `src/tools/file_read.rs` - read_file tool
- `src/tools/list_dir.rs` - list_directory tool
- `src/tools/grep.rs` - grep_file tool
- `src/main.rs` - register new tools

### Testing

```bash
# Test read_file
echo '{"jsonrpc":"2.0","id":1,"method":"tools/call","params":{"name":"read_file","arguments":{"path":"Cargo.toml"}}}' | cargo run

# Test list_directory
echo '{"jsonrpc":"2.0","id":2,"method":"tools/call","params":{"name":"list_directory","arguments":{"path":"src","recursive":false}}}' | cargo run

# Test grep_file
echo '{"jsonrpc":"2.0","id":3,"method":"tools/call","params":{"name":"grep_file","arguments":{"path":"src/main.rs","pattern":"registry","case_sensitive":false}}}' | cargo run
```

---

## Phase 2: Project Search and Analysis Tools

**Goal**: Enable AI to search across codebase and understand structure

### Tools to Implement

**2.1 `grep_project**` - Search across multiple files

- Input: `pattern: String`, `path: Option<String>`, `include_glob: Option<String>`, `exclude_glob: Option<String>`, `max_results: Option<u32>`
- Output: `matches: Vec<FileMatch>`, `files_searched: u32`, `truncated: bool`

**2.2 `file_stats**` - Get file/directory statistics

- Input: `path: String`
- Output: `exists: bool`, `is_file: bool`, `is_dir: bool`, `size: Option<u64>`, `modified: Option<String>`

**2.3 `find_files**` - Find files by name pattern

- Input: `pattern: String`, `path: Option<String>`, `max_depth: Option<u32>`
- Output: `files: Vec<String>`, `count: u32`

### Dependencies to Add

```toml
glob = "0.3"
walkdir = "2.4"
regex = "1.10"
```

### Testing

```bash
# Test grep_project
echo '{"jsonrpc":"2.0","id":1,"method":"tools/call","params":{"name":"grep_project","arguments":{"pattern":"impl Tool","include_glob":"*.rs"}}}' | cargo run

# Test find_files
echo '{"jsonrpc":"2.0","id":2,"method":"tools/call","params":{"name":"find_files","arguments":{"pattern":"*.rs","path":"src"}}}' | cargo run
```

---

## Phase 3: Development Workflow Tools

**Goal**: Enable AI to validate code and run builds

### Tools to Implement

**3.1 `cargo_check**` - Run cargo check

- Input: `path: Option<String>`, `package: Option<String>`
- Output: `success: bool`, `errors: Vec<Diagnostic>`, `warnings: Vec<Diagnostic>`
- Diagnostic: `message`, `file`, `line`, `column`, `level` (enum: Error, Warning)

**3.2 `cargo_test**` - Run cargo test

- Input: `test_name: Option<String>`, `package: Option<String>`, `no_capture: bool`
- Output: `success: bool`, `passed: u32`, `failed: u32`, `ignored: u32`, `failures: Vec<TestFailure>`

**3.3 `cargo_build**` - Run cargo build

- Input: `release: bool`, `package: Option<String>`
- Output: `success: bool`, `errors: Vec<Diagnostic>`, `artifacts: Vec<String>`

### Implementation Notes

- Execute cargo commands via `std::process::Command`
- Parse JSON output from cargo (`--message-format=json`)
- Set timeouts to prevent hanging

### Testing

```bash
# Test cargo_check
echo '{"jsonrpc":"2.0","id":1,"method":"tools/call","params":{"name":"cargo_check","arguments":{}}}' | cargo run

# Test cargo_test (with specific test)
echo '{"jsonrpc":"2.0","id":2,"method":"tools/call","params":{"name":"cargo_test","arguments":{"test_name":"test_hello"}}}' | cargo run
```

---

## Phase 4: System Inspection Tools

**Goal**: Enable AI to inspect environment and configurations

### Tools to Implement

**4.1 `read_env**` - Read environment variables

- Input: `name: Option<String>`, `prefix: Option<String>`
- Output: `variables: Vec<EnvVar>`, `count: u32`
- EnvVar: `name`, `value` (masked if sensitive)

**4.2 `check_command**` - Check if command exists

- Input: `command: String`
- Output: `exists: bool`, `path: Option<String>`, `version: Option<String>`

**4.3 `system_info**` - Get system information

- Input: (none)
- Output: `os`, `arch`, `rust_version`, `cargo_version`, `cwd`

**4.4 `read_toml**` - Parse and query TOML files

- Input: `path: String`, `query: Option<String>`
- Output: `success: bool`, `data: Option<Value>`, `error: Option<String>`

### Dependencies to Add

```toml
toml = "0.8"
which = "6.0"
```

### Testing

```bash
# Test read_env
echo '{"jsonrpc":"2.0","id":1,"method":"tools/call","params":{"name":"read_env","arguments":{"prefix":"CARGO"}}}' | cargo run

# Test check_command
echo '{"jsonrpc":"2.0","id":2,"method":"tools/call","params":{"name":"check_command","arguments":{"command":"rustc"}}}' | cargo run

# Test read_toml
echo '{"jsonrpc":"2.0","id":3,"method":"tools/call","params":{"name":"read_toml","arguments":{"path":"Cargo.toml","query":"package.name"}}}' | cargo run
```

---

## Final File Structure

```
src/
├── main.rs
├── protocol.rs
├── registry.rs
├── server.rs
├── tool.rs
└── tools/
    ├── mod.rs           # Tool module exports
    ├── hello.rs         # (existing)
    ├── file_read.rs     # Phase 1
    ├── list_dir.rs      # Phase 1
    ├── grep.rs          # Phase 1
    ├── grep_project.rs  # Phase 2
    ├── file_stats.rs    # Phase 2
    ├── find_files.rs    # Phase 2
    ├── cargo_check.rs   # Phase 3
    ├── cargo_test.rs    # Phase 3
    ├── cargo_build.rs   # Phase 3
    ├── read_env.rs      # Phase 4
    ├── check_command.rs # Phase 4
    ├── system_info.rs   # Phase 4
    └── read_toml.rs     # Phase 4
```

---

## AI-Friendly Design Guidelines (Applied Throughout)

1. **Use enums for operation types** - e.g., `DiagnosticLevel::Error | Warning`
2. **Explicit error variants** - Always include `error: Option<String>` field
3. **Structured outputs** - Avoid raw strings, use typed structs
4. **Deterministic behavior** - Same input always produces same output
5. **Safe defaults** - Limit max results, timeout long operations
6. **Rich schemas** - Include descriptions, enums, and constraints

---

## Integration Test Script

Create `test_all_tools.sh` after each phase:

```bash
#!/bin/bash
set -e

BINARY="./target/release/mcp-rs"
cargo build --release

echo "=== Testing Phase 1 Tools ==="
# ... tests for file access tools

echo "=== Testing Phase 2 Tools ==="
# ... tests for search tools

# etc.
```

---

## Dependencies Summary (Cargo.toml additions)

```toml
# Phase 2
glob = "0.3"
walkdir = "2.4"
regex = "1.10"

# Phase 4
toml = "0.8"
which = "6.0"
```

