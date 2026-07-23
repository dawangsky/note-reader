<script setup lang="ts">
import { onMounted, ref } from "vue";
import { open } from "@tauri-apps/plugin-dialog";
import { getPlatformInfo } from "../api";
import { finishOnboarding, store } from "../store";
import type { PlatformInfo } from "../types";

const busy = ref(false);
const error = ref("");
const pathInput = ref("");
const platform = ref<PlatformInfo | null>(null);

onMounted(async () => {
  pathInput.value = store.pathInfo?.defaultPath || store.pathInfo?.columnsPath || "";
  try {
    platform.value = await getPlatformInfo();
    if (!pathInput.value) pathInput.value = platform.value.defaultColumnsPath;
  } catch {
    /* ignore */
  }
});

const isWindows = () =>
  (platform.value?.os || store.pathInfo?.os || "") === "windows";

async function pickFolder() {
  const selected = await open({
    directory: true,
    multiple: false,
    title: "选择专栏文档读取目录",
    defaultPath: pathInput.value || platform.value?.dataHome,
  });
  if (typeof selected === "string" && selected) pathInput.value = selected;
}

async function useDefault() {
  busy.value = true;
  error.value = "";
  try {
    await finishOnboarding(null);
  } catch (e) {
    error.value = String(e);
  } finally {
    busy.value = false;
  }
}

async function useCustom() {
  const next = pathInput.value.trim();
  if (!next) {
    error.value = "请填写专栏读取路径";
    return;
  }
  busy.value = true;
  error.value = "";
  try {
    await finishOnboarding(next);
  } catch (e) {
    error.value = String(e);
  } finally {
    busy.value = false;
  }
}
</script>

<template>
  <div class="onboard-mask" role="dialog" aria-modal="true" aria-label="初始化路径">
    <div class="onboard-card">
      <h1>设置专栏读取路径</h1>
      <p class="onboard-lead">
        {{ platform?.installHint || "请确认本地专栏文档存放位置。" }}
      </p>

      <div v-if="isWindows()" class="onboard-tip">
        <strong>Windows 说明</strong>
        <ul>
          <li>安装向导里可选择<strong>软件安装路径</strong>（程序本体）。</li>
          <li>下面设置的是<strong>专栏读取路径</strong>（Markdown / Word / PDF 目录），默认为用户目录下的 <code>NoteReader\columns</code>。</li>
        </ul>
      </div>
      <div v-else class="onboard-tip">
        <strong>macOS / 其他</strong>
        <ul>
          <li>应用本体按系统习惯安装；专栏内容默认放在用户目录 <code>NoteReader/columns</code>。</li>
          <li>也可在此改成任意文件夹，之后仍可在「设置 → 路径设置」中调整。</li>
        </ul>
      </div>

      <label class="onboard-label">专栏读取路径</label>
      <div class="path-row">
        <input
          v-model="pathInput"
          type="text"
          class="path-input"
          spellcheck="false"
          :disabled="busy"
        />
        <button type="button" :disabled="busy" @click="pickFolder">浏览…</button>
      </div>
      <p class="muted">
        默认：<code>{{ platform?.defaultColumnsPath || store.pathInfo?.defaultPath }}</code>
      </p>
      <p v-if="error" class="onboard-error">{{ error }}</p>

      <div class="onboard-actions">
        <button type="button" class="primary" :disabled="busy" @click="useDefault">
          使用默认路径
        </button>
        <button type="button" :disabled="busy" @click="useCustom">使用上面的路径</button>
      </div>
    </div>
  </div>
</template>
