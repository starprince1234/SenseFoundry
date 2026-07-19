<script lang="ts">
  import { api } from '$lib/api';
  type Cluster = { id: string; label: string; member_count: number; stability_score: number };
  let headword = $state(''); let clusters = $state<Cluster[]>([]); let error = $state('');
  async function search() { error = ''; try { clusters = await api<Cluster[]>(`/clusters?headword=${encodeURIComponent(headword)}`); } catch (reason) { error = reason instanceof Error ? reason.message : 'Request failed'; } }
</script>
<h1>Sense clusters</h1><form onsubmit={(event) => { event.preventDefault(); void search(); }}><label>Headword <input bind:value={headword} required /></label><button>Load clusters</button></form>
{#if error}<p role="alert">{error}</p>{/if}
<section>{#each clusters as cluster}<article><h2>{cluster.label}</h2><p>{cluster.member_count} members</p><meter min="0" max="1" value={cluster.stability_score}>{cluster.stability_score}</meter><span> stability {cluster.stability_score.toFixed(2)}</span></article>{/each}</section>
<style>section{display:grid;grid-template-columns:repeat(auto-fit,minmax(220px,1fr));gap:1rem;margin-top:1rem}article{background:white;padding:1rem;border-radius:.5rem}</style>
