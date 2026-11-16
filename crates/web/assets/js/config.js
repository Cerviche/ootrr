import { h, fill, debounce } from "./dom.js";
import { get, q } from "./api.js";
import { state, touch } from "./store.js";
import { pick_folder } from "./picker.js";
import { tip_button } from "./tooltip.js";
import { marked } from "./controls.js";

const NAME_RE = /^[A-Za-z0-9 _\-!.]{1,64}$/;

export function name_ok(s) {
  return NAME_RE.test(s) && !s.includes("..") && s.trim() === s;
}

export function gen_missing() {
  const miss = [];
  if (!state.paths.game.trim()) miss.push("the extracted game folder");
  if (!state.paths.mod.trim()) miss.push("the mod folder");
  return miss.join(" and ");
}

export function mod_target() {
  const base = state.paths.mod.trim();
  if (!base || !state.info) return "";
  const sep = state.info.sep;
  const flat = ["hookshot", "longshot", "nx2navi"].includes(state.paths.host);
  const mid = flat ? "" : `${state.info.title_id}${sep}`;
  return `${base}${base.endsWith(sep) ? "" : sep}${mid}${state.paths.name}`;
}

const HOSTS = [
  { id: "eden", label: "Lichii / Eden" },
  { id: "hookshot", label: "Hookshot" },
  { id: "longshot", label: "Longshot" },
  { id: "nx2navi", label: "NX2navi" },
];
const SHUFFLE = (what) => [
  { id: "off", label: `Off, keep the original ${what}` },
  { id: "grouped", label: `Shuffle within each kind of ${what === "music" ? "track" : "sound"}` },
  { id: "anywhere", label: `Shuffle across all ${what === "music" ? "tracks" : "sounds"}` },
];
const COLORS = [
  { id: "off", label: "Off, keep the original colours" },
  { id: "random", label: "A random colour for each item" },
  { id: "shared", label: "One random hue turn for everything" },
];

const CONFIG = { id: "config", label: "Configuration", heading: "Configuration" };
const COSMETICS = { id: "cosmetics", label: "Cosmetics", heading: "Configuration" };

export const FIELDS = [
  { id: "host", page: CONFIG, section: "Where you play", label: "Game", choices: HOSTS,
    tip: "Lichii and Eden use their load folder. Hookshot, Longshot and NX2navi use the mods folder inside the game folder." },
  { id: "game", page: CONFIG, section: "Folders", label: "Extracted game folder", tip: "The folder with your extracted game files. It is only read, never changed." },
  { id: "mod", page: CONFIG, section: "Folders", label: "Mod folder", tip: "The folder your emulator or loader reads mods from. Generate writes the mod here." },
  { id: "name", page: CONFIG, section: "Folders", label: "Mod name", tip: "The name of the mod's own folder. Use 1 to 64 letters, digits, spaces and _ - ! . characters, with no spaces at the ends." },
  { id: "seed", page: CONFIG, section: "Seed", label: "Seed",
    tip: "Type a seed to make the same game again. Left empty, Generate picks a random seed. This box is never filled in for you. The seed that was used is in the spoiler log and on the Seed page." },
  { id: "spoiler_log", page: CONFIG, section: "Seed", label: "Include Spoiler Log",
    tip: "Writes Spoiler.txt into the mod folder next to the mod. It lists the seed, the settings string, every setting used, the expected playthrough step by step, and the item at every location. Off, no spoiler is made and an old Spoiler.txt is removed." },
  { id: "music", page: COSMETICS, section: "Music and sound", label: "Music shuffle", choices: SHUFFLE("music"),
    tip: "Music changes only the mod you build, never the seed, so the same seed can be built with different music. Cutscene music keeps its own tracks. Themes built from stacked layers trade whole layer sets. Area themes that change with the time of day play one track all day. Jingles only trade with other jingles." },
  { id: "sfx", page: COSMETICS, section: "Music and sound", label: "Sound effect shuffle", choices: SHUFFLE("sounds"), tip: "Shuffles sound effects in the mod you build. The seed is not changed." },
  { id: "colors", page: COSMETICS, section: "Colours", label: "Colour shuffle", choices: COLORS,
    tip: "Colours cover the three tunics, the swords, the shields, Navi, hair, the boomerang gem, the gauntlet jewels and recovery hearts. Particle glows, trails and the menu keep their own colours." },
  { id: "cosseed", page: COSMETICS, section: "Colours", label: "Cosmetic seed", tip: "Leave empty to use the seed text. Set it to get the same music and colours with a different seed." },
].map((f) => ({ ...f, page_label: f.page.label, heading: f.page.heading, page: f.page.id }));

function set_status(el, kind, title) {
  el.className = `stat ${kind}`;
  el.textContent = kind === "ok" ? "✓" : kind === "bad" ? "✗" : "";
  el.title = title;
  el.setAttribute("aria-label", title);
  el.hidden = kind === "none";
}

function row(f, terms, control, extra, badge) {
  const tip = tip_button(f.label, () => f.tip);
  const head = f.id === "spoiler_log"
    ? h("div", { class: "wl" }, h("label", { class: "chk", for: `f_${f.id}` }, control, h("span", null, marked(f.label, terms))), tip)
    : h("div", { class: "wl" }, h("label", { for: `f_${f.id}` }, marked(f.label, terms)), tip, badge || null);
  return h("div", { class: "w", "data-id": f.id }, head, f.id === "spoiler_log" ? null : control, extra);
}

function select(id, choices, value, onchange) {
  const sel = h("select", { id: `f_${id}` }, choices.map((c) => h("option", { value: c.id }, c.label)));
  sel.value = value;
  sel.addEventListener("change", () => onchange(sel.value));
  return sel;
}

export function field_el(f, terms = []) {
  const p = state.paths;
  const cos = state.cosmetic;
  if (f.id === "host") {
    return row(f, terms, select("host", HOSTS, p.host || "eden", (v) => {
      p.host = v;
      touch();
    }));
  }
  if (f.id === "game" || f.id === "mod") {
    const out = h("div", { class: "why err", "aria-live": "polite" });
    const stat = h("span", { class: "stat", role: "img", hidden: true });
    const inp = h("input", { type: "text", id: `f_${f.id}`, value: p[f.id], spellcheck: "false", autocomplete: "off",
      placeholder: f.id === "game" ? "Folder with the extracted game files" : "Folder the game loads mods from" });
    const check = async () => {
      const v = p[f.id].trim();
      out.textContent = "";
      if (!v) return set_status(stat, "none", "");
      set_status(stat, "wait", "Checking the folder");
      try {
        if (f.id === "game") {
          const r = await get(`/api/validate?path=${q(v)}`);
          if (v !== p.game.trim()) return;
          state.game_check = r;
          if (r.ok) set_status(stat, "ok", `Game folder found at ${r.romfs}`);
          else {
            set_status(stat, "bad", "This is not a usable game folder");
            out.textContent = r.problems.join(" ");
          }
        } else {
          await get(`/api/fs?path=${q(v)}`);
          if (v !== p.mod.trim()) return;
          state.mod_check = { ok: true };
          set_status(stat, "ok", "Folder found");
        }
      } catch (e) {
        if (f.id === "mod") state.mod_check = { ok: false };
        set_status(stat, "bad", e.message);
        out.textContent = e.message;
      }
    };
    const later = debounce(check, 350);
    inp.addEventListener("input", () => {
      p[f.id] = inp.value;
      touch();
      later();
    });
    const browse = h("button", { type: "button", class: "btn", onclick: async () => {
      browse.disabled = true;
      let v = null;
      try {
        v = await pick_folder(inp.value.trim(), f.id === "game" ? "Pick the game folder" : "Pick the mod folder");
      } finally {
        browse.disabled = false;
      }
      if (!v) return;
      inp.value = v;
      p[f.id] = v;
      touch();
      check();
    } }, "Browse");
    check();
    return row(f, terms, h("div", { class: "pathrow" }, inp, browse), out, stat);
  }
  if (f.id === "name") {
    const out = h("div", { class: "hint", "aria-live": "polite" });
    const inp = h("input", { type: "text", id: "f_name", value: p.name, spellcheck: "false", autocomplete: "off", maxlength: "64" });
    const show = () => {
      if (!name_ok(p.name)) return fill(out, h("span", { class: "mark bad" }, "Use 1 to 64 letters, digits, spaces and _ - ! . characters, with no spaces at the ends."));
      const t = mod_target();
      fill(out, t ? `The mod will go to ${t}` : "");
    };
    inp.addEventListener("input", () => {
      p.name = inp.value;
      touch();
      show();
    });
    show();
    return row(f, terms, inp, out);
  }
  if (f.id === "seed") {
    const inp = h("input", { type: "text", id: "f_seed", value: state.seed, maxlength: "100", spellcheck: "false", autocomplete: "off", placeholder: "Leave empty for a random seed" });
    inp.addEventListener("input", () => {
      state.seed = inp.value;
      touch();
    });
    return row(f, terms, inp);
  }
  if (f.id === "spoiler_log") {
    const box = h("input", { type: "checkbox", id: "f_spoiler_log", checked: state.spoiler_log });
    box.addEventListener("change", () => {
      state.spoiler_log = box.checked;
      touch();
    });
    return row(f, terms, box);
  }
  if (f.id === "music" || f.id === "sfx" || f.id === "colors") {
    return row(f, terms, select(f.id, f.choices, cos[f.id], (v) => {
      cos[f.id] = v;
      touch();
    }));
  }
  const inp = h("input", { type: "text", id: "f_cosseed", value: cos.seed, spellcheck: "false", autocomplete: "off", maxlength: "64", placeholder: "Leave empty to use the seed text" });
  inp.addEventListener("input", () => {
    cos.seed = inp.value;
    touch();
  });
  return row(f, terms, inp);
}

export function gen_line() {
  const line = h("p", { class: "genline", "aria-live": "polite" });
  const draw = () => {
    const miss = gen_missing();
    line.classList.toggle("ready", !miss);
    line.textContent = miss
      ? `Fill in ${miss}, then use Generate at the top right.`
      : `Generate, at the top right, writes the mod to ${mod_target()}${state.spoiler_log ? ", with Spoiler.txt beside it" : ""}.`;
  };
  draw();
  return { el: line, draw };
}

export function page_fields(page) {
  const secs = new Map();
  for (const f of FIELDS.filter((x) => x.page === page)) {
    if (!secs.has(f.section)) secs.set(f.section, []);
    secs.get(f.section).push(f);
  }
  return secs;
}
