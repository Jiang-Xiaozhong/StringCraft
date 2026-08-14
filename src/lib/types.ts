export interface TransformButton {
  id: string;
  name: string;
  description: string;
  transform: string;
}

export interface AppConfig {
  version: number;
  hotkey: string;
  rows: number;
  buttonWidth: number;
  buttonHeight: number;
  fontSize: number;
  opacity: number;
  theme: "system" | "light" | "dark";
  autoStart: boolean;
  restoreClipboard: boolean;
  replaceDelayMs: number;
  buttons: TransformButton[];
}
