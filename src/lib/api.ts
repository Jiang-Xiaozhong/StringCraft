import { invoke } from "@tauri-apps/api/core";
import type { AppConfig } from "./types";

export async function getConfig(): Promise<AppConfig> {
  return invoke<AppConfig>("get_config");
}

export async function saveConfig(config: AppConfig): Promise<AppConfig> {
  return invoke<AppConfig>("save_config", { config });
}

export async function showSettingsWindow(): Promise<void> {
  await invoke("open_settings");
}

export async function executeButton(transformId: string): Promise<string> {
  return invoke<string>("execute_button", { transformId });
}
