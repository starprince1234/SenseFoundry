import { e as escape_html } from './index.js-UTZZBTT-.js';
import '@sveltejs/kit';
import '@sveltejs/kit/internal';
import '@sveltejs/kit/internal/server';

function _layout($$renderer, $$props) {
  $$renderer.component(($$renderer2) => {
    let { children } = $$props;
    $$renderer2.push(`<nav aria-label="Primary navigation" class="svelte-12qhfyh"><a href="/" class="svelte-12qhfyh">SenseFoundry</a><a href="/cards" class="svelte-12qhfyh">Cards</a><a href="/clusters" class="svelte-12qhfyh">Clusters</a> <a href="/definitions" class="svelte-12qhfyh">Definitions</a><a href="/publish" class="svelte-12qhfyh">Publish</a> <span class="spacer svelte-12qhfyh"></span><span>${escape_html("guest")}</span> `);
    {
      $$renderer2.push("<!--[-1-->");
    }
    $$renderer2.push(`<!--]--></nav> <main class="svelte-12qhfyh">`);
    children($$renderer2);
    $$renderer2.push(`<!----></main>`);
  });
}

export { _layout as default };
//# sourceMappingURL=_layout.svelte-D9GbI9WJ.js.map
