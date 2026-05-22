# MCP-RS Prompt Templates

This document provides reusable system prompts and instructions for configuring AI assistants (Cursor, Claude Desktop, etc.) to use MCP-RS as their trusted execution gateway.

---

## Table of Contents

1. [Core System Prompt](#core-system-prompt)
2. [Tool Catalog](#tool-catalog)
3. [Example Usage Patterns](#example-usage-patterns)
4. [Security Constraints](#security-constraints)
5. [Prompt Variants](#prompt-variants)
6. [Integration Examples](#integration-examples)

---

## Core System Prompt

Use this as the foundation for any AI system that should operate through MCP-RS.

### Full System Prompt

```
You are operating in an environment where MCP-RS is the single execution gateway. All actions must be performed through MCP tools. Natural language reasoning stays on your side; commands cross the boundary only as structured tool calls.

## Core Principle

AI = untrusted planner
MCP-RS = trusted executor

## Behavioral Rules

1. You may NOT read files directly — use the `read_file` tool
2. You may NOT list directories directly — use the `list_directory` tool
3. You may NOT run shell commands directly — no direct system access
4. You may NOT assume file contents or system state without tool confirmation
5. You may NOT bypass MCP tools for any file system or system operations

If a tool is not available for a required action, clearly state that you cannot perform it.

## Available Capabilities

All operations are performed through typed MCP tools with JSON input/output. Each tool call is:
- Validated against security policy before execution
- Logged to an audit trail for review
- Executed in a controlled, sandboxed manner

## When You Need Information

- To read a file: Use `read_file` with the path
- To list directory contents: Use `list_directory` with the path
- To search for patterns: Use `grep_file` (single file) or `grep_project` (multiple files)
- To find files by name: Use `find_files` with a glob pattern
- To check system info: Use `system_info`
- To run cargo commands: Use `cargo_check`, `cargo_test`, or `cargo_build`

Always verify information through tools before making assumptions or proceeding with recommendations.
```

### Compact System Prompt

For contexts with limited prompt space:

```
You operate through MCP-RS, a trusted execution gateway. All file and system operations must use MCP tools — no direct access allowed.

Rules:
- Use `read_file` to read files (never assume contents)
- Use `list_directory` to explore directories
- Use `grep_file`/`grep_project` to search code
- Use `find_files` to locate files by pattern
- Use cargo tools for Rust operations

If a tool doesn't exist for an action, state you cannot perform it. All tool calls are validated and logged.
```

---

## Tool Catalog

### File Operations

| Tool | Description | Required Parameters | Optional Parameters |
|------|-------------|---------------------|---------------------|
| `read_file` | Read contents of a file | `path` (string) | `start_line` (int), `end_line` (int) |
| `list_directory` | List files and directories | `path` (string) | `recursive` (bool), `include_hidden` (bool) |
| `file_stats` | Get file metadata | `path` (string) | — |
| `find_files` | Find files by glob pattern | `pattern` (string) | `base_path` (string), `max_results` (int) |
| `read_toml` | Parse and read TOML files | `path` (string) | `key` (string) |

### Search Operations

| Tool | Description | Required Parameters | Optional Parameters |
|------|-------------|---------------------|---------------------|
| `grep_file` | Search within a single file | `path` (string), `pattern` (string) | `case_sensitive` (bool), `context_lines` (int) |
| `grep_project` | Search across project files | `pattern` (string) | `glob` (string), `case_sensitive` (bool), `max_results` (int) |

### Cargo/Rust Development

| Tool | Description | Required Parameters | Optional Parameters |
|------|-------------|---------------------|---------------------|
| `cargo_check` | Run `cargo check` | — | `package` (string), `all_features` (bool) |
| `cargo_test` | Run `cargo test` | — | `test_name` (string), `package` (string) |
| `cargo_build` | Run `cargo build` | — | `release` (bool), `package` (string) |

### System Information

| Tool | Description | Required Parameters | Optional Parameters |
|------|-------------|---------------------|---------------------|
| `system_info` | Get OS, architecture, versions | — | — |
| `read_env` | Read environment variables | `name` (string) | — |
| `check_command` | Check if command exists | `command` (string) | — |

### Utility

| Tool | Description | Required Parameters | Optional Parameters |
|------|-------------|---------------------|---------------------|
| `health` | Check MCP-RS server status | — | `include_details` (bool) |
| `say_hello` | Simple greeting (testing) | `name` (string) | — |

---

## Example Usage Patterns

### Pattern 1: Reading and Understanding Code

**Goal**: Understand how a feature is implemented

```
Step 1: Use `list_directory` to see project structure
        → {"path": "src"}

Step 2: Use `find_files` to locate relevant files
        → {"pattern": "*.rs", "base_path": "src"}

Step 3: Use `grep_project` to find specific implementations
        → {"pattern": "fn handle_request", "glob": "**/*.rs"}

Step 4: Use `read_file` to examine the implementation
        → {"path": "src/server.rs", "start_line": 50, "end_line": 100}
```

### Pattern 2: Investigating Compilation Errors

**Goal**: Find and understand cargo check errors

```
Step 1: Use `cargo_check` to get diagnostics
        → {}

Step 2: Read the output, identify files with errors

Step 3: Use `read_file` to examine problematic code
        → {"path": "src/tool.rs"}

Step 4: Use `grep_file` to find related code
        → {"path": "src/tool.rs", "pattern": "impl Tool"}
```

### Pattern 3: Exploring Project Dependencies

**Goal**: Understand project dependencies and configuration

```
Step 1: Use `read_toml` to read Cargo.toml
        → {"path": "Cargo.toml"}

Step 2: Use `read_toml` to get specific sections
        → {"path": "Cargo.toml", "key": "dependencies"}

Step 3: Use `system_info` to check Rust version
        → {}

Step 4: Use `check_command` to verify toolchain
        → {"command": "cargo"}
```

### Pattern 4: Search and Replace Investigation

**Goal**: Find all usages of a function before modifying it

```
Step 1: Use `grep_project` to find all occurrences
        → {"pattern": "my_function\\(", "glob": "**/*.rs"}

Step 2: For each result, use `read_file` to get context
        → {"path": "src/main.rs", "start_line": 40, "end_line": 60}

Step 3: Use `file_stats` to understand file sizes
        → {"path": "src/main.rs"}
```

### Pattern 5: Running Tests and Validating Changes

**Goal**: Ensure code changes don't break tests

```
Step 1: Use `cargo_check` first (faster feedback)
        → {}

Step 2: If check passes, use `cargo_test` to run tests
        → {}

Step 3: If specific test fails, use `cargo_test` with filter
        → {"test_name": "test_read_file"}

Step 4: Use `read_file` to examine failing test
        → {"path": "src/tools/file_read.rs"}
```

---

## Security Constraints

### What MCP-RS Protects Against

1. **Path Traversal Attacks**
   - `../` sequences are detected and blocked
   - Paths are canonicalized before access
   - Symlink following is controlled

2. **Unauthorized File Access**
   - System directories (`/etc`, `/System`, `/usr/bin`) are blocked by default
   - Configurable allowlist and denylist
   - File size limits prevent memory exhaustion

3. **Command Injection**
   - No arbitrary shell command execution
   - Only specific cargo operations are permitted
   - Commands are executed with controlled arguments

4. **Information Disclosure**
   - Sensitive environment variables can be masked
   - File contents are not cached between sessions
   - Audit logs track all access attempts

### Policy-Enforced Boundaries

```
ALLOWED by default:
✓ Current working directory and subdirectories
✓ /tmp and /var/tmp directories
✓ Cargo operations (check, test, build)
✓ Reading environment variables

DENIED by default:
✗ System configuration (/etc/*)
✗ System binaries (/usr/bin/*, /System/*)
✗ User secrets (~/.ssh/*, ~/.gnupg/*)
✗ Arbitrary command execution
✗ Network operations (unless tool-specific)
```

### Audit Trail

Every tool call generates an audit log entry containing:
- Timestamp
- Tool name
- Input arguments
- Result (success/denied/error)
- Duration

Example audit entry:
```json
{
  "timestamp": "2024-01-15T10:30:00Z",
  "tool_name": "read_file",
  "arguments": {"path": "/etc/passwd"},
  "result": "PolicyDenied",
  "policy_checks": ["path_denied"],
  "duration_ms": 0
}
```

---

## Prompt Variants

### For Cursor IDE

Add to `.cursor/rules/` or include in project instructions:

```markdown
# MCP-RS Gateway Enforcement

All file and system operations must use MCP tools. You have access to:

- `read_file`, `list_directory`, `file_stats`, `find_files` for file operations
- `grep_file`, `grep_project` for searching
- `cargo_check`, `cargo_test`, `cargo_build` for Rust development
- `system_info`, `read_env`, `check_command` for system information
- `health` for server status

**Rules:**
1. Never assume file contents — always verify with tools
2. Use the most specific tool for each task
3. If a tool call fails, report the error clearly
4. All actions are logged — operate transparently
```

### For Claude Desktop

Add to your initial conversation or system instructions:

```markdown
I'm connected to an MCP-RS server that provides controlled access to the file system and development tools. 

When you need to:
- Read files → ask me to use `read_file`
- List directories → ask me to use `list_directory`
- Search code → ask me to use `grep_file` or `grep_project`
- Run cargo commands → ask me to use the cargo tools

I cannot directly access the file system or run arbitrary commands. All operations go through the MCP-RS gateway which validates and logs each request.

Please be explicit about which tool operations you'd like me to perform.
```

### For API/Programmatic Use

When constructing prompts programmatically:

```json
{
  "system": "You operate through MCP-RS, a trusted execution gateway. Use only the provided MCP tools for all file and system operations. Tool calls are validated against security policy and logged to an audit trail. Available tools: read_file, list_directory, grep_file, grep_project, file_stats, find_files, read_toml, cargo_check, cargo_test, cargo_build, system_info, read_env, check_command, health, say_hello.",
  "tools_instructions": "Always verify information with tools before making assumptions. Report tool errors clearly. Do not attempt operations without corresponding tools."
}
```

---

## Integration Examples

### Cursor Integration

**File: `.cursor/mcp.json`**
```json
{
  "mcpServers": {
    "mcp-rs": {
      "command": "/path/to/mcp-rs/target/release/mcp-rs",
      "env": {
        "RUST_LOG": "mcp_rs=info"
      }
    }
  }
}
```

**File: `.cursor/rules/mcp-gateway.mdc`**

See the full rules file in the project at `.cursor/rules/mcp-gateway.mdc`.

### Claude Desktop Integration

**File: `~/Library/Application Support/Claude/claude_desktop_config.json`**
```json
{
  "mcpServers": {
    "mcp-rs": {
      "command": "/path/to/mcp-rs/target/release/mcp-rs",
      "env": {
        "RUST_LOG": "mcp_rs=info"
      }
    }
  }
}
```

### Testing the Integration

After configuration, verify the integration works:

1. **List available tools**:
   ```bash
   echo '{"jsonrpc":"2.0","id":1,"method":"tools/list"}' | /path/to/mcp-rs
   ```

2. **Test a simple tool call**:
   ```bash
   echo '{"jsonrpc":"2.0","id":1,"method":"tools/call","params":{"name":"health","arguments":{}}}' | /path/to/mcp-rs
   ```

3. **Verify policy enforcement**:
   ```bash
   echo '{"jsonrpc":"2.0","id":1,"method":"tools/call","params":{"name":"read_file","arguments":{"path":"/etc/passwd"}}}' | /path/to/mcp-rs
   # Should return policy denied error
   ```

---

## Summary

The MCP-RS prompt templates establish a clear contract:

1. **AI reasons and plans** in natural language
2. **AI requests actions** via structured MCP tool calls
3. **MCP-RS validates** each request against policy
4. **MCP-RS executes** approved operations
5. **MCP-RS returns** structured results
6. **AI interprets** results and continues

This architecture provides:
- **Security**: All operations are validated and sandboxed
- **Auditability**: Complete trace of all AI actions
- **Reliability**: Structured I/O prevents hallucinated commands
- **Flexibility**: Easy to add new tools or modify policies

By using these prompt templates, you ensure consistent, secure AI behavior across all your development tools.
