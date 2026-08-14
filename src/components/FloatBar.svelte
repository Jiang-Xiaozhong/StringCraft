<script lang="ts">
  import { onDestroy, onMount } from "svelte";
  import { listen } from "@tauri-apps/api/event";
  import { currentMonitor, getCurrentWindow, LogicalSize, PhysicalPosition } from "@tauri-apps/api/window";
  import { getConfig, showSettingsWindow, executeButton } from "../lib/api";
  import { DEFAULT_CONFIG } from "../lib/defaults";
  import type { AppConfig, TransformButton } from "../lib/types";

  let config: AppConfig = $state(DEFAULT_CONFIG);
  let activeId: string | null = $state(null);
  let bubble: string | null = $state(null);
  let bubbleTimer: ReturnType<typeof setTimeout> | undefined;
  let unlisten: (() => void) | undefined;

  const win = getCurrentWindow();
  const BAR_WIDTH = 760;

  function chunkButtons(buttons: TransformButton[], rows: number): TransformButton[][] {
    if (rows < 1 || buttons.length === 0) return [buttons];
    const perRow = Math.ceil(buttons.length / rows);
    const result: TransformButton[][] = [];
    for (let i = 0; i < buttons.length; i += perRow) {
      result.push(buttons.slice(i, i + perRow));
    }
    return result;
  }

  function showBubble(message: string) {
    bubble = message;
    clearTimeout(bubbleTimer);
    bubbleTimer = setTimeout(() => {
      bubble = null;
    }, 2200);
  }

  async function handleClick(button: TransformButton) {
    if (activeId) return; // 执行期间忽略重复点击
    activeId = button.id;
    try {
      const message = await executeButton(button.transform);
      showBubble(message);
    } catch (err) {
      showBubble(String(err));
    } finally {
      setTimeout(() => {
        activeId = null;
      }, 260);
    }
  }

  async function refresh() {
    try {
      config = await getConfig();
      resizeToFit();
      await positionToTopRight();
    } catch {
      showBubble("读取配置失败，使用默认配置");
    }
  }

  function resizeToFit() {
    const height = Math.ceil(
      12 + config.rows * config.buttonHeight + (config.rows - 1) * 4 + 2,
    );
    void win.setSize(new LogicalSize(BAR_WIDTH, height));
  }

  async function positionToTopRight() {
    try {
      const monitor = await currentMonitor();
      const size = await win.outerSize();
      if (!monitor) return;
      const x = monitor.position.x + monitor.size.width - size.width - 16;
      const y = monitor.position.y + 16;
      await win.setPosition(new PhysicalPosition(x, y));
    } catch {
      // 定位失败不影响悬浮条使用
    }
  }

  onMount(async () => {
    unlisten = await listen("config-changed", () => refresh());
    await refresh();
  });

  onDestroy(() => {
    unlisten?.();
  });
</script>

<div
  class="float-bar"
  data-tauri-drag-region
  style="--button-width: {config.buttonWidth}px; --button-height: {config.buttonHeight}px; --font-size: {config.fontSize}px;"
>
  <div class="bar-body" data-tauri-drag-region>
    <div class="button-rows" data-tauri-drag-region>
      {#each chunkButtons(config.buttons, config.rows) as row, rowIndex (rowIndex)}
        <div class="button-row" data-tauri-drag-region>
          {#each row as button (button.id)}
            <button
              type="button"
              class="transform-button"
              class:is-active={activeId === button.id}
              title={button.label}
              onclick={() => handleClick(button)}
            >
              {button.label}
            </button>
          {/each}
        </div>
      {/each}
    </div>
    <button type="button" class="settings-button" title="设置" onclick={showSettingsWindow}>
      <svg viewBox="0 0 24 24" width="18" height="18" aria-hidden="true">
        <path
          d="M19.14 12.94c.04-.3.06-.61.06-.94s-.02-.64-.07-.94l2.03-1.58a.5.5 0 0 0 .12-.63l-1.92-3.32a.5.5 0 0 0-.61-.22l-2.39.96a7.06 7.06 0 0 0-1.62-.94l-.36-2.54a.5.5 0 0 0-.5-.42h-3.84a.5.5 0 0 0-.5.42l-.36 2.54c-.59.24-1.13.56-1.62.94l-2.39-.96a.5.5 0 0 0-.61.22L2.66 8.85a.5.5 0 0 0 .12.63l2.03 1.58c-.05.3-.08.61-.08.94s.02.64.07.94l-2.03 1.58a.5.5 0 0 0-.12.63l1.92 3.32c.13.23.4.32.63.22l2.39-.96c.49.38 1.03.7 1.62.94l.36 2.54c.04.24.25.42.5.42h3.84c.25 0 .46-.18.5-.42l.36-2.54c.59-.24 1.13-.56 1.62-.94l2.39.96c.23.1.5 0 .63-.22l1.92-3.32a.5.5 0 0 0-.12-.63l-2.03-1.58zM12 15.5A3.5 3.5 0 1 1 12 8a3.5 3.5 0 0 1 0 7.5z"
          fill="currentColor"
        />
      </svg>
    </button>
  </div>

  {#if bubble}
    <div class="bubble" role="status">{bubble}</div>
  {/if}
</div>
