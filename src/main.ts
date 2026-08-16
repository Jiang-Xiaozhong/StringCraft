import { mount } from "svelte";
import { getCurrentWindow } from "@tauri-apps/api/window";
import App from "./App.svelte";
import "./styles.css";

let windowLabel = "float-bar";
try {
  windowLabel = getCurrentWindow().label;
} catch {
  // 浏览器中直接打开时没有 Tauri 环境，回退到悬浮条
}

const app = mount(App, {
  target: document.getElementById("app")!,
  props: { windowLabel },
});

export default app;
