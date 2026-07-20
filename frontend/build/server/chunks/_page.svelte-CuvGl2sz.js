import { a as ensure_array_like, e as escape_html, b as attr, p as public_env } from './index.js-UTZZBTT-.js';
import '@sveltejs/kit';
import '@sveltejs/kit/internal';
import '@sveltejs/kit/internal/server';

async function api(path, init) {
  const response = await fetch(`${public_env.PUBLIC_API_URL ?? "http://localhost:8080"}${path}`, {
    ...init,
    headers: { "Content-Type": "application/json", ...{}, ...init?.headers }
  });
  if (!response.ok) throw new Error(`API request failed: ${response.status}`);
  return response.json();
}
function _page($$renderer, $$props) {
  $$renderer.component(($$renderer2) => {
    let cards = [];
    let status = "";
    let page = 1;
    const pageSize = 20;
    async function load() {
      const query = new URLSearchParams({ page: String(page), page_size: String(pageSize) });
      const result = await api(`/cards?${query}`);
      cards = result.items;
    }
    $$renderer2.push(`<h1>Corpus card verification</h1> <label>Status `);
    $$renderer2.select(
      {
        value: status,
        onchange: () => {
          page = 1;
          void load();
        }
      },
      ($$renderer3) => {
        $$renderer3.option({ value: "" }, ($$renderer4) => {
          $$renderer4.push(`All`);
        });
        $$renderer3.option({}, ($$renderer4) => {
          $$renderer4.push(`PROCESSING`);
        });
        $$renderer3.option({}, ($$renderer4) => {
          $$renderer4.push(`VERIFIED`);
        });
        $$renderer3.option({}, ($$renderer4) => {
          $$renderer4.push(`REJECTED`);
        });
      }
    );
    $$renderer2.push(`</label> <table class="svelte-qmtet4"><thead><tr><th class="svelte-qmtet4">Sentence</th><th class="svelte-qmtet4">Target spans</th><th class="svelte-qmtet4">Status</th></tr></thead><tbody><!--[-->`);
    const each_array = ensure_array_like(cards);
    for (let $$index = 0, $$length = each_array.length; $$index < $$length; $$index++) {
      let card = each_array[$$index];
      $$renderer2.push(`<tr><td class="svelte-qmtet4">${escape_html(card.sentence_text)}</td><td class="svelte-qmtet4">${escape_html(card.target_spans.map((span) => `${span.start_char}-${span.end_char}`).join(", "))}</td><td class="svelte-qmtet4">${escape_html(card.status)}</td></tr>`);
    }
    $$renderer2.push(`<!--]--></tbody></table> <button${attr("disabled", page === 1, true)}>Previous</button><span>Page ${escape_html(page)}</span><button${attr("disabled", cards.length < pageSize, true)}>Next</button>`);
  });
}

export { _page as default };
//# sourceMappingURL=_page.svelte-CuvGl2sz.js.map
