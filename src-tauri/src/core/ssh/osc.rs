//! Shared OSC parsing, shell detection types, and injection script generation.
//!
//! Used by both SSH (`core::ssh::io`) and local PTY (`core::pty`) to avoid duplication.

/// Remote shell flavour detected via exec channel or local shell path.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShellKind {
    Bash,
    Zsh,
    Fish,
    PowerShell,
    PosixSh,
    Unknown,
}

impl ShellKind {
    /// Classify a shell name / path string (case-insensitive).
    pub fn from_name(name: &str) -> Self {
        let s = name.to_ascii_lowercase();
        if s.contains("fish") {
            Self::Fish
        } else if s.contains("zsh") {
            Self::Zsh
        } else if s.contains("bash") {
            Self::Bash
        } else if s.contains("powershell") || s.contains("pwsh") {
            Self::PowerShell
        } else if s.contains("sh") {
            Self::PosixSh
        } else {
            Self::Unknown
        }
    }
}

// ---------------------------------------------------------------------------
// Ready marker
// ---------------------------------------------------------------------------

/// Build a session-unique ready marker: `\x1b]7777;DflyReady:<id>\x07`.
pub fn build_ready_marker(session_id: &str) -> String {
    format!("\x1b]7777;DflyReady:{}\x07", session_id)
}

// ---------------------------------------------------------------------------
// Injection scripts (per shell)
// ---------------------------------------------------------------------------

/// Generate the shell-specific injection script that installs an OSC 7 hook
/// and emits the ready marker.  Returns `None` for shells we cannot inject
/// (plain POSIX sh, unknown).
pub fn injection_script(shell: ShellKind, ready_marker: &str) -> Option<String> {
    let ready_osc = ready_marker
        .replace('\x1b', "\\033")
        .replace('\x07', "\\007");

    match shell {
        ShellKind::Bash => Some(format!(
            concat!(
                " if [ -z \"${{DFLY_INJ:-}}\" ]; then export DFLY_INJ=1;",
                " __df_host(){{ hostname 2>/dev/null || printf localhost; }};",
                " __df_emit(){{ printf '\\033]7;file://%s%s\\007' \"$(__df_host)\" \"$PWD\"; }};",
                " case \"${{PROMPT_COMMAND-}}\" in (*__df_emit*) ;; (*)",
                " PROMPT_COMMAND=\"__df_emit${{PROMPT_COMMAND:+; $PROMPT_COMMAND}}\" ;; esac;",
                " fi;",
                " printf '{}' 2>/dev/null\n",
            ),
            ready_osc,
        )),

        ShellKind::Zsh => Some(format!(
            concat!(
                " if [ -z \"${{DFLY_INJ:-}}\" ]; then export DFLY_INJ=1;",
                " __df_host(){{ hostname 2>/dev/null || printf localhost; }};",
                " __df_emit(){{ printf '\\033]7;file://%s%s\\007' \"$(__df_host)\" \"$PWD\"; }};",
                " autoload -Uz add-zsh-hook 2>/dev/null || true;",
                " typeset -ga precmd_functions;",
                " [[ \" ${{precmd_functions[*]}} \" == *\" __df_emit \"* ]] || precmd_functions+=(__df_emit);",
                " fi;",
                " printf '{}' 2>/dev/null\n",
            ),
            ready_osc,
        )),

        ShellKind::Fish => Some(format!(
            concat!(
                " if not set -q DFLY_INJ;",
                " set -gx DFLY_INJ 1;",
                " function __df_emit --on-event fish_prompt;",
                " printf '\\033]7;file://%s%s\\007' (hostname) $PWD;",
                " end;",
                " end;",
                " printf '{}' 2>/dev/null\n",
            ),
            ready_osc,
        )),

        ShellKind::PowerShell => Some(format!(
            concat!(
                " if (-not $env:DFLY_INJ) {{ $env:DFLY_INJ='1';",
                " function prompt {{ $p = (pwd).ProviderPath;",
                " $h = [System.Net.Dns]::GetHostName();",
                " Write-Host -NoNewline \"`e]7;file://$h$p`a\";",
                " return \"PS $p> \" }} }};",
                " Write-Host -NoNewline \"`e]7777;DflyReady:{}`a\"\n",
            ),
            // PowerShell uses backtick-e for ESC; embed the raw session id.
            ready_marker
                .trim_start_matches('\x1b')
                .trim_start_matches("]7777;DflyReady:")
                .trim_end_matches('\x07'),
        )),

        ShellKind::PosixSh | ShellKind::Unknown => None,
    }
}

// ---------------------------------------------------------------------------
// Streaming OSC stripper
// ---------------------------------------------------------------------------

const MAX_OSC_BUF: usize = 64 * 1024;

/// Result returned by [`OscStripper::push`].
pub struct OscResult {
    /// Text safe to display in the terminal (all recognised OSC sequences removed).
    pub visible: String,
    /// CWD paths extracted from OSC 7 sequences in this chunk.
    pub cwd_paths: Vec<String>,
    /// Whether the ready marker was detected in this chunk.
    pub ready: bool,
}

/// Streaming parser that strips OSC 7 and DflyReady sequences from terminal
/// output, handling split packets and extracting CWD paths.
pub struct OscStripper {
    buf: String,
}

impl OscStripper {
    pub fn new(_ready_marker: &str) -> Self {
        Self {
            buf: String::new(),
        }
    }

    /// Feed a chunk of terminal output.  Returns visible text with OSC
    /// sequences stripped, any CWD paths found, and whether the ready
    /// marker appeared.
    pub fn push(&mut self, chunk: &str) -> OscResult {
        self.buf.push_str(chunk);

        // Safety valve: if the buffer is enormous without any ESC, just
        // flush everything as visible to avoid unbounded memory growth.
        if self.buf.len() > MAX_OSC_BUF && !self.buf.contains('\x1b') {
            return OscResult {
                visible: std::mem::take(&mut self.buf),
                cwd_paths: Vec::new(),
                ready: false,
            };
        }

        let mut visible = String::new();
        let mut paths = Vec::new();
        let mut ready = false;

        loop {
            let esc_pos = match self.buf.find("\x1b]") {
                Some(i) => i,
                None => {
                    // No ESC] left — everything is visible text.
                    visible.push_str(&self.buf);
                    self.buf.clear();
                    break;
                }
            };

            // Text before the ESC is always visible.
            visible.push_str(&self.buf[..esc_pos]);
            let rest = self.buf[esc_pos..].to_string();

            // Find the terminator: BEL (\x07) or ST (\x1b\\).
            let end = rest.find('\x07').map(|i| (i, 1)).or_else(|| {
                // Make sure we don't match the opening \x1b] as \x1b\\.
                rest[2..].find("\x1b\\").map(|i| (i + 2, 2))
            });

            let Some((end_idx, term_len)) = end else {
                // Incomplete sequence — keep in buffer for next chunk.
                self.buf = rest;

                // But if buffer is already huge, give up and flush.
                if self.buf.len() > MAX_OSC_BUF {
                    visible.push_str(&self.buf);
                    self.buf.clear();
                }
                break;
            };

            let seq = &rest[..end_idx + term_len];
            let inner = &rest[2..end_idx]; // between \x1b] and terminator

            if inner.starts_with("7;") {
                // OSC 7 — extract CWD path.
                if let Some(path) = parse_osc7_payload(&inner[2..]) {
                    paths.push(path);
                }
            } else if inner.starts_with("7777;DflyReady:") {
                ready = true;
            } else {
                // Not ours — pass through to the terminal.
                visible.push_str(seq);
            }

            self.buf = rest[end_idx + term_len..].to_string();
        }

        OscResult {
            visible,
            cwd_paths: paths,
            ready,
        }
    }

    /// Drain any buffered bytes as visible text (used on timeout / teardown).
    pub fn flush(&mut self) -> String {
        std::mem::take(&mut self.buf)
    }
}

/// Parse the payload of an OSC 7 sequence (`file://host/path`).
fn parse_osc7_payload(payload: &str) -> Option<String> {
    let after_scheme = payload.strip_prefix("file://")?;
    let path = if after_scheme.starts_with('/') {
        after_scheme.to_string()
    } else {
        let slash = after_scheme.find('/')?;
        after_scheme[slash..].to_string()
    };
    if path.is_empty() {
        None
    } else {
        Some(path)
    }
}
