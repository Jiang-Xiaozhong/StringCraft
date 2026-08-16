export interface MacaronColor {
  name: string;
  light: string;
  dark: string;
}

export const MACARON_COLORS: MacaronColor[] = [
  { name: "樱花粉", light: "#FCE4EC", dark: "#4A2B36" },
  { name: "蜜桃橙", light: "#FFE0CC", dark: "#4A3324" },
  { name: "奶油黄", light: "#FFF3C4", dark: "#4A4526" },
  { name: "抹茶绿", light: "#E2F0D9", dark: "#2F3D2B" },
  { name: "薄荷绿", light: "#D9F2E6", dark: "#2B3E35" },
  { name: "天空蓝", light: "#DCEBFA", dark: "#27384A" },
  { name: "雾霾蓝", light: "#E3E8F5", dark: "#2E3548" },
  { name: "薰衣草紫", light: "#EDE3F6", dark: "#3A3046" },
  { name: "玫瑰粉", light: "#F8DDEB", dark: "#452D3A" },
  { name: "燕麦灰", light: "#F1EDE8", dark: "#3B3733" },
];

export const MORANDI_COLORS: MacaronColor[] = [
  { name: "灰蓝", light: "#B7C4D1", dark: "#2B3642" },
  { name: "灰粉", light: "#D8B8BC", dark: "#3E2D30" },
  { name: "灰绿", light: "#B9C6B5", dark: "#2C3528" },
  { name: "灰紫", light: "#C9BBD6", dark: "#332B3D" },
  { name: "灰黄", light: "#D8CDB0", dark: "#3A3526" },
  { name: "灰橙", light: "#D6B3A0", dark: "#3B2B22" },
  { name: "灰棕", light: "#CBBBA9", dark: "#372F27" },
  { name: "灰青", light: "#AFC9C5", dark: "#283A38" },
  { name: "灰红", light: "#D4A6A0", dark: "#3D2826" },
  { name: "灰米", light: "#D8D2C6", dark: "#37332E" },
];

function clamp(value: number, min: number, max: number): number {
  return Math.min(max, Math.max(min, value));
}

function hexToRgb(hex: string): { r: number; g: number; b: number } {
  const normalized = hex.replace("#", "");
  const value = Number.parseInt(normalized, 16);
  return {
    r: (value >> 16) & 0xff,
    g: (value >> 8) & 0xff,
    b: value & 0xff,
  };
}

function rgbToHex(r: number, g: number, b: number): string {
  return `#${[r, g, b]
    .map((channel) => clamp(Math.round(channel), 0, 255).toString(16).padStart(2, "0"))
    .join("")
    .toUpperCase()}`;
}

export function hexToRgba(hex: string, alpha: number): string {
  const { r, g, b } = hexToRgb(hex);
  return `rgba(${r}, ${g}, ${b}, ${clamp(alpha, 0, 1)})`;
}

/**
 * 为自定义浅色生成一个对应的深色变体：
 * 转成 HSL 后压低亮度并降低一点饱和度，得到同色系深色背景。
 */
export function darkenHex(hex: string): string {
  const { r, g, b } = hexToRgb(hex);
  const rn = r / 255;
  const gn = g / 255;
  const bn = b / 255;
  const max = Math.max(rn, gn, bn);
  const min = Math.min(rn, gn, bn);
  const lightness = (max + min) / 2;
  const delta = max - min;

  let saturation = 0;
  if (delta !== 0) {
    saturation = delta / (1 - Math.abs(2 * lightness - 1));
  }

  const nextLightness = clamp(lightness * 0.42, 0.13, 0.3);
  const nextSaturation = clamp(saturation * 0.72, 0, 0.55);

  const c = (1 - Math.abs(2 * nextLightness - 1)) * nextSaturation;
  const hue = hueFromRgb(max, delta, rn, gn, bn);
  const x = c * (1 - Math.abs(((hue / 60) % 2) - 1));
  const m = nextLightness - c / 2;

  let rr = 0;
  let gg = 0;
  let bb = 0;
  if (hue < 60) {
    rr = c;
    gg = x;
  } else if (hue < 120) {
    rr = x;
    gg = c;
  } else if (hue < 180) {
    gg = c;
    bb = x;
  } else if (hue < 240) {
    gg = x;
    bb = c;
  } else if (hue < 300) {
    rr = x;
    bb = c;
  } else {
    rr = c;
    bb = x;
  }

  return rgbToHex((rr + m) * 255, (gg + m) * 255, (bb + m) * 255);
}

function hueFromRgb(
  max: number,
  delta: number,
  rn: number,
  gn: number,
  bn: number,
): number {
  if (delta === 0) return 0;
  let hue = 0;
  if (max === rn) {
    hue = 60 * (((gn - bn) / delta) % 6);
  } else if (max === gn) {
    hue = 60 * ((bn - rn) / delta + 2);
  } else {
    hue = 60 * ((rn - gn) / delta + 4);
  }
  return hue < 0 ? hue + 360 : hue;
}
