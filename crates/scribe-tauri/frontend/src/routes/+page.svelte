<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";

  let pingResult = $state<string>("");
  let busy = $state(false);

  async function ping() {
    busy = true;
    try {
      pingResult = await invoke<string>("ping");
    } catch (e) {
      pingResult = `error: ${e}`;
    } finally {
      busy = false;
    }
  }
</script>

<main class="flex min-h-screen flex-col items-center justify-center gap-6 p-8">
  <h1 class="text-4xl font-semibold tracking-tight">Scribe</h1>
  <p class="text-neutral-600">Markdown → Word, no manual cleanup.</p>

  <button
    class="rounded-full bg-neutral-900 px-6 py-2 text-white shadow-sm transition hover:bg-neutral-700 disabled:opacity-50"
    disabled={busy}
    onclick={ping}
  >
    {busy ? "Pinging…" : "Ping backend"}
  </button>

  {#if pingResult}
    <p class="font-mono text-sm text-neutral-500">→ {pingResult}</p>
  {/if}
</main>
