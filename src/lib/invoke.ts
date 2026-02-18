import { invoke as tauriInvoke } from "@tauri-apps/api/core";
import { logger } from "./logger";

/**
 * Typed wrapper around Tauri's `invoke()` with built-in error logging.
 *
 * Usage:
 *   const result = await invoke<string>("create_ssh_session", { config });
 *   const sessions = await invoke<SessionInfo[]>("list_sessions");
 */
export async function invoke<T>(cmd: string, args?: Record<string, unknown>): Promise<T> {
  try {
    return await tauriInvoke<T>(cmd, args);
  } catch (error) {
    logger.error(`Command "${cmd}" failed`, error);
    throw error;
  }
}
