<script lang="ts">
  import { onMount } from "svelte";
  import { getCurrentWindow } from "@tauri-apps/api/window";
  import { getConfig, saveConfig } from "../lib/api";
  import { DEFAULT_BUTTONS, DEFAULT_CONFIG } from "../lib/defaults";
  import type { AppConfig, TransformButton } from "../lib/types";

  let config: AppConfig = $state(structuredClone(DEFAULT_CONFIG));
  let loadError: string | null = $state(null);
  let status: string | null = $state(null);
  let recording = $state(false);
  let newTransformId = $state(DEFAULT_BUTTONS[0].transform);
  let newLabel = $state("");
  let saveTimer: ReturnType<typeof setTimeout> | undefined;

  const win = getCurrentWindow();

  onMount(async () => {
    try {
      config = await getConfig();
    } catch (e) {
      loadError = String(e);
    }
  });

  function scheduleSave(next: AppConfig) {
    clearTimeout(saveTimer);
    status = "正在保存…";
    saveTimer = setTimeout(() => doSave(next), 350);
  }

  async function doSave(next: AppConfig) {
    try {
      config = await saveConfig(next);
      status = "已保存";
    } catch (e) {
      status = String(e);
    }
  }

  function clamp(value: number, min: number, max: number): number {
    return Math.min(max, Math.max(min, value));
  }

  function numericInput(
    e: Event,
    min: number,
    max: number,
    apply: (value: number) => void,
  ) {
    const value = Number((e.currentTarget as HTMLInputElement).value);
    if (Number.isNaN(value)) return;
    apply(clamp(Math.round(value), min, max));
  }

  // ---------- 快捷键 ----------
  function startRecording() {
    recording = true;
    status = "请按下新的组合键（需包含 Ctrl/Alt/Shift）…";
  }

  function onHotkeyKeydown(e: KeyboardEvent) {
    if (!recording) return;
    e.preventDefault();

    const mods: string[] = [];
    if (e.ctrlKey) mods.push("Ctrl");
    if (e.altKey) mods.push("Alt");
    if (e.shiftKey) mods.push("Shift");
    if (e.metaKey) mods.push("Cmd");

    const key = keyName(e);
    if (!key) return;
    if (mods.length === 0) {
      status = "请包含至少一个修饰键（Ctrl/Alt/Shift）";
      return;
    }

    recording = false;
    scheduleSave({ ...config, hotkey: [...mods, key].join("+") });
  }

  function keyName(e: KeyboardEvent): string | null {
    if (e.code.startsWith("Key") && e.code.length === 4) return e.code.slice(3);
    if (e.code.startsWith("Digit") && e.code.length === 6) return e.code.slice(5);
    if (/^F([1-9]|1[0-9]|2[0-4])$/.test(e.code)) return e.code;
    switch (e.code) {
      case "Space":
        return "Space";
      case "ArrowUp":
        return "Up";
      case "ArrowDown":
        return "Down";
      case "ArrowLeft":
        return "Left";
      case "ArrowRight":
        return "Right";
      case "Enter":
        return "Enter";
      case "Tab":
        return "Tab";
      case "Backspace":
        return "Backspace";
      default:
        return null;
    }
  }

  function restoreDefaultHotkey() {
    doSave({ ...config, hotkey: DEFAULT_CONFIG.hotkey });
  }

  // ---------- 按钮管理 ----------
  function groupedRows(buttons: TransformButton[], rows: number): TransformButton[][] {
    if (rows < 1 || buttons.length === 0) return [buttons];
    const perRow = Math.ceil(buttons.length / rows);
    const groups: TransformButton[][] = [];
    for (let i = 0; i < buttons.length; i += perRow) {
      groups.push(buttons.slice(i, i + perRow));
    }
    return groups;
  }

  function updateRows(value: number) {
    scheduleSave({ ...config, rows: clamp(Math.round(value), 1, 3) });
  }

  function moveButton(from: number, to: number) {
    const buttons = [...config.buttons];
    if (to < 0 || to >= buttons.length) return;
    [buttons[from], buttons[to]] = [buttons[to], buttons[from]];
    scheduleSave({ ...config, buttons });
  }

  function removeButton(index: number) {
    const buttons = config.buttons.filter((_, i) => i !== index);
    scheduleSave({ ...config, buttons });
  }

  const unusedTransforms = $derived(
    DEFAULT_BUTTONS.filter(
      (source) => !config.buttons.some((item) => item.transform === source.transform),
    ),
  );

  function addButton() {
    const source = DEFAULT_BUTTONS.find((item) => item.transform === newTransformId);
    if (!source) return;
    const button: TransformButton = {
      id: `${source.transform}-${Date.now()}`,
      label: newLabel.trim() || source.label,
      transform: source.transform,
    };
    scheduleSave({ ...config, buttons: [...config.buttons, button] });
    newLabel = "";
    const nextButtons = [...config.buttons, button];
    const nextUnused = DEFAULT_BUTTONS.find(
      (source) => !nextButtons.some((item) => item.transform === source.transform),
    );
    if (nextUnused) newTransformId = nextUnused.transform;
  }

  function restoreDefaultButtons() {
    scheduleSave({ ...config, buttons: DEFAULT_BUTTONS.map((b) => ({ ...b })) });
  }

  // ---------- 外观 / 通用 ----------
  function updateAppearance(patch: Partial<AppConfig>) {
    scheduleSave({ ...config, ...patch });
  }

  function flatIndexOf(buttons: TransformButton[], target: TransformButton): number {
    return buttons.findIndex((item) => item.id === target.id);
  }
</script>

<main class="settings-page">
  <header class="settings-header">
    <h1>StringCraft 设置</h1>
    <div class="header-actions">
      {#if status}
        <span class="save-status" class:error={status.includes("失败") || status.includes("错误")}>
          {status}
        </span>
      {/if}
      <button type="button" class="ghost-button" onclick={() => win.hide()}>关闭</button>
    </div>
  </header>

  {#if loadError}
    <p class="error-text">读取配置失败：{loadError}</p>
  {/if}

  <section class="settings-section">
    <h2>全局快捷键</h2>
    <p class="hint">点击输入框后直接按下目标组合键；必须包含至少一个修饰键。</p>
    <div class="field-row">
      <label for="hotkey">呼入/呼出悬浮条</label>
      <input
        id="hotkey"
        type="text"
        readonly
        value={config.hotkey}
        class:recording={recording}
        onclick={startRecording}
        onkeydown={onHotkeyKeydown}
      />
      <button type="button" class="ghost-button" onclick={restoreDefaultHotkey}>
        恢复默认
      </button>
    </div>
  </section>

  <section class="settings-section">
    <h2>按钮管理</h2>
    <p class="hint">按行分组展示，可增删、上下移动（支持跨行）、修改文字；每行最多 30 个。</p>

    <div class="field-row">
      <label for="rows">行数</label>
      <input
        id="rows"
        type="number"
        min="1"
        max="3"
        value={config.rows}
        onchange={(e) => numericInput(e, 1, 3, updateRows)}
      />
    </div>

    {#each groupedRows(config.buttons, config.rows) as row, rowIndex (rowIndex)}
      <div class="row-block">
        <h3>第 {rowIndex + 1} 行（{row.length} 个）</h3>
        <div class="button-list">
          {#each row as button (button.id)}
            {@const index = flatIndexOf(config.buttons, button)}
            <div class="button-list-item">
              <span class="index">{index + 1}</span>
              <input
                type="text"
                value={button.label}
                maxlength="8"
                onchange={(e) => {
                  const label = (e.currentTarget as HTMLInputElement).value.trim();
                  if (!label) return;
                  const buttons = config.buttons.map((item, i) =>
                    i === index ? { ...item, label } : item,
                  );
                  scheduleSave({ ...config, buttons });
                }}
              />
              <code>{button.transform}</code>
              <button
                type="button"
                class="icon-button"
                title="上移"
                disabled={index === 0}
                onclick={() => moveButton(index, index - 1)}
              >
                ↑
              </button>
              <button
                type="button"
                class="icon-button"
                title="下移"
                disabled={index === config.buttons.length - 1}
                onclick={() => moveButton(index, index + 1)}
              >
                ↓
              </button>
              <button
                type="button"
                class="icon-button danger"
                title="删除"
                onclick={() => removeButton(index)}
              >
                ✕
              </button>
            </div>
          {/each}
        </div>
      </div>
    {/each}

    <div class="add-row">
      <select
        bind:value={newTransformId}
        disabled={unusedTransforms.length === 0}
      >
        {#each unusedTransforms as source (source.transform)}
          <option value={source.transform}>{source.label}</option>
        {/each}
      </select>
      <input
        type="text"
        placeholder="按钮文字（默认用功能名，≤8 字）"
        maxlength="8"
        bind:value={newLabel}
        onkeydown={(e) => {
          if (e.key === "Enter") addButton();
        }}
      />
      <button
        type="button"
        class="ghost-button"
        disabled={unusedTransforms.length === 0}
        onclick={addButton}
      >
        添加按钮
      </button>
    </div>

    <button type="button" class="ghost-button" onclick={restoreDefaultButtons}>
      恢复默认按钮
    </button>
  </section>

  <section class="settings-section">
    <h2>外观</h2>
    <div class="field-grid">
      <div class="field-row">
        <label for="button-width">按钮宽度（40~200px）</label>
        <input
          id="button-width"
          type="number"
          min="40"
          max="200"
          value={config.buttonWidth}
          onchange={(e) =>
            numericInput(e, 40, 200, (v) => updateAppearance({ buttonWidth: v }))}
        />
      </div>
      <div class="field-row">
        <label for="button-height">按钮高度（28~80px）</label>
        <input
          id="button-height"
          type="number"
          min="28"
          max="80"
          value={config.buttonHeight}
          onchange={(e) =>
            numericInput(e, 28, 80, (v) => updateAppearance({ buttonHeight: v }))}
        />
      </div>
      <div class="field-row">
        <label for="font-size">按钮字号（10~24px）</label>
        <input
          id="font-size"
          type="number"
          min="10"
          max="24"
          value={config.fontSize}
          onchange={(e) =>
            numericInput(e, 10, 24, (v) => updateAppearance({ fontSize: v }))}
        />
      </div>
      <div class="field-row">
        <label for="opacity">背景不透明度</label>
        <input
          id="opacity"
          type="range"
          min="20"
          max="100"
          value={config.opacity}
          oninput={(e) =>
            numericInput(e, 20, 100, (v) => updateAppearance({ opacity: v }))}
        />
        <span class="range-value">{config.opacity}%</span>
      </div>
      <div class="field-row">
        <label for="theme">主题</label>
        <select
          id="theme"
          value={config.theme}
          onchange={(e) =>
            updateAppearance({ theme: e.currentTarget.value as AppConfig["theme"] })}
        >
          <option value="system">跟随系统</option>
          <option value="light">浅色</option>
          <option value="dark">深色</option>
        </select>
      </div>
    </div>
  </section>

  <section class="settings-section">
    <h2>通用</h2>
    <div class="toggle-row">
      <label for="autostart">开机自启</label>
      <input
        id="autostart"
        type="checkbox"
        checked={config.autoStart}
        onchange={(e) => updateAppearance({ autoStart: e.currentTarget.checked })}
      />
    </div>
    <div class="toggle-row">
      <label for="restore-clipboard">自动替换后恢复原剪贴板</label>
      <input
        id="restore-clipboard"
        type="checkbox"
        checked={config.restoreClipboard}
        onchange={(e) =>
          updateAppearance({ restoreClipboard: e.currentTarget.checked })}
      />
    </div>
    <div class="field-row">
      <label for="delay">自动替换延迟（ms，20~1000，高级）</label>
      <input
        id="delay"
        type="number"
        min="20"
        max="1000"
        value={config.replaceDelayMs}
        onchange={(e) =>
          numericInput(e, 20, 1000, (v) => updateAppearance({ replaceDelayMs: v }))}
      />
    </div>
  </section>

  <footer class="settings-footer">
    <p>StringCraft v0.1.0 · M4 设置页</p>
  </footer>
</main>

<style>
  .settings-page {
    height: 100vh;
    display: flex;
    flex-direction: column;
    background: var(--bar-bg);
    color: var(--text);
    overflow-y: auto;
    user-select: text;
  }

  .settings-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 16px 24px;
    border-bottom: 1px solid var(--bar-border);
    user-select: none;
  }

  .settings-header h1 {
    font-size: 18px;
    font-weight: 600;
  }

  .header-actions {
    display: flex;
    align-items: center;
    gap: 12px;
  }

  .save-status {
    font-size: 12px;
    color: var(--text-muted);
  }

  .save-status.error {
    color: var(--danger);
  }

  .ghost-button {
    padding: 6px 14px;
    border-radius: var(--radius);
    background: var(--button-bg);
  }

  .ghost-button:hover {
    background: var(--button-bg-hover);
  }

  .settings-section {
    padding: 20px 24px;
    border-bottom: 1px solid var(--bar-border);
  }

  .settings-section h2 {
    font-size: 14px;
    font-weight: 600;
    margin-bottom: 8px;
  }

  .hint {
    font-size: 12px;
    color: var(--text-muted);
    margin-bottom: 12px;
  }

  .error-text {
    padding: 10px 24px;
    color: var(--danger);
    font-size: 13px;
  }

  .field-row {
    display: flex;
    align-items: center;
    gap: 12px;
    margin-bottom: 10px;
  }

  .field-row label {
    flex: 1;
    font-size: 13px;
  }

  .field-row input,
  .field-row select {
    width: 220px;
    padding: 6px 10px;
    border: 1px solid var(--bar-border);
    border-radius: 6px;
    background: var(--button-bg);
    color: var(--text);
    font-size: 13px;
  }

  .field-row input[type="range"] {
    padding: 0;
  }

  .range-value {
    font-size: 12px;
    color: var(--text-muted);
    min-width: 40px;
  }

  .field-grid {
    display: grid;
    grid-template-columns: repeat(2, minmax(0, 1fr));
    gap: 0 24px;
  }

  input.recording {
    outline: 2px solid var(--accent);
  }

  .row-block {
    margin-bottom: 14px;
  }

  .row-block h3 {
    font-size: 12px;
    font-weight: 600;
    color: var(--text-muted);
    margin-bottom: 6px;
  }

  .button-list {
    display: grid;
    grid-template-columns: repeat(2, minmax(0, 1fr));
    gap: 6px;
  }

  .button-list-item {
    display: flex;
    align-items: center;
    gap: 6px;
    padding: 5px 8px;
    border: 1px solid var(--bar-border);
    border-radius: 6px;
    font-size: 13px;
  }

  .button-list-item .index {
    width: 18px;
    color: var(--text-muted);
    font-size: 12px;
    flex: none;
  }

  .button-list-item input {
    width: 90px;
    padding: 4px 6px;
    border: 1px solid var(--bar-border);
    border-radius: 4px;
    background: var(--button-bg);
    color: var(--text);
    font-size: 13px;
  }

  .button-list-item code {
    font-size: 11px;
    color: var(--text-muted);
    flex: 1;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .icon-button {
    width: 24px;
    height: 24px;
    display: grid;
    place-items: center;
    border-radius: 4px;
    color: var(--text-muted);
    font-size: 12px;
    flex: none;
  }

  .icon-button:hover {
    background: var(--button-bg-hover);
    color: var(--text);
  }

  .icon-button.danger:hover {
    background: rgba(220, 38, 38, 0.15);
    color: var(--danger);
  }

  .add-row {
    display: flex;
    gap: 8px;
    margin: 12px 0;
  }

  .add-row select,
  .add-row input {
    padding: 6px 10px;
    border: 1px solid var(--bar-border);
    border-radius: 6px;
    background: var(--button-bg);
    color: var(--text);
    font-size: 13px;
  }

  .add-row input {
    flex: 1;
  }

  .toggle-row {
    display: flex;
    align-items: center;
    gap: 12px;
    margin-bottom: 10px;
  }

  .toggle-row label {
    flex: 1;
    font-size: 13px;
  }

  .settings-footer {
    padding: 14px 24px;
    color: var(--text-muted);
    font-size: 12px;
  }
</style>
