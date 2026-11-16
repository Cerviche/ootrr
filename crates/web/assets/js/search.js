const ALIASES = {
  shuffle_entrances: "er entrance randomizer",
  shuffle_smallkeys: "keysanity",
  shuffle_bosskeys: "keysanity bk",
  shuffle_skulltulas: "gs skulls tokens tokensanity",
  disabled_locations: "exclude excluded",
  shuffle_scrubs: "deku scrubsanity",
  shuffle_ganon_bosskey: "gbk lacs",
  logic_rules: "glitchless no logic",
  shuffle_songs: "songsanity",
  shuffle_shops: "shopsanity",
  shuffle_cows: "cowsanity",
  skip_cutscenes: "cutscene",
  skip_intro: "cutscene",
  skip_story_scenes: "cutscene",
  faster_chests: "fast chest animation",
};

function hay(label, tip, choices, extra) {
  return {
    label: label.toLowerCase(),
    opts: (choices || []).map((c) => c.label).join(" | ").toLowerCase(),
    rest: `${tip || ""} ${(choices || []).map((c) => c.tip || "").join(" ")} ${extra}`.toLowerCase(),
  };
}

export function build_index(schema, pages, fields) {
  const by_id = new Map(schema.settings.map((s) => [s.id, s]));
  const out = [];
  for (const p of pages) {
    for (const sec of p.sections) {
      for (const id of sec.ids) {
        const s = by_id.get(id);
        if (!s) continue;
        out.push({ kind: "setting", id, s, page: p, section: sec.label, h: hay(s.label, s.tip, s.choices, `${ALIASES[id] || ""} ${id} ${sec.label} ${p.label}`) });
      }
    }
  }
  for (const f of fields) {
    const page = { id: f.page, label: f.page_label, heading: f.heading };
    out.push({ kind: "field", id: f.id, f, page, section: f.section, h: hay(f.label, f.tip, f.choices, `${f.section} ${f.page_label}`) });
  }
  return out;
}

export function terms_of(q) {
  return q.toLowerCase().split(/\s+/).filter(Boolean);
}

export function search(index, q) {
  const terms = terms_of(q);
  const hits = [];
  if (!terms.length) return hits;
  for (const e of index) {
    let score = 0;
    let in_label = false;
    let opt = null;
    let ok = true;
    for (const t of terms) {
      if (e.h.label.includes(t)) {
        score += e.h.label.startsWith(t) ? 14 : 10;
        in_label = true;
      } else if (e.h.opts.includes(t)) {
        score += 5;
        const cs = (e.s || e.f).choices || [];
        opt = opt || cs.find((c) => c.label.toLowerCase().includes(t));
      } else if (e.h.rest.includes(t)) {
        score += 1;
      } else {
        ok = false;
        break;
      }
    }
    if (!ok) continue;
    let note = "";
    if (!in_label) note = opt ? `Matched the option “${opt.label}”` : "Matched in the description";
    hits.push({ ...e, score, note, terms });
  }
  return hits;
}

export function grouped(hits) {
  const by = new Map();
  for (const x of hits) {
    if (!by.has(x.page.id)) by.set(x.page.id, []);
    by.get(x.page.id).push(x);
  }
  return [...by.values()].map((xs) => xs.sort((a, b) => b.score - a.score)).sort((a, b) => b[0].score - a[0].score);
}
