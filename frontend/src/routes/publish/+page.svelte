<script lang="ts">
  import { api } from '$lib/api';
  type PublicationPreview = { headword: string; approvals: number; diff: Record<string, unknown>; content: unknown };
  let headword = $state(''); let preview = $state<PublicationPreview | null>(null); let signed = $state(false); let status = $state('');
  async function load() { preview = await api<PublicationPreview>(`/publication-preview?headword=${encodeURIComponent(headword)}`); signed = false; }
  async function publish() { if (!preview || preview.approvals < 2 || !signed) return; await api('/publications', { method: 'POST', body: JSON.stringify({ headword: preview.headword, content: preview.content, signed: true }) }); status = 'Edition published'; }
</script>
<h1>Publish edition</h1><form onsubmit={(event) => { event.preventDefault(); void load(); }}><label>Headword <input bind:value={headword} required /></label><button>Preview</button></form>
{#if preview}<p class:ready={preview.approvals >= 2}>{preview.approvals}/2 approvals</p><h2>Edition diff</h2><pre>{JSON.stringify(preview.diff, null, 2)}</pre><label><input type="checkbox" bind:checked={signed} /> Sign this exact diff</label><button disabled={preview.approvals < 2 || !signed} onclick={publish}>Sign and publish</button><p aria-live="polite">{status}</p>{/if}
<style>pre{white-space:pre-wrap;background:#17221f;color:#d8efe6;padding:1rem}.ready{color:#08734f;font-weight:bold}</style>
