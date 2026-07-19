<script lang="ts">
  import { onMount } from 'svelte';
  import { completeLogin, getSession, logout, type Session } from '$lib/auth';
  let { children } = $props();
  let session = $state<Session | null>(null);
  onMount(async () => { await completeLogin(); session = getSession(); });
</script>

<nav aria-label="Primary navigation">
  <a href="/">SenseFoundry</a><a href="/cards">Cards</a><a href="/clusters">Clusters</a>
  <a href="/definitions">Definitions</a><a href="/publish">Publish</a>
  <span class="spacer"></span><span>{session?.role ?? 'guest'}</span>
  {#if session}<button onclick={logout}>Logout</button>{/if}
</nav>
<main>{@render children()}</main>

<style>
  :global(body) { margin: 0; font-family: system-ui, sans-serif; color: #17221f; background: #f4f7f5; }
  nav { display: flex; gap: 1rem; align-items: center; padding: 1rem 2rem; background: #123c32; color: white; }
  nav a { color: white; text-decoration: none; } .spacer { flex: 1; } main { max-width: 1100px; margin: auto; padding: 2rem; }
</style>
