import { b as attr, a as ensure_array_like, e as escape_html } from './index.js-UTZZBTT-.js';
import '@sveltejs/kit';
import '@sveltejs/kit/internal';
import '@sveltejs/kit/internal/server';

function _page($$renderer, $$props) {
  $$renderer.component(($$renderer2) => {
    let headword = "";
    let clusters = [];
    $$renderer2.push(`<h1>Sense clusters</h1><form><label>Headword <input${attr("value", headword)} required=""/></label><button>Load clusters</button></form> `);
    {
      $$renderer2.push("<!--[-1-->");
    }
    $$renderer2.push(`<!--]--> <section class="svelte-nm65ug"><!--[-->`);
    const each_array = ensure_array_like(clusters);
    for (let $$index = 0, $$length = each_array.length; $$index < $$length; $$index++) {
      let cluster = each_array[$$index];
      $$renderer2.push(`<article class="svelte-nm65ug"><h2>${escape_html(cluster.label)}</h2><p>${escape_html(cluster.member_count)} members</p><meter min="0" max="1"${attr("value", cluster.stability_score)}>${escape_html(cluster.stability_score)}</meter><span>stability ${escape_html(cluster.stability_score.toFixed(2))}</span></article>`);
    }
    $$renderer2.push(`<!--]--></section>`);
  });
}

export { _page as default };
//# sourceMappingURL=_page.svelte-Bd2F7ef-.js.map
