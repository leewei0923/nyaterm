import { invoke } from "./invoke";

export async function readClipboardText(): Promise<string> {
  const text = await invoke<string | null>("read_clipboard_text");
  return text ?? "";
}
