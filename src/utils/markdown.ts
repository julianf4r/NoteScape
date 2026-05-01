type ProseMirrorNode = {
  type?: string;
  text?: string;
  attrs?: Record<string, unknown>;
  marks?: Array<{ type?: string; attrs?: Record<string, unknown> }>;
  content?: ProseMirrorNode[];
};

function escapeText(value: string) {
  return value.replace(/\\/g, "\\\\").replace(/([*_`[\]])/g, "\\$1");
}

function escapeCode(value: string) {
  return value.replace(/`/g, "\\`");
}

function markText(text: string, marks: ProseMirrorNode["marks"]) {
  if (!marks?.length) return escapeText(text);
  return marks.reduce((current, mark) => {
    if (mark.type === "bold") return `**${current}**`;
    if (mark.type === "italic") return `*${current}*`;
    if (mark.type === "strike") return `~~${current}~~`;
    if (mark.type === "code") return `\`${escapeCode(text)}\``;
    if (mark.type === "link") {
      const href = typeof mark.attrs?.href === "string" ? mark.attrs.href : "";
      return href ? `[${current}](${href})` : current;
    }
    return current;
  }, escapeText(text));
}

function inlineMarkdown(node: ProseMirrorNode): string {
  if (node.type === "text") return markText(node.text ?? "", node.marks);
  if (node.type === "hardBreak") return "  \n";
  return node.content?.map(inlineMarkdown).join("") ?? "";
}

function textContent(node: ProseMirrorNode): string {
  if (node.type === "text") return node.text ?? "";
  return node.content?.map(textContent).join("") ?? "";
}

function indentLines(value: string, prefix: string) {
  return value
    .split("\n")
    .map((line) => (line ? `${prefix}${line}` : line))
    .join("\n");
}

function listItemMarkdown(node: ProseMirrorNode, index: number, ordered: boolean, depth: number) {
  const marker = ordered ? `${index + 1}. ` : "- ";
  const taskChecked = node.type === "taskItem" ? node.attrs?.checked === true : undefined;
  const taskMarker = taskChecked === undefined ? marker : `- [${taskChecked ? "x" : " "}] `;
  const prefix = "  ".repeat(depth) + taskMarker;
  const children = node.content ?? [];
  const [firstChild, ...restChildren] = children;
  const firstLine = firstChild ? nodeMarkdown(firstChild, depth).trim() : "";
  const rest = restChildren.map((child) => nodeMarkdown(child, depth + 1)).filter(Boolean);
  return [prefix + firstLine, ...rest.map((item) => indentLines(item, "  ".repeat(depth + 1)))].join("\n");
}

function nodeMarkdown(node: ProseMirrorNode, depth = 0): string {
  switch (node.type) {
    case "doc":
      return node.content?.map((child) => nodeMarkdown(child, depth)).filter(Boolean).join("\n\n") ?? "";
    case "paragraph":
      return inlineMarkdown(node);
    case "heading": {
      const level = Math.min(6, Math.max(1, Number(node.attrs?.level) || 1));
      return `${"#".repeat(level)} ${inlineMarkdown(node)}`;
    }
    case "blockquote":
      return indentLines(node.content?.map((child) => nodeMarkdown(child, depth)).join("\n\n") ?? "", "> ");
    case "bulletList":
      return node.content?.map((child, index) => listItemMarkdown(child, index, false, depth)).join("\n") ?? "";
    case "orderedList":
      return node.content?.map((child, index) => listItemMarkdown(child, index, true, depth)).join("\n") ?? "";
    case "taskList":
      return node.content?.map((child, index) => listItemMarkdown(child, index, false, depth)).join("\n") ?? "";
    case "listItem":
    case "taskItem":
      return listItemMarkdown(node, 0, false, depth);
    case "codeBlock": {
      const language = typeof node.attrs?.language === "string" ? node.attrs.language : "";
      return `\`\`\`${language}\n${textContent(node)}\n\`\`\``;
    }
    case "horizontalRule":
      return "---";
    default:
      return node.content?.map((child) => nodeMarkdown(child, depth)).filter(Boolean).join("\n\n") ?? inlineMarkdown(node);
  }
}

export function contentJsonToMarkdown(value: unknown) {
  if (!value || typeof value !== "object") return "";
  return nodeMarkdown(value as ProseMirrorNode).trim();
}
