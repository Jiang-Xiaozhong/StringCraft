import type { AppConfig, TransformButton } from "./types";

export const DEFAULT_BUTTONS: TransformButton[] = [
  { id: "upper", name: "全大写", transform: "upper", description: "所有字母转为大写" },
  { id: "lower", name: "全小写", transform: "lower", description: "所有字母转为小写" },
  { id: "capitalize-words", name: "首字母大写", transform: "capitalize-words", description: "每个单词首字母大写" },
  { id: "uncapitalize-words", name: "首字母小写", transform: "uncapitalize-words", description: "每个单词首字母小写" },
  { id: "sentence-case", name: "句子首字母大写", transform: "sentence-case", description: "每个句子首字母大写" },
  { id: "space-to-underscore", name: "空格→下划线", transform: "space-to-underscore", description: "空格替换为下划线" },
  { id: "to-camel", name: "下划线&空格→驼峰", transform: "to-camel", description: "下划线或空格分词并转为驼峰" },
  { id: "camel-to-underscore", name: "驼峰→下划线", transform: "camel-to-underscore", description: "驼峰分词并以下划线连接" },
  { id: "camel-to-space", name: "驼峰→空格", transform: "camel-to-space", description: "驼峰分词并以空格连接" },
  { id: "space-to-hyphen", name: "空格→中横线", transform: "space-to-hyphen", description: "空格替换为中横线" },
  { id: "underscore-to-hyphen", name: "下划线→中横线", transform: "underscore-to-hyphen", description: "下划线替换为中横线" },
  { id: "hyphen-to-underscore", name: "中横线→下划线", transform: "hyphen-to-underscore", description: "中横线替换为下划线" },
  { id: "underscore-to-space", name: "下划线→空格", transform: "underscore-to-space", description: "下划线替换为空格" },
  { id: "underscore-to-dot", name: "下划线→小数点", transform: "underscore-to-dot", description: "下划线替换为小数点" },
  { id: "dot-to-underscore", name: "小数点→下划线", transform: "dot-to-underscore", description: "小数点替换为下划线" },
  { id: "space-to-newline", name: "空格→换行", transform: "space-to-newline", description: "空格替换为换行" },
  { id: "newline-to-space", name: "换行→空格", transform: "newline-to-space", description: "换行替换为空格" },
  { id: "remove-symbols", name: "清除符号", transform: "remove-symbols", description: "删除字母、数字、空白以外的字符" },
  { id: "remove-spaces", name: "清除空格", transform: "remove-spaces", description: "删除所有空格字符" },
  { id: "remove-newlines", name: "清除换行", transform: "remove-newlines", description: "删除所有换行符" },
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
