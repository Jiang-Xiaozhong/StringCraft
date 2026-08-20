<script lang="ts">
  import { onDestroy, onMount } from "svelte";
  import { flip } from "svelte/animate";
  import { open, save } from "@tauri-apps/plugin-dialog";
  import { listen } from "@tauri-apps/api/event";
  import { getCurrentWindow } from "@tauri-apps/api/window";
  import {
    checkForUpdate,
    exportConfigTo,
    getConfig,
    importConfigFrom,
    installUpdate,
    isMacOS,
    macOSAccessibilityTrusted,
    openInBrowser,
    openMacOSAccessibilitySettings,
    saveConfig,
    type UpdateInfo,
  } from "../lib/api";
  import alipayImg from "../assets/alipay.jpg";
  import wechatImg from "../assets/wechat.jpg";
  import { DEFAULT_CONFIG, getDefaultButtons } from "../lib/defaults";
  import { t, translateRustMessage } from "../lib/i18n";
  import { darkenHex, MACARON_COLORS, MORANDI_COLORS } from "../lib/theme";
  import type { AppConfig, TransformButton } from "../lib/types";

  let config: AppConfig = $state(structuredClone(DEFAULT_CONFIG));
  let loadError: string | null = $state(null);
  let status: string | null = $state(null);
  let recording = $state(false);
  let newTransformId = $state("");
  let newName = $state("");
  let newDescription = $state("");
  let newCustomType = $state("");
  let newCustomParam1 = $state("");
  let newCustomParam2 = $state("");
  let newCustomName = $state("");
  let newCustomDescription = $state("");
  let checkingUpdate = $state(false);
  let updateInfo: UpdateInfo | null = $state(null);
  let updateReadyPath: string | null = $state(null);
  let latestHint: string | null = $state(null);
  let latestHintTimer: ReturnType<typeof setTimeout> | undefined;
  let platform = $state("windows");
  let macPermission = $state(false);
  let systemDark = $state(false);
  let draggingIndex: number | null = $state(null);
  let dragOverIndex: number | null = $state(null);
  let showRestoreConfirm = $state(false);
  let dragOffsetY = $state(0);
  let dragPointerStartY = 0;
  let dragTargets: number[] = [];
  let saveTimer: ReturnType<typeof setTimeout> | undefined;
  let unlistenUpdateFound: (() => void) | undefined;
  let unlistenUpdateReady: (() => void) | undefined;

  const CUSTOM_TYPES = [
    { id: "append-suffix", nameKey: "custom.appendSuffix", paramKey: "custom.suffixPlaceholder" },
    { id: "prepend-prefix", nameKey: "custom.prependPrefix", paramKey: "custom.prefixPlaceholder" },
    { id: "prepend-append", nameKey: "custom.prependAppend", descKey: "custom.prependAppendDesc", paramKey: "custom.prefixPlaceholder", param2Key: "custom.suffixPlaceholder" },
    { id: "replace-text", nameKey: "custom.replaceText", paramKey: "custom.replaceFromPlaceholder", param2Key: "custom.replaceToPlaceholder" },
    { id: "remove-duplicate-lines", nameKey: "custom.removeDuplicateLines", paramKey: "" },
  ];

  function tt(key: string, vars?: Record<string, string>): string {
    return t(config.language, key, vars);
  }

  const win = getCurrentWindow();
  const media = window.matchMedia("(prefers-color-scheme: dark)");

  const effectiveTheme = $derived(
    config.theme === "system" ? (systemDark ? "dark" : "light") : config.theme,
  );

  $effect(() => {
    document.documentElement.dataset.theme = effectiveTheme;
    document.documentElement.style.colorScheme = effectiveTheme;
  });

  function updateSystemDark() {
    systemDark = media.matches;
  }

  onMount(async () => {
    updateSystemDark();
    media.addEventListener("change", updateSystemDark);
    try {
      config = await getConfig();
    } catch (e) {
      loadError = String(e);
    }
    unlistenUpdateFound = await listen("update-found", (event) => {
      updateInfo = event.payload as UpdateInfo;
    });
    unlistenUpdateReady = await listen("update-ready", (event) => {
      updateReadyPath = event.payload as string;
      status = tt("settings.update.ready");
    });
    if (await isMacOS()) {
      platform = "macos";
      macPermission = await macOSAccessibilityTrusted();
    }
  });

  onDestroy(() => {
    media.removeEventListener("change", updateSystemDark);
    window.removeEventListener("keydown", onHotkeyKeydown);
    unlistenUpdateFound?.();
    unlistenUpdateReady?.();
    clearTimeout(saveTimer);
    clearTimeout(latestHintTimer);
  });

  function scheduleSave(next: AppConfig) {
    clearTimeout(saveTimer);
    status = tt("settings.save.saving");
    saveTimer = setTimeout(() => doSave(next), 350);
  }

  function scheduleQuickSave(next: AppConfig) {
    clearTimeout(saveTimer);
    status = tt("settings.save.saving");
    saveTimer = setTimeout(() => doSave(next), 80);
  }

  async function doSave(next: AppConfig) {
    try {
      const latest = await getConfig();
      const patch = diffConfig(config, next);
      config = await saveConfig({ ...latest, ...patch });
      status = tt("settings.save.saved");
    } catch (e) {
      status = translateRustMessage(config.language, String(e));
    }
  }

  function diffConfig(before: AppConfig, after: AppConfig): Partial<AppConfig> {
    const patch: Partial<AppConfig> = {};
    (Object.keys(after) as (keyof AppConfig)[]).forEach((key) => {
      if (JSON.stringify(before[key]) !== JSON.stringify(after[key])) {
        (patch as Record<string, unknown>)[key] = after[key];
      }
    });
    return patch;
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
    if (recording) return;
    recording = true;
    status = tt("settings.hotkey.recording");
    window.addEventListener("keydown", onHotkeyKeydown);
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
    window.removeEventListener("keydown", onHotkeyKeydown);
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
  function removeButton(index: number) {
    const buttons = config.buttons.filter((_, i) => i !== index);
    scheduleSave({ ...config, buttons });
  }

  function updateButtonName(index: number, name: string) {
    if (!name.trim()) return;
    const buttons = config.buttons.map((item, i) =>
      i === index ? { ...item, name: name.trim() } : item,
    );
    scheduleSave({ ...config, buttons });
  }

  function updateButtonDescription(index: number, description: string) {
    const buttons = config.buttons.map((item, i) =>
      i === index ? { ...item, description: description.trim() } : item,
    );
    scheduleSave({ ...config, buttons });
  }

  function updateButtonVisible(index: number, visible: boolean) {
    const buttons = config.buttons.map((item, i) =>
      i === index ? { ...item, visible } : item,
    );
    scheduleSave({ ...config, buttons });
  }

  function onDragPointerDown(event: PointerEvent, index: number) {
    if (event.button !== 0) return;
    event.preventDefault();
    event.stopPropagation();
    draggingIndex = index;
    dragOverIndex = index;
    dragPointerStartY = event.clientY;
    dragOffsetY = 0;
    dragTargets = Array.from(
      document.querySelectorAll<HTMLElement>(".button-list-item"),
    ).map((item) => {
      const rect = item.getBoundingClientRect();
      return rect.top + rect.height / 2;
    });
    window.addEventListener("pointermove", onDragPointerMove);
    window.addEventListener("pointerup", onDragPointerUp, { once: true });
    window.addEventListener("pointercancel", onDragPointerUp, { once: true });
  }

  function onDragPointerMove(event: PointerEvent) {
    if (draggingIndex === null) return;
    dragOffsetY = event.clientY - dragPointerStartY;

    let targetIndex = 0;
    for (let i = 0; i < dragTargets.length; i++) {
      if (event.clientY < dragTargets[i]) break;
      targetIndex = i + 1;
    }
    dragOverIndex = targetIndex;
  }

  function onDragPointerUp() {
    if (draggingIndex === null) {
      dragOverIndex = null;
      return;
    }
    const from = draggingIndex;
    const to = dragOverIndex ?? from;
    window.removeEventListener("pointermove", onDragPointerMove);
    window.removeEventListener("pointerup", onDragPointerUp);
    window.removeEventListener("pointercancel", onDragPointerUp);
    draggingIndex = null;
    dragOverIndex = null;
    dragOffsetY = 0;
    dragTargets = [];

    if (from !== to && to !== from + 1) {
      const buttons = [...config.buttons];
      const [moved] = buttons.splice(from, 1);
      const insertIndex = to > from ? to - 1 : to;
      buttons.splice(insertIndex, 0, moved);
      scheduleSave({ ...config, buttons });
    }
  }

  const unusedTransforms = $derived(
    getDefaultButtons(config.language).filter(
      (source) => !config.buttons.some((item) => item.transform === source.transform),
    ),
  );

  function applyTransformDefaults(id: string) {
    newTransformId = id;
    const source = getDefaultButtons(config.language).find((item) => item.transform === id);
    if (source) {
      newName = source.name;
      newDescription = source.description;
    }
  }

  function addButton() {
    const source = getDefaultButtons(config.language).find(
      (item) => item.transform === newTransformId,
    );
    if (!source) return;
    const button: TransformButton = {
      id: `${source.transform}-${Date.now()}`,
      name: newName.trim() || source.name,
      transform: source.transform,
      description: newDescription.trim(),
      visible: true,
    };
    scheduleSave({ ...config, buttons: [...config.buttons, button] });
    newTransformId = "";
    newName = "";
    newDescription = "";
  }

  function onAddTransformChange(event: Event) {
    applyTransformDefaults((event.currentTarget as HTMLSelectElement).value);
  }

  function customParamLabel(customType: string | null | undefined, first: boolean): string {
    const type = CUSTOM_TYPES.find((item) => item.id === customType);
    if (!type) return "";
    return first ? tt(type.paramKey) : type.param2Key ? tt(type.param2Key) : "";
  }

  function onCustomTypeChange(event: Event) {
    const id = (event.currentTarget as HTMLSelectElement).value;
    newCustomType = id;
    newCustomParam1 = "";
    newCustomParam2 = "";
    const type = CUSTOM_TYPES.find((item) => item.id === id);
    if (type) {
      newCustomName = tt(type.nameKey);
      newCustomDescription = tt(type.descKey ?? type.nameKey);
    } else {
      newCustomName = "";
      newCustomDescription = "";
    }
  }

  function addCustomButton() {
    const type = CUSTOM_TYPES.find((item) => item.id === newCustomType);
    if (!type) return;
    if (
      (type.id === "append-suffix" || type.id === "prepend-prefix") &&
      !newCustomParam1.trim()
    ) {
      status = "请填写目标文本";
      return;
    }
    if (
      type.id === "replace-text" &&
      (!newCustomParam1.trim() || !newCustomParam2.trim())
    ) {
      status = "请填写被替换文本和替换为文本";
      return;
    }
    if (
      type.id === "prepend-append" &&
      (!newCustomParam1.trim() || !newCustomParam2.trim())
    ) {
      status = "请填写前缀文本和后缀文本";
      return;
    }
    const button: TransformButton = {
      id: `custom-${Date.now()}`,
      name: newCustomName.trim() || tt(type.nameKey),
      transform: "custom",
      description: newCustomDescription.trim(),
      visible: true,
      customType: newCustomType,
      param1: type.id === "remove-duplicate-lines" ? null : newCustomParam1.trim(),
      param2:
        type.id === "replace-text" || type.id === "prepend-append"
          ? newCustomParam2.trim()
          : null,
    };
    scheduleSave({ ...config, buttons: [...config.buttons, button] });
    newCustomType = "";
    newCustomParam1 = "";
    newCustomParam2 = "";
    newCustomName = "";
    newCustomDescription = "";
  }

  function updateButtonParam1(index: number, value: string) {
    const buttons = config.buttons.map((item, i) =>
      i === index ? { ...item, param1: value.trim() } : item,
    );
    scheduleSave({ ...config, buttons });
  }

  function updateButtonParam2(index: number, value: string) {
    const buttons = config.buttons.map((item, i) =>
      i === index ? { ...item, param2: value.trim() } : item,
    );
    scheduleSave({ ...config, buttons });
  }

  function restoreDefaultButtons() {
    scheduleSave({
      ...config,
      buttons: getDefaultButtons(config.language).map((b) => ({ ...b })),
    });
  }

  function requestRestoreDefaultButtons() {
    showRestoreConfirm = true;
  }

  function confirmRestoreDefaultButtons() {
    showRestoreConfirm = false;
    restoreDefaultButtons();
  }

  function cancelRestoreDefaultButtons() {
    showRestoreConfirm = false;
  }

  async function exportConfig() {
    try {
      const path = await save({
        defaultPath: "stringcraft-config.json",
        filters: [{ name: "JSON", extensions: ["json"] }],
      });
      if (!path) return;
      status = await exportConfigTo(path);
    } catch (e) {
      status = translateRustMessage(config.language, String(e));
    }
  }

  async function importConfig() {
    try {
      const path = await open({
        multiple: false,
        directory: false,
        filters: [{ name: "JSON", extensions: ["json"] }],
      });
      if (!path) return;
      status = await importConfigFrom(path as string);
      config = await getConfig();
    } catch (e) {
      status = translateRustMessage(config.language, String(e));
    }
  }

  async function manualCheckUpdate() {
    checkingUpdate = true;
    try {
      const info = await checkForUpdate();
      updateInfo = info;
      if (!info.latest) {
        latestHint = tt("settings.update.latestHint");
        clearTimeout(latestHintTimer);
        latestHintTimer = setTimeout(() => {
          latestHint = null;
        }, 4000);
        status = tt("settings.update.latest", { version: info.version ?? "0.1.0" });
      }
    } catch (e) {
      status = translateRustMessage(config.language, String(e));
    } finally {
      checkingUpdate = false;
    }
  }

  async function openDownloadPage() {
    const url = updateInfo?.url ?? "https://github.com/Jiang-Xiaozhong/StringCraft/releases";
    try {
      await openInBrowser(url);
    } catch (e) {
      status = translateRustMessage(config.language, String(e));
    }
  }

  async function confirmInstallUpdate() {
    if (!updateReadyPath) return;
    try {
      status = await installUpdate(updateReadyPath);
    } catch (e) {
      status = translateRustMessage(config.language, String(e));
    }
    updateReadyPath = null;
  }

  async function refreshMacPermission() {
    macPermission = await macOSAccessibilityTrusted();
  }

  async function openMacSettings() {
    try {
      await openMacOSAccessibilitySettings();
    } catch (e) {
      status = translateRustMessage(config.language, String(e));
    }
  }

  // ---------- 外观 / 通用 ----------
  function updateAppearance(patch: Partial<AppConfig>) {
    scheduleQuickSave({ ...config, ...patch });
  }

  function selectPreset(color: { name: string; light: string; dark: string }) {
    scheduleSave({
      ...config,
      backgroundColor: color.light,
      backgroundColorDark: color.dark,
    });
  }

  function onCustomColorChange(event: Event) {
    const value = (event.currentTarget as HTMLInputElement).value;
    scheduleSave({
      ...config,
      backgroundColor: value,
      backgroundColorDark: darkenHex(value),
    });
  }

  function flatIndexOf(buttons: TransformButton[], target: TransformButton): number {
    return buttons.findIndex((item) => item.id === target.id);
  }
</script>

<main class="settings-page" oncontextmenu={(event) => event.preventDefault()}>
  <header class="settings-header">
    <h1>{tt("settings.title")}</h1>
    <div class="header-actions">
      {#if status}
        <span class="save-status" class:error={status.includes("失败") || status.includes("错误")}>
          {status}
        </span>
      {/if}
      <button type="button" class="ghost-button" onclick={() => win.hide()}>{tt("settings.close")}</button>
    </div>
  </header>

  {#if loadError}
    <p class="error-text">读取配置失败：{loadError}</p>
  {/if}

  <section class="settings-section">
    <h2>{tt("settings.section.hotkey")}</h2>
    <p class="hint">{tt("settings.hotkey.hint")}</p>
    <div class="field-row">
      <label for="hotkey">{tt("settings.hotkey.label")}</label>
      <input
        id="hotkey"
        type="text"
        readonly
        value={config.hotkey}
        class:recording={recording}
        onclick={startRecording}
      />
      <button type="button" class="ghost-button" onclick={restoreDefaultHotkey}>
        {tt("settings.hotkey.restore")}
      </button>
    </div>
  </section>

  <section class="settings-section">
    <h2>{tt("settings.section.buttons")}</h2>
    <p class="hint">{tt("settings.buttons.hint")}</p>

    <div class="button-list" role="list">
          {#each config.buttons as button (button.id)}
            {@const index = flatIndexOf(config.buttons, button)}
            <div
              class="button-list-item"
              role="listitem"
              animate:flip={{ duration: 160 }}
              class:is-hidden={!button.visible}
              class:is-dragging={draggingIndex === index}
              class:drag-over={dragOverIndex === index && dragOverIndex !== draggingIndex}
              style:transform={draggingIndex === index
                ? `translateY(${dragOffsetY}px)`
                : undefined}
            >
              <span
                class="drag-handle"
                role="button"
                tabindex="0"
                aria-label="拖拽排序"
                title="拖拽排序"
                onpointerdown={(event) => onDragPointerDown(event, index)}
              >
                ⠿
              </span>
              <span class="index">{index + 1}</span>
              <div class="item-main">
                <div class="item-fields">
                  <input
                    type="text"
                    value={button.name}
                    maxlength="8"
                    placeholder={tt("settings.buttons.name")}
                    onchange={(e) =>
                      updateButtonName(index, (e.currentTarget as HTMLInputElement).value)}
                  />
                  <input
                    type="text"
                    value={button.description}
                    maxlength="60"
                    placeholder={tt("settings.buttons.desc")}
                    onchange={(e) =>
                      updateButtonDescription(
                        index,
                        (e.currentTarget as HTMLInputElement).value,
                      )}
                  />
                  {#if button.transform === "custom"}
                    {#if button.customType !== "remove-duplicate-lines"}
                      <input
                        class="param-input"
                        type="text"
                        value={button.param1 ?? ""}
                        maxlength="60"
                        placeholder={customParamLabel(button.customType, true)}
                        onchange={(e) =>
                          updateButtonParam1(
                            index,
                            (e.currentTarget as HTMLInputElement).value,
                          )}
                      />
                    {/if}
                    {#if button.customType === "replace-text" || button.customType === "prepend-append"}
                      <input
                        class="param-input"
                        type="text"
                        value={button.param2 ?? ""}
                        maxlength="60"
                        placeholder={customParamLabel(button.customType, false)}
                        onchange={(e) =>
                          updateButtonParam2(
                            index,
                            (e.currentTarget as HTMLInputElement).value,
                          )}
                      />
                    {/if}
                  {/if}
                </div>
              </div>
              <label class="visibility-toggle" title={tt("settings.buttons.showHide")}>
                <input
                  type="checkbox"
                  checked={button.visible}
                  onchange={(e) =>
                    updateButtonVisible(index, (e.currentTarget as HTMLInputElement).checked)}
                />
                <span>{tt("settings.buttons.show")}</span>
              </label>
              <button
                type="button"
                class="icon-button danger"
                title={tt("settings.buttons.delete")}
                onclick={() => removeButton(index)}
              >
                ✕
              </button>
            </div>
          {/each}
    </div>

    <div class="add-row">
      <select
        value={newTransformId}
        onchange={onAddTransformChange}
        disabled={unusedTransforms.length === 0}
      >
        <option value="" disabled>{tt("settings.buttons.defaultButton")}</option>
        {#each unusedTransforms as source (source.transform)}
          <option value={source.transform}>{source.name}</option>
        {/each}
      </select>
      <input
        type="text"
        placeholder={tt("settings.buttons.addName")}
        maxlength="8"
        bind:value={newName}
        onkeydown={(e) => {
          if (e.key === "Enter") addButton();
        }}
      />
      <input
        type="text"
        placeholder={tt("settings.buttons.desc")}
        maxlength="60"
        bind:value={newDescription}
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
        {tt("settings.buttons.add")}
      </button>
    </div>

    <div class="add-row custom-add-row">
      <select value={newCustomType} onchange={onCustomTypeChange}>
        <option value="" disabled>{tt("settings.buttons.customButton")}</option>
        {#each CUSTOM_TYPES as type (type.id)}
          <option value={type.id}>{tt(type.nameKey)}</option>
        {/each}
      </select>
      <input
        type="text"
        placeholder={tt("settings.buttons.customName")}
        maxlength="8"
        bind:value={newCustomName}
      />
      <input
        type="text"
        placeholder={tt("settings.buttons.desc")}
        maxlength="60"
        bind:value={newCustomDescription}
      />
      {#if newCustomType && newCustomType !== "remove-duplicate-lines"}
        <input
          type="text"
          placeholder={customParamLabel(newCustomType, true)}
          maxlength="60"
          bind:value={newCustomParam1}
        />
      {/if}
      {#if newCustomType === "replace-text" || newCustomType === "prepend-append"}
        <input
          type="text"
          placeholder={customParamLabel(newCustomType, false)}
          maxlength="60"
          bind:value={newCustomParam2}
        />
      {/if}
      <button type="button" class="ghost-button" onclick={addCustomButton}>
        {tt("settings.buttons.addCustom")}
      </button>
    </div>

    <button type="button" class="ghost-button" onclick={requestRestoreDefaultButtons}>
      {tt("settings.buttons.restore")}
    </button>
  </section>

  <section class="settings-section">
    <h2>{tt("settings.section.appearance")}</h2>
    <div class="field-grid">
      <div class="field-row">
        <label for="button-width">{tt("settings.appearance.width")}</label>
        <input
          id="button-width"
          type="range"
          min="20"
          max="200"
          value={config.buttonWidth}
          oninput={(e) =>
            numericInput(e, 20, 200, (v) => updateAppearance({ buttonWidth: v }))}
        />
        <span class="range-value">{config.buttonWidth}px</span>
      </div>
      <div class="field-row">
        <label for="button-height">{tt("settings.appearance.height")}</label>
        <input
          id="button-height"
          type="range"
          min="10"
          max="80"
          value={config.buttonHeight}
          oninput={(e) =>
            numericInput(e, 10, 80, (v) => updateAppearance({ buttonHeight: v }))}
        />
        <span class="range-value">{config.buttonHeight}px</span>
      </div>
      <div class="field-row">
        <label for="font-size">{tt("settings.appearance.font")}</label>
        <input
          id="font-size"
          type="range"
          min="10"
          max="24"
          value={config.fontSize}
          oninput={(e) =>
            numericInput(e, 10, 24, (v) => updateAppearance({ fontSize: v }))}
        />
        <span class="range-value">{config.fontSize}px</span>
      </div>
      <div class="field-row">
        <label for="opacity">{tt("settings.appearance.opacity")}</label>
        <input
          id="opacity"
          type="range"
          min="0"
          max="100"
          value={config.opacity}
          oninput={(e) =>
            numericInput(e, 0, 100, (v) => updateAppearance({ opacity: v }))}
        />
        <span class="range-value">{config.opacity}%</span>
      </div>
      <div class="field-row">
        <label for="theme">{tt("settings.appearance.theme")}</label>
        <select
          id="theme"
          value={config.theme}
          onchange={(e) =>
            updateAppearance({ theme: e.currentTarget.value as AppConfig["theme"] })}
        >
          <option value="system">{tt("settings.appearance.themeSystem")}</option>
          <option value="light">{tt("settings.appearance.themeLight")}</option>
          <option value="dark">{tt("settings.appearance.themeDark")}</option>
        </select>
      </div>
    </div>

    <div class="color-section">
      <div class="color-section-title">{tt("settings.appearance.colorMacaron")}</div>
      <div class="color-presets">
        {#each MACARON_COLORS as color (color.name)}
          <button
            type="button"
            class="color-swatch"
            class:selected={config.backgroundColor.toUpperCase() === color.light}
            title={tt("color." + color.name)}
            style:background={color.light}
            onclick={() => selectPreset(color)}
          >
            {tt("color." + color.name)}
          </button>
        {/each}
      </div>
      <div class="color-section-title">{tt("settings.appearance.colorMorandi")}</div>
      <div class="color-presets">
        {#each MORANDI_COLORS as color (color.name)}
          <button
            type="button"
            class="color-swatch"
            class:selected={config.backgroundColor.toUpperCase() === color.light}
            title={tt("color." + color.name)}
            style:background={color.light}
            onclick={() => selectPreset(color)}
          >
            {tt("color." + color.name)}
          </button>
        {/each}
      </div>
      <div class="custom-color-row">
        <label for="custom-color">{tt("settings.appearance.customColor")}</label>
        <input
          id="custom-color"
          type="color"
          value={config.backgroundColor}
          oninput={onCustomColorChange}
        />
        <span class="custom-preview">
          <span class="swatch-mini" style:background={config.backgroundColor}></span>
          {tt("settings.appearance.light")}
          <span class="swatch-mini" style:background={config.backgroundColorDark}></span>
          {tt("settings.appearance.dark")}
        </span>
      </div>
    </div>
  </section>

  {#if platform === "macos"}
    <section class="settings-section">
      <h2>{tt("settings.macos.title")}</h2>
      <p class="hint">
        {macPermission
          ? tt("settings.macos.statusGranted")
          : tt("settings.macos.statusDenied")}
      </p>
      <div class="field-row">
        <button type="button" class="ghost-button" onclick={refreshMacPermission}>
          {tt("settings.macos.check")}
        </button>
        <button type="button" class="ghost-button" onclick={openMacSettings}>
          {tt("settings.macos.open")}
        </button>
      </div>
    </section>
  {/if}

  <section class="settings-section">
    <h2>{tt("settings.section.general")}</h2>
    <div class="toggle-row">
      <label for="autostart">{tt("settings.general.autostart")}</label>
      <input
        id="autostart"
        type="checkbox"
        checked={config.autoStart}
        onchange={(e) => updateAppearance({ autoStart: e.currentTarget.checked })}
      />
    </div>
    <div class="toggle-row">
      <label for="restore-clipboard">{tt("settings.general.restoreClipboard")}</label>
      <input
        id="restore-clipboard"
        type="checkbox"
        checked={config.restoreClipboard}
        onchange={(e) =>
          updateAppearance({ restoreClipboard: e.currentTarget.checked })}
      />
    </div>
    <div class="toggle-row">
      <label for="debug-log">
        {tt("settings.general.debugLog")}
        <span class="toggle-hint">{tt("settings.general.debugHint")}</span>
      </label>
      <input
        id="debug-log"
        type="checkbox"
        checked={config.debugLog}
        onchange={(e) => updateAppearance({ debugLog: e.currentTarget.checked })}
      />
    </div>
    <div class="field-row">
      <label for="config-import-export">{tt("settings.general.importExport")}</label>
      <button type="button" class="ghost-button" onclick={exportConfig}>
        {tt("settings.general.export")}
      </button>
      <button type="button" class="ghost-button" onclick={importConfig}>
        {tt("settings.general.import")}
      </button>
    </div>
    <div class="field-row">
      <label for="language">{tt("settings.general.language")}</label>
      <select
        id="language"
        value={config.language}
        onchange={(e) =>
          updateAppearance({ language: e.currentTarget.value as AppConfig["language"] })}
      >
        <option value="zh-CN">中文</option>
        <option value="en-US">English</option>
      </select>
    </div>
    <div class="field-row">
      <label for="delay">{tt("settings.general.delay")}</label>
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

  <section class="settings-section">
    <h2>{tt("settings.section.update")}</h2>
    <div class="field-row">
      <label for="check-update">{tt("settings.update.check")}</label>
      {#if latestHint}
        <span class="update-latest-hint">{latestHint}</span>
      {/if}
      <button
        id="check-update"
        type="button"
        class="ghost-button"
        disabled={checkingUpdate}
        onclick={manualCheckUpdate}
      >
        {checkingUpdate ? tt("settings.update.checking") : tt("settings.update.check")}
      </button>
    </div>
    <div class="toggle-row">
      <label for="auto-check-update">{tt("settings.update.autoCheck")}</label>
      <input
        id="auto-check-update"
        type="checkbox"
        checked={config.autoCheckUpdate}
        onchange={(e) =>
          updateAppearance({ autoCheckUpdate: e.currentTarget.checked })}
      />
    </div>
    <div class="toggle-row">
      <label for="auto-update">{tt("settings.update.autoUpdate")}</label>
      <input
        id="auto-update"
        type="checkbox"
        checked={config.autoUpdate}
        onchange={(e) => updateAppearance({ autoUpdate: e.currentTarget.checked })}
      />
    </div>
    {#if updateInfo?.latest}
      <div class="update-panel">
        <p>{tt("settings.update.found", { version: updateInfo.version ?? "" })}</p>
        {#if updateInfo.notes}
          <pre>{updateInfo.notes}</pre>
        {/if}
        <button type="button" class="ghost-button" onclick={openDownloadPage}>
          {tt("settings.update.download")}
        </button>
      </div>
    {/if}
  </section>

  <section class="settings-section">
    <h2>{tt("settings.section.donation")}</h2>
    {#if config.showDonation}
      <p class="hint">{tt("settings.donation.copy")}</p>
      <div class="donation-images">
        <figure>
          <img src={alipayImg} alt={tt("settings.donation.alipay")} />
          <figcaption>{tt("settings.donation.alipay")}</figcaption>
        </figure>
        <figure>
          <img src={wechatImg} alt={tt("settings.donation.wechat")} />
          <figcaption>{tt("settings.donation.wechat")}</figcaption>
        </figure>
      </div>
      <button
        type="button"
        class="ghost-button"
        onclick={() => updateAppearance({ showDonation: false })}
      >
        {tt("settings.donation.hide")}
      </button>
    {:else}
      <button
        type="button"
        class="ghost-button"
        onclick={() => updateAppearance({ showDonation: true })}
      >
        {tt("settings.donation.show")}
      </button>
    {/if}
  </section>

  <footer class="settings-footer">
    <p>{tt("settings.footer.version")}</p>
    <p>{tt("settings.footer.feedback")}<a href="mailto:jxzlh1208@163.com">jxzlh1208@163.com</a></p>
  </footer>

  {#if showRestoreConfirm}
    <div class="modal-backdrop" role="presentation" onpointerdown={cancelRestoreDefaultButtons}>
      <div
        class="modal"
        role="dialog"
        aria-modal="true"
        tabindex="0"
        onpointerdown={(event) => event.stopPropagation()}
      >
        <h3>{tt("settings.buttons.restoreConfirmTitle")}</h3>
        <p>{tt("settings.buttons.restoreConfirmBody")}</p>
        <div class="modal-actions">
          <button type="button" class="ghost-button" onclick={cancelRestoreDefaultButtons}>
            {tt("settings.buttons.cancel")}
          </button>
          <button
            type="button"
            class="ghost-button danger"
            onclick={confirmRestoreDefaultButtons}
          >
            {tt("settings.buttons.confirm")}
          </button>
        </div>
      </div>
    </div>
  {/if}

  {#if updateReadyPath}
    <div class="modal-backdrop" role="presentation">
      <div class="modal" role="dialog" aria-modal="true" tabindex="0">
        <h3>{tt("settings.update.ready")}</h3>
        <div class="modal-actions">
          <button
            type="button"
            class="ghost-button"
            onclick={() => (updateReadyPath = null)}
          >
            {tt("settings.buttons.cancel")}
          </button>
          <button type="button" class="ghost-button" onclick={confirmInstallUpdate}>
            {tt("settings.update.install")}
          </button>
        </div>
      </div>
    </div>
  {/if}
</main>

<style>
  .settings-page {
    height: 100vh;
    display: flex;
    flex-direction: column;
    background: var(--settings-bg);
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
    border: 1px solid var(--glass-border);
  }

  .ghost-button:hover {
    background: var(--button-bg-hover);
  }

  .ghost-button.danger {
    color: var(--danger);
    border-color: var(--danger);
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
    background-color: var(--control-bg);
    color: var(--text);
    font-size: 13px;
  }

  select {
    appearance: none;
    -webkit-appearance: none;
    background-color: var(--control-bg);
    background-image: url("data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' width='10' height='6' viewBox='0 0 10 6'%3E%3Cpath d='M1 1l4 4 4-4' fill='none' stroke='%236b7280' stroke-width='1.5'/%3E%3C/svg%3E");
    background-repeat: no-repeat;
    background-position: right 8px center;
    padding-right: 28px;
  }

  select option {
    background-color: var(--control-bg);
    color: var(--text);
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

  .button-list {
    display: grid;
    grid-template-columns: 1fr;
    gap: 6px;
    max-height: 360px;
    overflow-y: auto;
  }

  .button-list-item {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 5px 8px;
    border: 1px solid var(--bar-border);
    border-radius: 6px;
    font-size: 13px;
    transition:
      opacity 0.12s ease,
      border-color 0.12s ease,
      background 0.12s ease,
      box-shadow 0.12s ease;
  }

  .button-list-item.is-hidden {
    opacity: 0.55;
  }

  .button-list-item.is-dragging {
    position: relative;
    z-index: 2;
    opacity: 0.78;
    box-shadow: 0 10px 22px rgba(0, 0, 0, 0.18);
  }

  .button-list-item.drag-over {
    box-shadow: inset 0 2px 0 var(--accent);
  }

  .drag-handle {
    width: 22px;
    flex: none;
    display: grid;
    place-items: center;
    color: var(--text-muted);
    cursor: grab;
    font-size: 14px;
    user-select: none;
  }

  .drag-handle:active {
    cursor: grabbing;
  }

  .button-list-item .index {
    width: 20px;
    color: var(--text-muted);
    font-size: 12px;
    flex: none;
  }

  .item-fields {
    display: flex;
    flex-wrap: wrap;
    gap: 6px;
    align-items: center;
  }

  .item-fields input {
    width: 130px;
    padding: 4px 6px;
    border: 1px solid var(--bar-border);
    border-radius: 4px;
    background-color: var(--control-bg);
    color: var(--text);
    font-size: 13px;
  }

  .item-main {
    flex: 1;
    min-width: 0;
  }

  .param-input {
    width: 130px;
    padding: 4px 6px;
    border: 1px solid var(--bar-border);
    border-radius: 4px;
    background-color: var(--control-bg);
    color: var(--text);
    font-size: 13px;
  }

  .visibility-toggle {
    display: inline-flex;
    align-items: center;
    gap: 5px;
    flex: none;
    margin-left: auto;
    color: var(--text-muted);
    font-size: 12px;
  }

  .visibility-toggle input {
    accent-color: var(--accent);
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
    background-color: var(--control-bg);
    color: var(--text);
    font-size: 13px;
  }

  .add-row select {
    width: 180px;
    flex: none;
  }

  .custom-add-row {
    flex-wrap: wrap;
  }

  .add-row input:first-of-type {
    width: 150px;
    flex: none;
  }

  .add-row input:last-of-type {
    flex: 1;
    min-width: 0;
  }

  .color-section {
    margin-top: 12px;
  }

  .color-section-title {
    font-size: 13px;
    font-weight: 600;
    margin-bottom: 8px;
  }

  .color-presets {
    display: flex;
    flex-wrap: wrap;
    gap: 8px;
    margin-bottom: 12px;
  }

  .color-swatch {
    min-width: 72px;
    padding: 7px 10px;
    border: 1px solid var(--bar-border);
    border-radius: 8px;
    color: #1f2937;
    font-size: 12px;
    box-shadow: 0 1px 3px rgba(0, 0, 0, 0.12);
  }

  .color-swatch.selected {
    outline: 2px solid var(--accent);
    outline-offset: 2px;
  }

  .custom-color-row {
    display: flex;
    align-items: center;
    gap: 12px;
  }

  .custom-color-row label {
    font-size: 13px;
  }

  .custom-color-row input[type="color"] {
    width: 44px;
    height: 32px;
    padding: 2px;
    border: 1px solid var(--bar-border);
    border-radius: 6px;
    background: var(--button-bg);
  }

  .custom-preview {
    display: inline-flex;
    align-items: center;
    gap: 6px;
    font-size: 12px;
    color: var(--text-muted);
  }

  .swatch-mini {
    width: 18px;
    height: 18px;
    display: inline-block;
    border: 1px solid var(--bar-border);
    border-radius: 4px;
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

  .toggle-hint {
    display: block;
    margin-top: 2px;
    font-size: 11px;
    color: var(--text-muted);
  }

  .toggle-row input {
    accent-color: var(--accent);
  }

  .settings-footer {
    padding: 14px 24px;
    color: var(--text-muted);
    font-size: 12px;
  }

  .settings-footer p {
    margin-bottom: 4px;
  }

  .settings-footer p:last-child {
    margin-bottom: 0;
  }

  .settings-footer a {
    color: var(--accent);
    text-decoration: none;
  }

  .settings-footer a:hover {
    text-decoration: underline;
  }

  .modal-backdrop {
    position: fixed;
    inset: 0;
    z-index: 100;
    display: flex;
    align-items: center;
    justify-content: center;
    background: rgba(0, 0, 0, 0.35);
  }

  .modal {
    width: 340px;
    max-width: calc(100vw - 48px);
    padding: 18px 20px;
    border: 1px solid var(--bar-border);
    border-radius: 10px;
    background: var(--settings-bg);
    color: var(--text);
    box-shadow: 0 16px 40px rgba(0, 0, 0, 0.25);
  }

  .modal h3 {
    font-size: 15px;
    font-weight: 600;
    margin-bottom: 8px;
  }

  .modal p {
    font-size: 13px;
    color: var(--text-muted);
    margin-bottom: 14px;
    line-height: 1.5;
  }

  .modal-actions {
    display: flex;
    justify-content: flex-end;
    gap: 10px;
  }

  .update-panel {
    margin-top: 10px;
    padding: 10px 12px;
    border: 1px solid var(--bar-border);
    border-radius: 8px;
    font-size: 13px;
  }

  .update-panel pre {
    margin: 6px 0;
    max-height: 120px;
    overflow: auto;
    font-size: 12px;
    white-space: pre-wrap;
    color: var(--text-muted);
  }

  .update-latest-hint {
    font-size: 12px;
    color: var(--success, #2f9e44);
    white-space: nowrap;
  }

  .donation-images {
    display: flex;
    gap: 24px;
    margin: 12px 0;
    flex-wrap: wrap;
  }

  .donation-images figure {
    text-align: center;
  }

  .donation-images img {
    width: 200px;
    height: 200px;
    object-fit: contain;
    border: 1px solid var(--bar-border);
    border-radius: 8px;
    background: #fff;
  }

  .donation-images figcaption {
    margin-top: 6px;
    font-size: 14px;
    font-weight: 700;
    color: var(--text-muted);
  }
</style>
