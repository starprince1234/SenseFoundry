<script lang="ts">
  import { getSession, login } from '$lib/auth';
</script>
<svelte:head><title>SenseFoundry</title></svelte:head>
<h1>Evidence-led lexicography</h1>
<p>Review corpus cards, refine sense clusters, and publish signed dictionary editions.</p>
{#if !getSession()}<button onclick={login}>Sign in with Keycloak</button>{/if}
