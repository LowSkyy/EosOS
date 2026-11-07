import { invoke } from "@tauri-apps/api/core";

/** Imposta `data-glass` sul documento: con "none" i token usano il fondo opaco. */
export async function initGlass(root: HTMLElement = document.documentElement) {
  try {
    const state = await invoke<string>("glass_state");
    root.setAttribute("data-glass", state);
  } catch {
    root.setAttribute("data-glass", "none");
  }
}
