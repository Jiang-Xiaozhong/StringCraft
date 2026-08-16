import { invoke } from "@tauri-apps/api/core";
import type { AppConfig } from "./types";

export async function getConfig(): Promise<AppConfig> {
  return invoke<AppConfig>("get_config");
}

export async function takeStartupNotice(): Promise<string | null> {
  return invoke<string | null>("take_startup_notice");
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

export async function exportConfigTo(path: string): Promise<string> {
  return invoke<string>("export_config_to", { path });
}

export async function importConfigFrom(path: string): Promise<string> {
  return invoke<string>("import_config_from", { path });
}

export interface UpdateInfo {
  latest: boolean;
  version?: string | null;
  notes?: string | null;
  url?: string | null;
  assetUrl?: string | null;
}

export async function checkForUpdate(): Promise<UpdateInfo> {
  return invoke<UpdateInfo>("check_for_update");
}

export async function downloadUpdate(assetUrl: string): Promise<string> {
  return invoke<string>("download_update", { assetUrl });
}

export async function installUpdate(path: string): Promise<string> {
  return invoke<string>("install_update", { path });
}

export async function showSettingsWindow(): Promise<void> {
  await invoke("open_settings");
}

export async function executeButton(
  transformId: string,
  custom?: {
    customType?: string | null;
    param1?: string | null;
    param2?: string | null;
  },
): Promise<string> {
  return invoke<string>("execute_button", {
    transformId,
    customType: custom?.customType ?? null,
    param1: custom?.param1 ?? null,
    param2: custom?.param2 ?? null,
  });
}
