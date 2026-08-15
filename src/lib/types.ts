export interface TransformButton {
  id: string;
  name: string;
  description: string;
  transform: string;
  visible: boolean;
}

export interface WindowPosition {
  x: number;
  y: number;
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
  backgroundColor: string;
  backgroundColorDark: string;
  position?: WindowPosition | null;
  autoStart: boolean;
  restoreClipboard: boolean;
  replaceDelayMs: number;
  buttons: TransformButton[];
}
