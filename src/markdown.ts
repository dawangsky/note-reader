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

marked.setOptions({ gfm: true, breaks: false });
marked.use({
  renderer: {
    code({ text, lang }: { text: string; lang?: string }) {
      const language = (lang || "").trim().split(/\s+/)[0] || "";
      let highlighted = text;
      try {
        if (language && hljs.getLanguage(language)) {
          highlighted = hljs.highlight(text, { language }).value;
        } else {
          highlighted = hljs.highlightAuto(text).value;
        }
      } catch {
        highlighted = text
          .replace(/&/g, "&amp;")
          .replace(/</g, "&lt;")
          .replace(/>/g, "&gt;");
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
