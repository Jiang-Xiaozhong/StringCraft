import type { AppConfig, TransformButton } from "./types";

export const DEFAULT_BUTTONS: TransformButton[] = [
  { id: "upper", label: "全大写", transform: "upper" },
  { id: "lower", label: "全小写", transform: "lower" },
  { id: "capitalize-words", label: "首字母大写", transform: "capitalize-words" },
  { id: "uncapitalize-words", label: "首字母小写", transform: "uncapitalize-words" },
  { id: "sentence-case", label: "句子首字母大写", transform: "sentence-case" },
  { id: "space-to-underscore", label: "空格→下划线", transform: "space-to-underscore" },
  { id: "to-camel", label: "下划线&空格→驼峰", transform: "to-camel" },
  { id: "camel-to-underscore", label: "驼峰→下划线", transform: "camel-to-underscore" },
  { id: "camel-to-space", label: "驼峰→空格", transform: "camel-to-space" },
  { id: "space-to-hyphen", label: "空格→中横线", transform: "space-to-hyphen" },
  { id: "underscore-to-hyphen", label: "下划线→中横线", transform: "underscore-to-hyphen" },
  { id: "hyphen-to-underscore", label: "中横线→下划线", transform: "hyphen-to-underscore" },
  { id: "underscore-to-space", label: "下划线→空格", transform: "underscore-to-space" },
  { id: "underscore-to-dot", label: "下划线→小数点", transform: "underscore-to-dot" },
  { id: "dot-to-underscore", label: "小数点→下划线", transform: "dot-to-underscore" },
  { id: "space-to-newline", label: "空格→换行", transform: "space-to-newline" },
  { id: "newline-to-space", label: "换行→空格", transform: "newline-to-space" },
  { id: "remove-symbols", label: "清除符号", transform: "remove-symbols" },
  { id: "remove-spaces", label: "清除空格", transform: "remove-spaces" },
  { id: "remove-newlines", label: "清除换行", transform: "remove-newlines" },
];

export const DEFAULT_CONFIG: AppConfig = {
  version: 1,
  hotkey: "Ctrl+Alt+Space",
  rows: 2,
  buttonWidth: 72,
  buttonHeight: 32,
  fontSize: 13,
  opacity: 100,
  theme: "system",
  autoStart: false,
  restoreClipboard: true,
  replaceDelayMs: 80,
  buttons: DEFAULT_BUTTONS,
};
