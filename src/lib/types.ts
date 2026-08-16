export interface TransformButton {
  id: string;
  name: string;
  description: string;
  transform: string;
  visible: boolean;
  customType?: string | null;
  param1?: string | null;
  param2?: string | null;
}

export interface WindowPosition {
  x: number;
  y: number;
}

export interface AppConfig {
  version: number;
  hotkey: string;
  toolbarWidth: number;
  buttonWidth: number;
  buttonHeight: number;
  fontSize: number;
  opacity: number;
  theme: "system" | "light" | "dark";
  backgroundColor: string;
  backgroundColorDark: string;
  position?: WindowPosition | null;
  autoStart: boolean;
  restoreClipboard: boolean;
  replaceDelayMs: number;
  debugLog: boolean;
  language: "zh-CN" | "en-US";
  autoCheckUpdate: boolean;
  autoUpdate: boolean;
  showDonation: boolean;
  buttons: TransformButton[];
}
