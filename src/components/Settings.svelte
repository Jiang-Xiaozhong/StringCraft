<script lang="ts">
  import { onMount } from "svelte";
  import { getCurrentWindow } from "@tauri-apps/api/window";
  import { getConfig } from "../lib/api";
  import { DEFAULT_CONFIG } from "../lib/defaults";
  import type { AppConfig } from "../lib/types";

  let config: AppConfig = $state(DEFAULT_CONFIG);
  let loadError: string | null = $state(null);

  const win = getCurrentWindow();

  onMount(() => {
    getConfig()
      .then((cfg) => {
        config = cfg;
      })
      .catch((err) => {
        loadError = String(err);
      });
  });
</script>

<main class="settings-page">
  <header class="settings-header">
    <h1>StringCraft 设置</h1>
    <button type="button" class="ghost-button" onclick={() => win.close()}>关闭</button>
  </header>

  {#if loadError}
    <p class="error-text">读取配置失败：{loadError}</p>
  {/if}

  <section class="settings-section">
    <h2>全局快捷键</h2>
    <p class="hint">M1 阶段仅展示默认快捷键，录制功能在 M4 实现。</p>
    <div class="field-row">
      <label for="hotkey">呼入/呼出悬浮条</label>
      <input id="hotkey" type="text" value={config.hotkey} disabled />
    </div>
  </section>

  <section class="settings-section">
    <h2>按钮管理</h2>
    <p class="hint">当前内置 {config.buttons.length} 个转换按钮，增删/排序/改文字在 M4 实现。</p>
    <div class="button-list">
      {#each config.buttons as button, index (button.id)}
        <div class="button-list-item">
          <span class="index">{index + 1}</span>
          <span class="label">{button.label}</span>
          <code>{button.transform}</code>
        </div>
      {/each}
    </div>
  </section>

  <section class="settings-section">
    <h2>外观</h2>
    <p class="hint">按钮尺寸、字号、主题等外观设置在 M4 实现。</p>
    <div class="field-grid">
      <div class="field-row">
        <label for="button-width">按钮宽度</label>
        <input id="button-width" type="number" value={config.buttonWidth} disabled />
      </div>
      <div class="field-row">
        <label for="button-height">按钮高度</label>
        <input id="button-height" type="number" value={config.buttonHeight} disabled />
      </div>
      <div class="field-row">
        <label for="font-size">按钮字号</label>
        <input id="font-size" type="number" value={config.fontSize} disabled />
      </div>
      <div class="field-row">
        <label for="rows">行数</label>
        <input id="rows" type="number" value={config.rows} disabled />
      </div>
    </div>
  </section>

  <footer class="settings-footer">
    <p>StringCraft v0.1.0 · M1 骨架阶段</p>
  </footer>
</main>
