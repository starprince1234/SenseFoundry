<script lang="ts">
  import { onMount } from 'svelte'; import { api } from '$lib/api';
  type Span = { start_char: number; end_char: number }; type Card = { id: string; sentence_text: string; status: string; target_spans: Span[] };
  let cards = $state<Card[]>([]); let status = $state(''); let page = $state(1); const pageSize = 20;
  async function load() { const query = new URLSearchParams({ page: String(page), page_size: String(pageSize) }); if (status) query.set('status', status); const result = await api<{ items: Card[] }>(`/cards?${query}`); cards = result.items; }
  onMount(load);
</script>
<h1>Corpus card verification</h1>
<label>Status <select bind:value={status} onchange={() => { page = 1; void load(); }}><option value="">All</option><option>PROCESSING</option><option>VERIFIED</option><option>REJECTED</option></select></label>
<table><thead><tr><th>Sentence</th><th>Target spans</th><th>Status</th></tr></thead><tbody>{#each cards as card}<tr><td>{card.sentence_text}</td><td>{card.target_spans.map((span) => `${span.start_char}-${span.end_char}`).join(', ')}</td><td>{card.status}</td></tr>{/each}</tbody></table>
<button disabled={page === 1} onclick={() => { page -= 1; void load(); }}>Previous</button><span> Page {page} </span><button disabled={cards.length < pageSize} onclick={() => { page += 1; void load(); }}>Next</button>
<style>table{width:100%;border-collapse:collapse;margin:1rem 0}th,td{text-align:left;padding:.7rem;border-bottom:1px solid #ccd7d3}</style>
