import { invoke } from "@tauri-apps/api/core";
import type { AppConfig } from "./types";

export async function getConfig(): Promise<AppConfig> {
  return invoke<AppConfig>("get_config");
}

export async function saveConfig(config: AppConfig): Promise<AppConfig> {
  return invoke<AppConfig>("save_config", { config });
}

export async function applyNoActivate(): Promise<void> {
  await invoke("apply_no_activate");
}

export async function saveFloatBarPosition(x: number, y: number): Promise<void> {
  await invoke("update_float_bar_position", { x, y });
}

export async function saveFloatBarWidth(width: number): Promise<void> {
  await invoke("update_float_bar_width", { width });
}

export async function showSettingsWindow(): Promise<void> {
  await invoke("open_settings");
}

export async function executeButton(transformId: string): Promise<string> {
  return invoke<string>("execute_button", { transformId });
}
