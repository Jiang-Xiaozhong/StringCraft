import { invoke } from "@tauri-apps/api/core";
import type { AppConfig } from "./types";

export async function getConfig(): Promise<AppConfig> {
  return invoke<AppConfig>("get_config");
}

export async function showSettingsWindow(): Promise<void> {
  await invoke("open_settings");
}

export async function executeButton(transformId: string): Promise<void> {
  await invoke("execute_button", { transformId });
}
