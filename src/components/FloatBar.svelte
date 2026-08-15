<script lang="ts">
  import { onDestroy, onMount } from "svelte";
  import { listen } from "@tauri-apps/api/event";
  import {
    currentMonitor,
    getCurrentWindow,
    LogicalSize,
    monitorFromPoint,
    PhysicalPosition,
  } from "@tauri-apps/api/window";
  import type { Monitor } from "@tauri-apps/api/window";
  import {
    applyNoActivate,
    executeButton,
    getConfig,
    saveFloatBarPosition,
    showSettingsWindow,
  } from "../lib/api";
  import { DEFAULT_CONFIG } from "../lib/defaults";
  import { hexToRgba } from "../lib/theme";
  import type { AppConfig, TransformButton } from "../lib/types";

  let config: AppConfig = $state(DEFAULT_CONFIG);
  let activeId: string | null = $state(null);
  let bubble: string | null = $state(null);
  let bubbleTimer: ReturnType<typeof setTimeout> | undefined;
  let buttonTooltip: {
    text: string;
    x: number;
    y: number;
    above: boolean;
  } | null = $state(null);
  let buttonTooltipTimer: ReturnType<typeof setTimeout> | undefined;
  let effectiveButtonWidth = $state(72);
  let systemDark = $state(false);
  let moveSaveTimer: ReturnType<typeof setTimeout> | undefined;
  let unlisten: (() => void) | undefined;
  let unlistenMove: (() => void) | undefined;

  const win = getCurrentWindow();
  const media = window.matchMedia("(prefers-color-scheme: dark)");

  const BAR_PADDING = 6;
  const SETTINGS_WIDTH = 36;
  const BODY_GAP = 6;
  const ROW_GAP = 4;
  const BAR_BORDER = 2;
  const CHROME_WIDTH = BAR_PADDING * 2 + SETTINGS_WIDTH + BODY_GAP + BAR_BORDER;

  const effectiveTheme = $derived(
    config.theme === "system" ? (systemDark ? "dark" : "light") : config.theme,
  );
  const barBackground = $derived(
    hexToRgba(
      effectiveTheme === "dark" ? config.backgroundColorDark : config.backgroundColor,
      config.opacity / 100,
    ),
  );
  const visibleButtons = $derived(
    config.buttons.filter((button) => button.visible !== false),
  );
  const rowCount = $derived(clampNumber(config.rows, 1, 3));
  const rowGroups = $derived(chunkButtons(visibleButtons, rowCount));

  $effect(() => {
    document.documentElement.dataset.theme = effectiveTheme;
    document.documentElement.style.colorScheme = effectiveTheme;
  });

  function clampNumber(value: number, min: number, max: number): number {
    if (max < min) return min;
    return Math.min(max, Math.max(min, value));
  }

  function chunkButtons(buttons: TransformButton[], rows: number): TransformButton[][] {
    if (rows < 1 || buttons.length === 0) return [buttons];
    const perRow = Math.ceil(buttons.length / rows);
    const result: TransformButton[][] = [];
    for (let i = 0; i < buttons.length; i += perRow) {
      result.push(buttons.slice(i, i + perRow));
    }
    return result;
  }

  function computeBarWidth(buttonCount: number, buttonWidth: number): number {
    if (buttonCount <= 0) return CHROME_WIDTH;
    return CHROME_WIDTH + buttonCount * buttonWidth + (buttonCount - 1) * ROW_GAP;
  }

  function updateSystemDark() {
    systemDark = media.matches;
  }

  function showBubble(message: string) {
    bubble = message;
    clearTimeout(bubbleTimer);
    bubbleTimer = setTimeout(() => {
      bubble = null;
    }, 2200);
  }

  const BUTTON_TOOLTIP_DELAY_MS = 500;

  function scheduleButtonTooltip(event: MouseEvent, button: TransformButton) {
    clearTimeout(buttonTooltipTimer);
    const text = button.description.trim() || button.name;
    if (!text) return;

    buttonTooltipTimer = setTimeout(() => {
      const width = Math.min(260, Math.max(140, text.length * 14 + 24));
      const height = 34;
      let x = event.clientX;
      if (x - width / 2 < 8) x = width / 2 + 8;
      if (x + width / 2 > window.innerWidth - 8) x = window.innerWidth - width / 2 - 8;

      const above = event.clientY - height - 12 >= 0;
      const y = above ? event.clientY - height - 12 : event.clientY + 12;
      buttonTooltip = { text, x, y, above };
    }, BUTTON_TOOLTIP_DELAY_MS);
  }

  function cancelButtonTooltip() {
    clearTimeout(buttonTooltipTimer);
    buttonTooltip = null;
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

  async function activeMonitor(): Promise<Monitor | null> {
    try {
      const position = await win.outerPosition();
      const size = await win.outerSize();
      return await monitorFromPoint(
        position.x + size.width / 2,
        position.y + size.height / 2,
      );
    } catch {
      return await currentMonitor();
    }
  }

  async function resizeToFit() {
    const visible = visibleButtons;
    const rows = clampNumber(config.rows, 1, 3);
    const groups = chunkButtons(visible, rows);
    const maxButtons = groups.reduce((max, row) => Math.max(max, row.length), 0);

    let buttonWidth = clampNumber(config.buttonWidth, 40, 200);
    let barWidth = computeBarWidth(maxButtons, buttonWidth);

    try {
      const monitor = await activeMonitor();
      if (monitor && maxButtons > 0) {
        const usable = Math.max(CHROME_WIDTH, monitor.size.width - 32);
        const availableForButtons = Math.max(0, usable - computeBarWidth(0, buttonWidth));
        const available = Math.floor(
          (availableForButtons - (maxButtons - 1) * ROW_GAP) / maxButtons,
        );
        if (available < buttonWidth) {
          buttonWidth = clampNumber(available, 36, buttonWidth);
          barWidth = computeBarWidth(maxButtons, buttonWidth);
          if (barWidth > usable) barWidth = usable;
        }
      }
    } catch {
      // 无法获取显示器信息时按配置宽度展示
    }

    barWidth = Math.max(120, barWidth);
    effectiveButtonWidth = buttonWidth;
    const renderedRowCount = Math.max(1, groups.length);
    const height = Math.ceil(
      BAR_PADDING * 2 +
        renderedRowCount * config.buttonHeight +
        (renderedRowCount - 1) * ROW_GAP +
        BAR_BORDER,
    );
    void win.setSize(new LogicalSize(Math.round(barWidth), height));
  }

  async function positionToTopRight() {
    try {
      const monitor = await currentMonitor();
      const size = await win.outerSize();
      if (!monitor) return;
      const x = monitor.position.x + monitor.size.width - size.width - 16;
      const y = monitor.position.y + 16;
      await win.setPosition(new PhysicalPosition(Math.round(x), Math.round(y)));
    } catch {
      // 定位失败不影响悬浮条使用
    }
  }

  async function restorePosition() {
    const saved = config.position;
    if (!saved) {
      await positionToTopRight();
      return;
    }

    try {
      const monitor =
        (await monitorFromPoint(saved.x, saved.y)) ?? (await currentMonitor());
      const size = await win.outerSize();
      if (!monitor) {
        await win.setPosition(new PhysicalPosition(Math.round(saved.x), Math.round(saved.y)));
        return;
      }
      const maxX = monitor.position.x + monitor.size.width - size.width;
      const maxY = monitor.position.y + monitor.size.height - size.height;
      const x = clampNumber(saved.x, monitor.position.x, Math.max(monitor.position.x, maxX));
      const y = clampNumber(saved.y, monitor.position.y, Math.max(monitor.position.y, maxY));
      await win.setPosition(new PhysicalPosition(Math.round(x), Math.round(y)));
    } catch {
      // 恢复失败时保持系统默认位置
    }
  }

  async function ensurePositionInScreen() {
    try {
      const position = await win.outerPosition();
      const monitor =
        (await monitorFromPoint(position.x, position.y)) ?? (await currentMonitor());
      const size = await win.outerSize();
      if (!monitor) return;
      const maxX = monitor.position.x + monitor.size.width - size.width;
      const maxY = monitor.position.y + monitor.size.height - size.height;
      const x = clampNumber(
        position.x,
        monitor.position.x,
        Math.max(monitor.position.x, maxX),
      );
      const y = clampNumber(
        position.y,
        monitor.position.y,
        Math.max(monitor.position.y, maxY),
      );
      if (x !== position.x || y !== position.y) {
        await win.setPosition(new PhysicalPosition(Math.round(x), Math.round(y)));
      }
    } catch {
      // 越界归位失败不影响使用
    }
  }

  function onWindowMoved(event: { payload: { x: number; y: number } }) {
    clearTimeout(moveSaveTimer);
    moveSaveTimer = setTimeout(() => {
      void saveFloatBarPosition(
        Math.round(event.payload.x),
        Math.round(event.payload.y),
      );
    }, 300);
  }

  async function loadInitial() {
    try {
      config = await getConfig();
      await resizeToFit();
      await restorePosition();
      await applyNoActivate();
    } catch {
      showBubble("读取配置失败，使用默认配置");
    }
  }

  async function refreshFromConfig() {
    try {
      config = await getConfig();
      await resizeToFit();
      await ensurePositionInScreen();
      await applyNoActivate();
    } catch {
      showBubble("读取配置失败，使用默认配置");
    }
  }

  onMount(async () => {
    updateSystemDark();
    media.addEventListener("change", updateSystemDark);
    unlisten = await listen("config-changed", () => refreshFromConfig());
    try {
      unlistenMove = await win.onMoved(onWindowMoved);
    } catch {
      // 无法监听移动事件时仍可手动拖动，只是不会记忆位置
    }
    await loadInitial();
  });

  onDestroy(() => {
    unlisten?.();
    unlistenMove?.();
    media.removeEventListener("change", updateSystemDark);
    clearTimeout(buttonTooltipTimer);
    clearTimeout(bubbleTimer);
    clearTimeout(moveSaveTimer);
  });
</script>

<div
  class="float-bar"
  data-tauri-drag-region
  style="--button-width: {effectiveButtonWidth}px; --button-height: {config.buttonHeight}px; --font-size: {config.fontSize}px; background: {barBackground};"
>
  <div class="bar-body" data-tauri-drag-region>
    <div class="button-rows" data-tauri-drag-region>
      {#each rowGroups as row, rowIndex (rowIndex)}
        <div class="button-row" data-tauri-drag-region>
          {#each row as button (button.id)}
            <button
              type="button"
              class="transform-button"
              class:is-active={activeId === button.id}
              onclick={() => handleClick(button)}
              onmouseenter={(event) => scheduleButtonTooltip(event, button)}
              onmouseleave={cancelButtonTooltip}
            >
              {button.name}
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

  {#if buttonTooltip}
    <div
      class="button-tooltip"
      class:above={buttonTooltip.above}
      style:left="{buttonTooltip.x}px"
      style:top="{buttonTooltip.y}px"
    >
      {buttonTooltip.text}
    </div>
  {/if}
</div>
