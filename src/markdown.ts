import { marked } from "marked";
import hljs from "highlight.js/lib/core";
import javascript from "highlight.js/lib/languages/javascript";
import typescript from "highlight.js/lib/languages/typescript";
import java from "highlight.js/lib/languages/java";
import python from "highlight.js/lib/languages/python";
import sql from "highlight.js/lib/languages/sql";
import bash from "highlight.js/lib/languages/bash";
import go from "highlight.js/lib/languages/go";
import xml from "highlight.js/lib/languages/xml";
import jsonLang from "highlight.js/lib/languages/json";
import { resolveAssetUrl } from "./api";
import { prefs } from "./prefs";

hljs.registerLanguage("javascript", javascript);
hljs.registerLanguage("js", javascript);
hljs.registerLanguage("typescript", typescript);
hljs.registerLanguage("ts", typescript);
hljs.registerLanguage("java", java);
hljs.registerLanguage("python", python);
hljs.registerLanguage("py", python);
hljs.registerLanguage("sql", sql);
hljs.registerLanguage("bash", bash);
hljs.registerLanguage("shell", bash);
hljs.registerLanguage("sh", bash);
hljs.registerLanguage("go", go);
hljs.registerLanguage("xml", xml);
hljs.registerLanguage("html", xml);
hljs.registerLanguage("json", jsonLang);

const MERMAID_START =
  /^(flowchart|graph|sequenceDiagram|classDiagram|stateDiagram(?:-v2)?|erDiagram|journey|gantt|pie|mindmap|timeline|quadrantChart|gitGraph|C4Context|requirementDiagram|sankey-beta|xychart-beta|block-beta)\b/;

function escapeHtml(text: string): string {
  return text
    .replace(/&/g, "&amp;")
    .replace(/</g, "&lt;")
    .replace(/>/g, "&gt;")
    .replace(/"/g, "&quot;");
}

function isMermaidBlock(lang: string, text: string): boolean {
  if (lang === "mermaid" || lang === "mmd") return true;
  if (lang) return false;
  return MERMAID_START.test(text.trim());
}

function mermaidBlockHtml(text: string): string {
  const source = text.replace(/\n$/, "");
  return `<div class="mermaid" data-mermaid="${encodeURIComponent(source)}">${escapeHtml(source)}</div>\n`;
}

marked.setOptions({ gfm: true, breaks: false });
marked.use({
  renderer: {
    code({ text, lang }: { text: string; lang?: string }) {
      const language = (lang || "").trim().split(/\s+/)[0] || "";
      if (isMermaidBlock(language, text)) {
        return mermaidBlockHtml(text);
      }
      let highlighted = text;
      try {
        if (language && hljs.getLanguage(language)) {
          highlighted = hljs.highlight(text, { language }).value;
        } else {
          highlighted = hljs.highlightAuto(text).value;
        }
      } catch {
        highlighted = escapeHtml(text);
      }
      const cls = language ? ` class="hljs language-${language}"` : ' class="hljs"';
      return `<pre><code${cls}>${highlighted}</code></pre>\n`;
    },
  },
});

export async function renderMarkdown(md: string, columnDir: string): Promise<string> {
  const rewritten = await rewriteImages(md, columnDir);
  return marked.parse(rewritten) as string | Promise<string>;
}

let mermaidConfiguredFor: string | null = null;

async function loadMermaid() {
  const mod = await import("mermaid");
  return mod.default;
}

async function ensureMermaid() {
  const mermaid = await loadMermaid();
  const theme = prefs.theme === "ink" ? "neutral" : "default";
  if (mermaidConfiguredFor === theme) return mermaid;
  mermaid.initialize({
    startOnLoad: false,
    theme,
    securityLevel: "loose",
    fontFamily: "inherit",
  });
  mermaidConfiguredFor = theme;
  return mermaid;
}

/** Render / re-render Mermaid diagrams inside a mounted article root. */
export async function hydrateMermaid(root: ParentNode): Promise<void> {
  const nodes = [...root.querySelectorAll<HTMLElement>(".mermaid")];
  if (!nodes.length) return;

  const mermaid = await ensureMermaid();

  for (const el of nodes) {
    const encoded = el.getAttribute("data-mermaid");
    if (encoded) {
      try {
        el.textContent = decodeURIComponent(encoded);
      } catch {
        /* keep existing text */
      }
    }
    el.removeAttribute("data-processed");
    el.removeAttribute("data-error");
  }

  try {
    await mermaid.run({ nodes });
  } catch (e) {
    console.warn("mermaid render failed", e);
  }
}

async function rewriteImages(md: string, columnDir: string): Promise<string> {
  const re = /!\[([^\]]*)\]\(([^)]+)\)/g;
  const matches = [...md.matchAll(re)];
  let out = md;
  for (const m of matches) {
    const full = m[0];
    const alt = m[1];
    let src = m[2].trim().replace(/^<|>$/g, "");
    if (/^https?:\/\//i.test(src) || src.startsWith("data:") || src.startsWith("asset:")) {
      continue;
    }
    if (src.startsWith("./")) src = src.slice(2);
    try {
      const url = await resolveAssetUrl(columnDir, src);
      out = out.replace(full, `![${alt}](${url})`);
    } catch {
      /* keep original */
    }
  }
  return out;
}
