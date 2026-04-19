<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";

  const SAMPLE = `# Scribe

Drop or type Markdown on the left — see the preview on the right.

## Inline formatting

**bold**, *italic*, ~~strike~~, \`code\`, and $E = mc^2$ math.

## A display equation

$$\\int_0^1 x^2 \\, dx = \\frac{1}{3}$$

## List

- one
- two
- three

## Code

\`\`\`rust
fn main() {
    println!("Hello, Scribe!");
}
\`\`\`
`;

  let markdown = $state<string>(SAMPLE);
  let previewHtml = $state<string>("");
  let busy = $state<boolean>(false);
  let error = $state<string | null>(null);

  let debounceHandle = $state<ReturnType<typeof setTimeout> | null>(null);

  $effect(() => {
    // Debounce: fire 250ms after typing stops.
    const snapshot = markdown;
    if (debounceHandle) clearTimeout(debounceHandle);
    debounceHandle = setTimeout(async () => {
      try {
        previewHtml = await invoke<string>("preview_html", { markdown: snapshot });
        error = null;
      } catch (e) {
        error = String(e);
      }
    }, 250);
  });

  async function exportDocx() {
    busy = true;
    try {
      const bytes = await invoke<number[]>("convert_string", { markdown });
      // Save via a temporary download — in a real Tauri app this would use
      // the dialog plugin to prompt for a destination path.
      const blob = new Blob([new Uint8Array(bytes)], {
        type: "application/vnd.openxmlformats-officedocument.wordprocessingml.document",
      });
      const url = URL.createObjectURL(blob);
      const a = document.createElement("a");
      a.href = url;
      a.download = "scribe.docx";
      a.click();
      URL.revokeObjectURL(url);
      error = null;
    } catch (e) {
      error = String(e);
    } finally {
      busy = false;
    }
  }
</script>

<div class="flex h-screen flex-col">
  <header
    class="flex items-center justify-between border-b border-neutral-200 bg-white/80 px-4 py-2 backdrop-blur"
  >
    <div class="flex items-center gap-2">
      <span class="text-lg font-semibold tracking-tight">Scribe</span>
      <span class="text-xs text-neutral-500">Markdown → Word</span>
    </div>
    <button
      class="rounded-md bg-neutral-900 px-4 py-1.5 text-sm text-white shadow-sm transition hover:bg-neutral-700 disabled:opacity-50"
      disabled={busy}
      onclick={exportDocx}
    >
      {busy ? "Exporting…" : "Export .docx"}
    </button>
  </header>

  {#if error}
    <div class="bg-red-50 px-4 py-2 text-sm text-red-800">
      {error}
    </div>
  {/if}

  <div class="flex flex-1 overflow-hidden">
    <div class="flex w-1/2 flex-col border-r border-neutral-200">
      <div class="flex-none bg-neutral-100 px-3 py-1 text-xs font-medium text-neutral-500">
        MARKDOWN
      </div>
      <textarea
        class="flex-1 resize-none bg-white p-4 font-mono text-sm leading-relaxed outline-none"
        bind:value={markdown}
        spellcheck="false"
      ></textarea>
    </div>
    <div class="flex w-1/2 flex-col">
      <div class="flex-none bg-neutral-100 px-3 py-1 text-xs font-medium text-neutral-500">
        PREVIEW
      </div>
      <div
        class="scribe-preview flex-1 overflow-auto bg-white p-6"
        role="document"
      >
        {@html previewHtml}
      </div>
    </div>
  </div>
</div>

<style>
  :global(.scribe-preview h1) {
    font-size: 1.75rem;
    font-weight: 600;
    margin-top: 1em;
    margin-bottom: 0.5em;
  }
  :global(.scribe-preview h2) {
    font-size: 1.4rem;
    font-weight: 600;
    margin-top: 1em;
    margin-bottom: 0.4em;
  }
  :global(.scribe-preview h3) {
    font-size: 1.2rem;
    font-weight: 600;
    margin-top: 0.8em;
  }
  :global(.scribe-preview p) {
    margin: 0.6em 0;
  }
  :global(.scribe-preview pre) {
    background: #f6f7f9;
    border-radius: 6px;
    padding: 0.75em 1em;
    overflow-x: auto;
    font-size: 0.875em;
  }
  :global(.scribe-preview code) {
    font-family: Menlo, Consolas, monospace;
    font-size: 0.875em;
  }
  :global(.scribe-preview pre code) {
    background: transparent;
  }
  :global(.scribe-preview p code, .scribe-preview li code) {
    background: #f0f1f3;
    padding: 0.1em 0.35em;
    border-radius: 3px;
  }
  :global(.scribe-preview ul, .scribe-preview ol) {
    margin: 0.5em 0;
    padding-left: 1.5em;
  }
  :global(.scribe-preview blockquote) {
    border-left: 3px solid #ccc;
    padding-left: 1em;
    color: #555;
    margin: 1em 0;
  }
  :global(.scribe-preview .math-block) {
    text-align: center;
    font-family: "Latin Modern Math", "Cambria Math", serif;
    margin: 1em 0;
  }
  :global(.scribe-preview .math-inline) {
    font-family: "Latin Modern Math", "Cambria Math", serif;
  }
  :global(.scribe-preview table) {
    border-collapse: collapse;
    margin: 1em 0;
  }
  :global(.scribe-preview th, .scribe-preview td) {
    border: 1px solid #aaa;
    padding: 0.3em 0.6em;
  }
  :global(.scribe-preview a) {
    color: #0563c1;
    text-decoration: underline;
  }
  :global(.scribe-preview hr) {
    border: none;
    border-top: 1px solid #ccc;
    margin: 2em 0;
  }
</style>

