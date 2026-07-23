<script setup lang="ts">
import { computed, onMounted, ref, watch } from "vue";
import { useRoute, useRouter } from "vue-router";
import { open } from "@tauri-apps/plugin-dialog";
import { getCloseBehavior, setCloseBehavior } from "../api";
import { store, persistMeta, reloadColumns, applyColumnsPath } from "../store";
import { slugifyTag, tagColor } from "../prefs";
import AppTopbar from "../components/AppTopbar.vue";

type SectionId = "path" | "content" | "quit";

const SECTIONS: { id: SectionId; label: string; desc: string }[] = [
  { id: "path", label: "路径设置", desc: "专栏文档固定目录" },
  { id: "content", label: "内容管理", desc: "标签与专栏打标" },
  { id: "quit", label: "退出程序", desc: "关闭窗口时的行为" },
];

const route = useRoute();
const router = useRouter();

const section = computed<SectionId>(() => {
  const raw = String(route.params.section || "path");
  if (raw === "content" || raw === "quit" || raw === "path") return raw;
  return "path";
});

const notice = ref("");
const newName = ref("");
const newColor = ref("#0f5c4c");
const columnsPathInput = ref("");
const pathBusy = ref(false);

const closeChoice = ref<"ask" | "tray" | "quit">("ask");
const quitBusy = ref(false);

watch(
  () => store.pathInfo?.fixedColumnsPath || store.pathInfo?.columnsPath,
  (path) => {
    if (path && !pathBusy.value) columnsPathInput.value = path;
  },
  { immediate: true }
);

onMounted(async () => {
  const path = store.pathInfo?.fixedColumnsPath || store.pathInfo?.columnsPath;
  if (path) columnsPathInput.value = path;
  try {
    const behavior = await getCloseBehavior();
    closeChoice.value = behavior === "tray" || behavior === "quit" ? behavior : "ask";
  } catch {
    closeChoice.value = "ask";
  }
});

function goSection(id: SectionId) {
  notice.value = "";
  if (id === section.value) return;
  router.push({ name: "settings", params: { section: id } });
}

async function addTag() {
  const name = newName.value.trim();
  if (!name) return;
  let id = slugifyTag(name);
  const ids = new Set(store.meta.tags.map((t) => t.id));
  if (ids.has(id)) id = `${id}-${Date.now().toString(36)}`;
  store.meta.tags.push({ id, name, color: newColor.value || tagColor(name) });
  await persistMeta();
  newName.value = "";
  notice.value = "已添加标签";
}

async function saveTag(id: string, name: string, color: string) {
  const tag = store.meta.tags.find((t) => t.id === id);
  if (!tag || !name.trim()) return;
  tag.name = name.trim();
  tag.color = color;
  await persistMeta();
  notice.value = `已更新标签「${tag.name}」`;
}

async function deleteTag(id: string) {
  if (!confirm("删除该标签？相关专栏上的该标签也会移除。")) return;
  store.meta.tags = store.meta.tags.filter((t) => t.id !== id);
  for (const slug of Object.keys(store.meta.columnTags)) {
    store.meta.columnTags[slug] = (store.meta.columnTags[slug] || []).filter((x) => x !== id);
  }
  await persistMeta();
  notice.value = "已删除标签";
}

function isChecked(slug: string, tagId: string) {
  return (store.meta.columnTags[slug] || []).includes(tagId);
}

function toggleTag(slug: string, tagId: string, checked: boolean) {
  const cur = new Set(store.meta.columnTags[slug] || []);
  if (checked) cur.add(tagId);
  else cur.delete(tagId);
  if (cur.size) store.meta.columnTags[slug] = [...cur];
  else delete store.meta.columnTags[slug];
}

function onToggle(slug: string, tagId: string, e: Event) {
  toggleTag(slug, tagId, (e.target as HTMLInputElement).checked);
}

async function saveAll() {
  await persistMeta();
  notice.value = "专栏标签已保存";
}

async function reload() {
  await reloadColumns();
  notice.value = "已重新扫描专栏目录";
}

async function savePath() {
  const next = columnsPathInput.value.trim();
  if (!next) {
    notice.value = "路径不能为空，或点「恢复默认」";
    return;
  }
  pathBusy.value = true;
  try {
    await applyColumnsPath(next);
    columnsPathInput.value = store.pathInfo?.columnsPath || next;
    notice.value = `已切换专栏目录，共 ${store.columns.length} 个专栏`;
  } catch (e) {
    notice.value = `切换失败：${e}`;
  } finally {
    pathBusy.value = false;
  }
}

async function pickFolder() {
  pathBusy.value = true;
  try {
    const selected = await open({
      directory: true,
      multiple: false,
      title: "选择专栏文档目录（子文件夹 = 专栏）",
      defaultPath: columnsPathInput.value || store.pathInfo?.defaultPath,
    });
    if (typeof selected === "string" && selected) {
      columnsPathInput.value = selected;
      await applyColumnsPath(selected);
      notice.value = `已切换到所选目录，共 ${store.columns.length} 个专栏`;
    }
  } catch (e) {
    notice.value = `选择文件夹失败：${e}`;
  } finally {
    pathBusy.value = false;
  }
}

async function resetPath() {
  pathBusy.value = true;
  try {
    await applyColumnsPath(null);
    columnsPathInput.value = store.pathInfo?.columnsPath || "";
    notice.value = `已恢复默认目录，共 ${store.columns.length} 个专栏`;
  } catch (e) {
    notice.value = `恢复失败：${e}`;
  } finally {
    pathBusy.value = false;
  }
}

async function saveQuitBehavior() {
  quitBusy.value = true;
  try {
    const next = closeChoice.value === "ask" ? null : closeChoice.value;
    await setCloseBehavior(next);
    notice.value =
      closeChoice.value === "ask"
        ? "已改为每次关闭时询问"
        : closeChoice.value === "tray"
          ? "已记住：关闭时最小化到托盘"
          : "已记住：关闭时退出程序";
  } catch (e) {
    notice.value = `保存失败：${e}`;
  } finally {
    quitBusy.value = false;
  }
}
</script>

<template>
  <AppTopbar />
  <main class="main settings-main">
    <div class="breadcrumb">
      <router-link to="/">全部专栏</router-link> / 设置
    </div>
    <section class="hero hero-compact">
      <h1>设置</h1>
      <p>路径、内容与退出行为集中管理，与主流桌面软件习惯一致。</p>
    </section>
    <div v-if="notice" class="admin-notice settings-notice">{{ notice }}</div>

    <div class="settings-layout">
      <nav class="settings-tree" aria-label="设置分类">
        <button
          v-for="item in SECTIONS"
          :key="item.id"
          type="button"
          class="settings-tree-item"
          :class="{ active: section === item.id }"
          @click="goSection(item.id)"
        >
          <strong>{{ item.label }}</strong>
          <small>{{ item.desc }}</small>
        </button>
      </nav>

      <div class="settings-panel">
        <template v-if="section === 'path'">
          <section class="admin-section">
            <h2>路径设置</h2>
            <p class="muted">
              这里改的是<strong>固定路径</strong>（会写入设置）。临时看别的目录请用顶栏「加载 → 临时浏览」，不会改这里。
              正式版默认
              <code>NoteReader/columns</code>
              （开发模式为项目内
              <code>content/columns</code>
              ）；标签保存在对应内容根目录的
              <code>meta.json</code>。
            </p>
            <div class="path-row">
              <input
                v-model="columnsPathInput"
                type="text"
                class="path-input"
                spellcheck="false"
                placeholder="专栏目录绝对路径"
                :disabled="pathBusy"
              />
              <button type="button" :disabled="pathBusy" @click="pickFolder">浏览…</button>
              <button type="button" class="primary" :disabled="pathBusy" @click="savePath">应用</button>
              <button type="button" :disabled="pathBusy" @click="resetPath">恢复默认</button>
            </div>
            <dl class="path-meta">
              <div>
                <dt>固定</dt>
                <dd><code>{{ store.pathInfo?.fixedColumnsPath || "—" }}</code></dd>
              </div>
              <div>
                <dt>当前</dt>
                <dd>
                  <code>{{ store.pathInfo?.columnsPath || "—" }}</code>
                  <span v-if="store.pathInfo?.isSessionBrowse" class="path-session-tag">临时浏览中</span>
                </dd>
              </div>
              <div>
                <dt>默认</dt>
                <dd><code>{{ store.pathInfo?.defaultPath || "—" }}</code></dd>
              </div>
              <div>
                <dt>模式</dt>
                <dd>{{ store.pathInfo?.isCustom ? "自定义固定路径" : "默认用户目录" }}</dd>
              </div>
            </dl>
          </section>
        </template>

        <template v-else-if="section === 'content'">
          <section class="admin-section">
            <h2>标签库</h2>
            <form class="tag-create" @submit.prevent="addTag">
              <input v-model="newName" type="text" placeholder="新标签名称，如 Java" required />
              <input v-model="newColor" type="color" title="颜色" />
              <button type="submit">添加标签</button>
            </form>
            <ul class="tag-admin-list">
              <li v-for="t in store.meta.tags" :key="t.id">
                <span class="tag" :style="{ '--tag': t.color }">{{ t.name }}</span>
                <input
                  class="tag-name-input"
                  type="text"
                  v-model="t.name"
                  @change="saveTag(t.id, t.name, t.color)"
                />
                <input type="color" v-model="t.color" @change="saveTag(t.id, t.name, t.color)" />
                <button type="button" class="danger" @click="deleteTag(t.id)">删除</button>
              </li>
              <li v-if="!store.meta.tags.length" class="muted">暂无标签</li>
            </ul>
          </section>

          <section class="admin-section">
            <h2>专栏打标</h2>
            <p class="muted">每个专栏可勾选多个标签，改完后点「保存全部」。</p>
            <div class="admin-columns">
              <div v-for="col in store.columns" :key="col.slug" class="admin-col">
                <div class="admin-col-head">
                  <strong>{{ col.title }}</strong>
                  <span class="meta">{{ col.articleCount }} 篇</span>
                </div>
                <div class="admin-col-tags">
                  <label v-for="t in store.meta.tags" :key="t.id" class="check-tag">
                    <input
                      type="checkbox"
                      :checked="isChecked(col.slug, t.id)"
                      @change="onToggle(col.slug, t.id, $event)"
                    />
                    <span class="tag" :style="{ '--tag': t.color }">{{ t.name }}</span>
                  </label>
                  <span v-if="!store.meta.tags.length" class="muted">请先创建标签</span>
                </div>
              </div>
            </div>
            <div class="admin-actions">
              <button type="button" class="primary" @click="saveAll">保存全部</button>
              <button type="button" @click="reload">重新扫描</button>
            </div>
          </section>
        </template>

        <template v-else>
          <section class="admin-section">
            <h2>退出程序</h2>
            <p class="muted">
              点击窗口关闭按钮时的默认行为。选择「每次询问」后，下次关闭仍会弹出确认框。
            </p>
            <div class="quit-options" role="radiogroup" aria-label="关闭窗口行为">
              <label class="quit-option" :class="{ selected: closeChoice === 'ask' }">
                <input v-model="closeChoice" type="radio" value="ask" />
                <span>
                  <strong>每次询问</strong>
                  <small>关闭时弹出：最小化到托盘 / 退出程序</small>
                </span>
              </label>
              <label class="quit-option" :class="{ selected: closeChoice === 'tray' }">
                <input v-model="closeChoice" type="radio" value="tray" />
                <span>
                  <strong>最小化到托盘</strong>
                  <small>关闭窗口后程序在后台继续运行</small>
                </span>
              </label>
              <label class="quit-option" :class="{ selected: closeChoice === 'quit' }">
                <input v-model="closeChoice" type="radio" value="quit" />
                <span>
                  <strong>退出程序</strong>
                  <small>关闭窗口即完全退出应用</small>
                </span>
              </label>
            </div>
            <div class="admin-actions">
              <button type="button" class="primary" :disabled="quitBusy" @click="saveQuitBehavior">
                保存
              </button>
            </div>
          </section>
        </template>
      </div>
    </div>
  </main>
</template>
