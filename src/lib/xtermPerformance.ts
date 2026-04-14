export const XTERM_PERFORMANCE_CONFIG = {
  highlighting: {
    /** Debounce delay in ms before re-scanning the visible viewport. */
    debounceMs: 80,
  },
  output: {
    /** Max characters to write into xterm in a single call. */
    writeChunkChars: 32 * 1024,
    /** Max synchronous work budget per animation frame. */
    frameBudgetMs: 8,
    /** Queue cap while the terminal is visible. */
    visibleBacklogCap: 1_000_000,
    /** Queue cap while the terminal is hidden. */
    hiddenBacklogCap: 250_000,
    /** Recovery threshold after overload while visible. */
    visibleRecoveryThreshold: 200_000,
    /** Recovery threshold after overload while hidden. */
    hiddenRecoveryThreshold: 50_000,
    /** How long to keep the recovery notice visible. */
    recoveryNoticeMs: 3_000,
  },
} as const;
