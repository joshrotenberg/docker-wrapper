# Clippy Fixes for Docker-Wrapper

This document outlines the systematic approach to fix clippy warnings found in the codebase. These fixes should be applied incrementally during Sprint 1 development.

## 🎯 Priority Levels

### High Priority (Fix Immediately)
- Missing `#[must_use]` on builder methods
- Redundant closures
- Format string inefficiencies

### Medium Priority (Fix During Development)
- Missing panic documentation
- Struct with excessive bools
- Inherent `to_string` implementations

### Low Priority (Address in Polish Phase)
- Must-use candidates on getters
- Minor style improvements

## 🔧 Specific Fixes Needed

### 1. Client Module (`src/client.rs`)

#### Fix 1: Add `#[must_use]` to builder methods
```rust
// BEFORE:
pub fn flag(mut self, flag: impl Into<String>) -> Self {

// AFTER:
#[must_use]
pub fn flag(mut self, flag: impl Into<String>) -> Self {
```

#### Fix 2: Simplify boolean logic
```rust
// BEFORE:
if !flag.starts_with('-') {
    self.args.push(format!("--{}", flag));
} else {
    self.args.push(flag);
}

// AFTER:
if flag.starts_with('-') {
    self.args.push(flag);
} else {
    self.args.push(format!("--{flag}"));
}
```

#### Fix 3: Replace inherent `to_string` with Display trait
```rust
// BEFORE:
pub fn to_string(&self) -> String {
    let mut parts = vec![format!("docker {}", self.subcommand)];
    parts.extend(self.args.iter().cloned());
    parts.join(" ")
}

// AFTER:
impl std::fmt::Display for CommandBuilder {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut parts = vec![format!("docker {}", self.subcommand)];
        parts.extend(self.args.iter().cloned());
        write!(f, "{}", parts.join(" "))
    }
}
```

### 2. Container Exec Module (`src/container/exec.rs`)

#### Fix 1: Add `#[must_use]` to all builder methods
```rust
// Add to all methods returning Self:
#[must_use]
pub fn working_dir(mut self, dir: impl Into<PathBuf>) -> Self { ... }

#[must_use]
pub fn env(mut self, key: impl Into<String>, value: impl Into<String>) -> Self { ... }

#[must_use]
pub fn user(mut self, user: impl Into<String>) -> Self { ... }

// etc.
```

#### Fix 2: Fix redundant closure
```rust
// BEFORE:
cmd_str.split_whitespace().map(|s| s.to_string()).collect();

// AFTER:
cmd_str.split_whitespace().map(std::string::ToString::to_string).collect();
```

#### Fix 3: Add panic documentation
```rust
/// Execute command with streaming output
/// 
/// # Panics
/// 
/// Panics if the child process stdout cannot be captured. This should not
/// happen under normal circumstances as stdout is always piped.
pub async fn exec_streaming<F>(
    &self,
    container_id: &ContainerId,
    config: ExecConfig,
    mut callback: F,
) -> DockerResult<ExecResult>
```

#### Fix 4: Refactor excessive bools in ExecConfig
```rust
// BEFORE:
pub struct ExecConfig {
    pub command: Vec<String>,
    pub working_dir: Option<PathBuf>,
    pub environment: HashMap<String, String>,
    pub user: Option<String>,
    pub attach_stdin: bool,
    pub attach_stdout: bool,
    pub attach_stderr: bool,
    pub tty: bool,
    pub privileged: bool,
    pub detached: bool,
    pub interactive: bool,
}

// AFTER:
#[derive(Debug, Clone)]
pub enum AttachMode {
    None,
    Stdout,
    Stderr,
    Both,
}

#[derive(Debug, Clone)]
pub enum ExecutionMode {
    Normal,
    Privileged,
    Interactive,
    Detached,
}

pub struct ExecConfig {
    pub command: Vec<String>,
    pub working_dir: Option<PathBuf>,
    pub environment: HashMap<String, String>,
    pub user: Option<String>,
    pub attach_mode: AttachMode,
    pub tty: bool,
    pub execution_mode: ExecutionMode,
    pub attach_stdin: bool,
}
```

### 3. General Pattern Fixes

#### Remove unnecessary borrows
```rust
// BEFORE:
let mut cmd = Command::new(&self.client.docker_path());

// AFTER:
let mut cmd = Command::new(self.client.docker_path());
```

#### Add `#[must_use]` to constructors and getters
```rust
#[must_use]
pub fn new(command: Vec<String>) -> Self { ... }

#[must_use]
pub fn is_success(&self) -> bool { ... }

#[must_use]
pub fn combined_output(&self) -> String { ... }
```

## 🔄 Implementation Strategy

### Phase 1: Critical Fixes (Day 1)
1. Fix format string issues in client.rs
2. Add `#[must_use]` to most commonly used builder methods
3. Fix redundant closures

### Phase 2: Structural Improvements (Day 2-3)
1. Refactor ExecConfig boolean overload
2. Replace inherent to_string with Display
3. Add panic documentation to risky methods

### Phase 3: Polish (During Development)
1. Add remaining `#[must_use]` attributes
2. Fix remaining style issues
3. Optimize performance-sensitive code

## 🧪 Testing Strategy

### Validation Steps
1. Run `cargo clippy --lib -- -D warnings` after each fix
2. Ensure all existing tests still pass
3. Verify no behavioral changes in API
4. Test that new enums work with existing code

### Regression Prevention
- Run full test suite after major structural changes
- Validate that builder patterns still work correctly
- Ensure ExecConfig refactoring doesn't break existing usage

## 📋 Implementation Checklist

### Client Module
- [ ] Add `#[must_use]` to `flag()` method
- [ ] Add `#[must_use]` to `option()` method  
- [ ] Add `#[must_use]` to `build()` method
- [ ] Fix boolean logic in flag handling
- [ ] Replace inherent `to_string` with Display
- [ ] Fix format string efficiency issues

### Container Exec Module
- [ ] Add `#[must_use]` to all builder methods
- [ ] Fix redundant closure in command parsing
- [ ] Add panic documentation to streaming methods
- [ ] Refactor ExecConfig excessive bools
- [ ] Add `#[must_use]` to constructors and getters

### General Fixes
- [ ] Remove unnecessary borrows throughout codebase
- [ ] Add missing `#[must_use]` attributes
- [ ] Fix any remaining format string issues
- [ ] Ensure all new code follows clippy guidelines

## 🎯 Success Criteria

### Metrics
- [ ] Zero clippy warnings with `-D warnings` flag
- [ ] All existing tests pass (100% pass rate maintained)
- [ ] No performance regression in test execution
- [ ] API compatibility maintained (no breaking changes)

### Quality Improvements
- [ ] Better API ergonomics with `#[must_use]` guidance
- [ ] Cleaner boolean handling in configuration structs
- [ ] More efficient string formatting
- [ ] Better error handling with panic documentation

---

**Note**: These fixes should be applied incrementally during Sprint 1 network testing development. Each fix should be tested immediately to ensure no regressions are introduced.