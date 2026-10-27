//! Acan's worker harness. A worker takes one task from an origin over MCP: it reads the task
//! back, claims it, declares its footprint, waits until the origin clears it to build, edits its
//! worktree through a tool-calling model, commits, and reports the build.
//!
//! The origin decides every rule that matters: which readbacks match, which clashes block a
//! build, and which generation a write must carry. The harness sequences the calls, renews the
//! lease while it works, and keeps its own writes behind the origin's build clearance.

mod grounding;
mod mcp;
mod tools;
mod worker;

pub use mcp::{McpClient, Reply};
pub use tools::{Worktree, normalize_path};
pub use worker::{Exit, Outcome, WorkerConfig, run};
