import mammoth from "mammoth";
import { readFileBytes } from "./api";

export async function renderDocx(path: string): Promise<string> {
  const bytes = await readFileBytes(path);
  const arrayBuffer = bytes.buffer.slice(
    bytes.byteOffset,
    bytes.byteOffset + bytes.byteLength
  ) as ArrayBuffer;
  const result = await mammoth.convertToHtml(
    { arrayBuffer },
    { convertImage: mammoth.images.dataUri }
  );
  const warnings = result.messages.filter((m) => m.type === "warning");
  const note =
    warnings.length > 0
      ? `<p class="doc-warn">部分 Word 样式可能未完整还原（${warnings.length}）</p>`
      : "";
  return `${note}<div class="docx-body">${result.value}</div>`;
}
