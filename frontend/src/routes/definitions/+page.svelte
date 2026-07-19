<script lang="ts">
  import { api } from '$lib/api';
  type Card = { id: string; sentence_text: string }; type Example = { id: string; sentence: string; score: number; accepted?: boolean }; type Draft = { id: string; definition: string; evidence_cards: Card[]; examples: Example[] };
  let draftId = $state(''); let draft = $state<Draft | null>(null); let amendment = $state(''); let message = $state('');
  async function load() { draft = await api<Draft>(`/definition-drafts/${draftId}`); amendment = draft.definition; }
  async function save() { if (!draft) return; await api(`/definition-drafts/${draft.id}`, { method: 'PATCH', body: JSON.stringify({ definition: amendment, examples: draft.examples }) }); message = 'Expert amendment saved'; }
</script>
<h1>Definition editor</h1><form onsubmit={(event) => { event.preventDefault(); void load(); }}><label>Draft ID <input bind:value={draftId} required /></label><button>Open</button></form>
{#if draft}<label>Definition<textarea bind:value={amendment} rows="5"></textarea></label><h2>Evidence cards</h2>{#each draft.evidence_cards as card}<blockquote>{card.sentence_text}<small>{card.id}</small></blockquote>{/each}
<h2>Ranked examples</h2>{#each draft.examples as example}<article><span>{example.score.toFixed(2)} — {example.sentence}</span><button onclick={() => example.accepted = true}>Accept</button><button onclick={() => example.accepted = false}>Reject</button></article>{/each}<button onclick={save}>Save amendment</button><p aria-live="polite">{message}</p>{/if}
<style>textarea{display:block;width:100%;margin:.5rem 0}blockquote{background:white;padding:1rem}small{display:block;color:#65736e}article{display:flex;gap:.5rem;align-items:center;margin:.5rem 0}article span{flex:1}</style>
