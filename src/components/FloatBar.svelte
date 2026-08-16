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
    saveFloatBarWidth,
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
    width: number;
  } | null = $state(null);
  let buttonTooltipTimer: ReturnType<typeof setTimeout> | undefined;
  let tooltipMouse: { x: number; y: number } | null = null;
  let tooltipText = "";
  let systemDark = $state(false);
  let currentBarWidth = DEFAULT_CONFIG.toolbarWidth;
  let actionsHorizontal = $state(false);
  let maxBarWidth = 4000;
  let moveSaveTimer: ReturnType<typeof setTimeout> | undefined;
  let widthSaveTimer: ReturnType<typeof setTimeout> | undefined;
  let resizeState: { startX: number; startWidth: number } | null = null;
  let unlisten: (() => void) | undefined;
  let unlistenMove: (() => void) | undefined;

  const win = getCurrentWindow();
  const media = window.matchMedia("(prefers-color-scheme: dark)");

  const BAR_PADDING = 6;
  const BODY_GAP = 6;
  const ROW_GAP = 4;
  const BAR_BORDER = 2;
  const ACTION_GAP = 4;

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

  $effect(() => {
    document.documentElement.dataset.theme = effectiveTheme;
    document.documentElement.style.colorScheme = effectiveTheme;
  });

  function clampNumber(value: number, min: number, max: number): number {
    if (max < min) return min;
    return Math.min(max, Math.max(min, value));
  }

  function minBarWidth(): number {
    const buttonWidth = clampNumber(config.buttonWidth, 20, 200);
    const size = actionSize();
    const actionWidthHorizontal = size * 2 + ACTION_GAP;
    return Math.max(
      120,
      BAR_PADDING * 2 + actionWidthHorizontal + BODY_GAP + BAR_BORDER + buttonWidth,
    );
  }

  function buttonAreaWidth(width: number, horizontal: boolean): number {
    const size = actionSize();
    const actionWidth = horizontal ? size * 2 + ACTION_GAP : size;
    return Math.max(0, width - (BAR_PADDING * 2 + actionWidth + BODY_GAP + BAR_BORDER));
  }

  function actionSize(): number {
    return clampNumber(config.buttonHeight, 10, 80);
  }

  function rowsFor(width: number, horizontal: boolean): number {
    const buttonWidth = clampNumber(config.buttonWidth, 20, 200);
    const area = buttonAreaWidth(width, horizontal);
    const perRow =
      area >= buttonWidth
        ? Math.max(1, Math.floor((area + ROW_GAP) / (buttonWidth + ROW_GAP)))
        : 1;
    return visibleButtons.length === 0 ? 1 : Math.ceil(visibleButtons.length / perRow);
  }

  function computeLayout(width: number): {
    rows: number;
    height: number;
    actionsHorizontal: boolean;
  } {
    const buttonHeight = clampNumber(config.buttonHeight, 10, 80);
    const horizontalRows = rowsFor(width, true);
    const actionsHorizontal = horizontalRows <= 1;
    const rows = actionsHorizontal ? horizontalRows : rowsFor(width, false);
    const size = actionSize();
    const actionHeight = actionsHorizontal ? size : size * 2 + ACTION_GAP;
    const contentHeight = rows * buttonHeight + (rows - 1) * ROW_GAP;
    const height = Math.max(
      BAR_PADDING * 2 + actionHeight + BAR_BORDER,
      BAR_PADDING * 2 + contentHeight + BAR_BORDER,
    );
    return { rows, height, actionsHorizontal };
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

  function computeTooltipPlacement(text: string, mouseX: number, mouseY: number) {
    const margin = 8;
    const width = Math.min(
      260,
      Math.max(140, text.length * 14 + 24),
      Math.max(60, window.innerWidth - margin * 2),
    );
    const height = Math.min(34, Math.max(20, window.innerHeight - margin * 2));
    const x = clampNumber(
      mouseX,
      margin + width / 2,
      Math.max(margin + width / 2, window.innerWidth - width / 2 - margin),
    );

    const aboveY = mouseY - 12;
    const belowY = mouseY + 12;
    const fitsAbove = aboveY - height >= margin && aboveY <= window.innerHeight - margin;
    const fitsBelow = belowY >= margin && belowY + height <= window.innerHeight - margin;

    let y: number;
    let above: boolean;
    if (fitsAbove) {
      y = aboveY;
      above = true;
    } else if (fitsBelow) {
      y = belowY;
      above = false;
    } else {
      y = margin;
      above = false;
    }
    return { x, y, above, width };
  }

  function scheduleButtonTooltip(event: MouseEvent, button: TransformButton) {
    clearTimeout(buttonTooltipTimer);
    const text = button.description.trim() || button.name;
    if (!text) return;

    tooltipText = text;
    tooltipMouse = { x: event.clientX, y: event.clientY };
    buttonTooltipTimer = setTimeout(() => {
      if (!tooltipMouse) return;
      buttonTooltip = {
        text,
        ...computeTooltipPlacement(text, tooltipMouse.x, tooltipMouse.y),
      };
    }, BUTTON_TOOLTIP_DELAY_MS);
  }

  function updateButtonTooltipPosition(event: MouseEvent) {
    tooltipMouse = { x: event.clientX, y: event.clientY };
    if (buttonTooltip && tooltipText) {
      buttonTooltip = {
        text: tooltipText,
        ...computeTooltipPlacement(tooltipText, tooltipMouse.x, tooltipMouse.y),
      }
    }
  }

  function cancelButtonTooltip() {
    clearTimeout(buttonTooltipTimer);
    buttonTooltip = null;
    tooltipMouse = null;
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

  async function updateScreenConstraints() {
    const monitor = await activeMonitor();
    if (monitor) {
      maxBarWidth = Math.max(minBarWidth(), monitor.size.width - 32);
    }
  }

  async function applySize(width: number) {
    const clampedWidth = clampNumber(width, minBarWidth(), maxBarWidth);
    currentBarWidth = clampedWidth;
    const layout = computeLayout(clampedWidth);
    actionsHorizontal = layout.actionsHorizontal;
    await win.setSize(
      new LogicalSize(Math.round(clampedWidth), Math.round(layout.height)),
    );
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

  function onBarPointerDown(event: PointerEvent) {
    if (event.button !== 0) return;
    const target = event.target as HTMLElement | null;
    if (target?.closest("button, input, select, textarea, .resize-handle")) return;
    void win.startDragging();
  }

  function onResizePointerDown(event: PointerEvent) {
    if (event.button !== 0) return;
    event.preventDefault();
    event.stopPropagation();
    resizeState = { startX: event.screenX, startWidth: currentBarWidth };
    window.addEventListener("pointermove", onResizePointerMove);
    window.addEventListener("pointerup", onResizePointerUp, { once: true });
    window.addEventListener("pointercancel", onResizePointerUp, { once: true });
  }

  function onResizePointerMove(event: PointerEvent) {
    if (!resizeState) return;
    const delta = event.screenX - resizeState.startX;
    const nextWidth = clampNumber(
      Math.round(resizeState.startWidth + delta),
      minBarWidth(),
      maxBarWidth,
    );
    void applySize(nextWidth);
  }

  function onResizePointerUp() {
    if (!resizeState) return;
    resizeState = null;
    window.removeEventListener("pointermove", onResizePointerMove);
    window.removeEventListener("pointerup", onResizePointerUp);
    window.removeEventListener("pointercancel", onResizePointerUp);
    persistBarWidth(currentBarWidth);
  }

  function persistBarWidth(width: number) {
    clearTimeout(widthSaveTimer);
    widthSaveTimer = setTimeout(() => {
      void saveFloatBarWidth(Math.round(width));
    }, 250);
  }

  async function loadInitial() {
    try {
      config = await getConfig();
      await updateScreenConstraints();
      await applySize(config.toolbarWidth);
      await restorePosition();
      await applyNoActivate();
    } catch {
      showBubble("读取配置失败，使用默认配置");
    }
  }

  async function refreshFromConfig() {
    try {
      config = await getConfig();
      await updateScreenConstraints();
      await applySize(config.toolbarWidth);
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
    window.removeEventListener("pointermove", onResizePointerMove);
    window.removeEventListener("pointerup", onResizePointerUp);
    window.removeEventListener("pointercancel", onResizePointerUp);
    clearTimeout(buttonTooltipTimer);
    clearTimeout(bubbleTimer);
    clearTimeout(moveSaveTimer);
    clearTimeout(widthSaveTimer);
  });
</script>

<div
  class="float-bar"
  role="group"
  aria-label="StringCraft 工具条"
  onpointerdown={onBarPointerDown}
  oncontextmenu={(event) => event.preventDefault()}
  style="--button-width: {config.buttonWidth}px; --button-height: {config.buttonHeight}px; --action-size: {config.buttonHeight}px; --font-size: {config.fontSize}px; background: {barBackground};"
>
  <div class="bar-body">
    <div class="button-rows">
      {#each visibleButtons as button (button.id)}
        <button
          type="button"
          class="transform-button"
          class:is-active={activeId === button.id}
              onclick={() => handleClick(button)}
              onmouseenter={(event) => scheduleButtonTooltip(event, button)}
              onmousemove={updateButtonTooltipPosition}
              onmouseleave={cancelButtonTooltip}
        >
          {button.name}
        </button>
      {/each}
    </div>
    <div class="bar-actions" class:is-horizontal={actionsHorizontal}>
      <button type="button" class="settings-button" title="设置" onclick={showSettingsWindow}>
        <svg viewBox="0 0 24 24" width="18" height="18" aria-hidden="true">
          <path
            d="M19.14 12.94c.04-.3.06-.61.06-.94s-.02-.64-.07-.94l2.03-1.58a.5.5 0 0 0 .12-.63l-1.92-3.32a.5.5 0 0 0-.61-.22l-2.39.96a7.06 7.06 0 0 0-1.62-.94l-.36-2.54a.5.5 0 0 0-.5-.42h-3.84a.5.5 0 0 0-.5.42l-.36 2.54c-.59.24-1.13.56-1.62.94l-2.39-.96a.5.5 0 0 0-.61.22L2.66 8.85a.5.5 0 0 0 .12.63l2.03 1.58c-.05.3-.08.61-.08.94s.02.64.07.94l-2.03 1.58a.5.5 0 0 0-.12.63l1.92 3.32c.13.23.4.32.63.22l2.39-.96c.49.38 1.03.7 1.62.94l.36 2.54c.04.24.25.42.5.42h3.84c.25 0 .46-.18.5-.42l.36-2.54c.59-.24 1.13-.56 1.62-.94l2.39.96c.23.1.5 0 .63-.22l1.92-3.32a.5.5 0 0 0-.12-.63l-2.03-1.58zM12 15.5A3.5 3.5 0 1 1 12 8a3.5 3.5 0 0 1 0 7.5z"
            fill="currentColor"
          />
        </svg>
      </button>
      <button type="button" class="hide-button" title="隐藏悬浮条" onclick={() => win.hide()}>
        <svg viewBox="0 0 24 24" width="18" height="18" aria-hidden="true">
          <path d="M12 15.5 6 9.5 7.4 8.1 12 12.7 16.6 8.1 18 9.5z" fill="currentColor" />
        </svg>
      </button>
    </div>
  </div>

  <div
    class="resize-handle"
    role="presentation"
    aria-hidden="true"
    onpointerdown={onResizePointerDown}
  ></div>

  {#if bubble}
    <div class="bubble" role="status">{bubble}</div>
  {/if}

  {#if buttonTooltip}
    <div
      class="button-tooltip"
      class:above={buttonTooltip.above}
      style:left="{buttonTooltip.x}px"
      style:top="{buttonTooltip.y}px"
      style:max-width="{buttonTooltip.width}px"
    >
      {buttonTooltip.text}
    </div>
  {/if}
</div>
