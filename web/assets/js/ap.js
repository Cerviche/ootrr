import { h, fill, notice, copy } from "./dom.js";
import { get, post } from "./api.js";
import { state } from "./store.js";
import { gen_missing, name_ok, update_used } from "./config.js";
import { live, refresh_string } from "./strings.js";
import { go } from "./nav.js";

// The Archipelago page: read a slot's .apootrr file, then build its mod with the folders from
// the Configuration page. Kept in memory only: a patch is for one room and one slot.
const ap = { text: "", file: "", sum: null, name: "", phase: "idle", job: null, error: "", target: "" };
let redraw = () => {};

const SYMS = { done: ["ok", "✓"], failed: ["bad", "✗"], unavailable: ["wait", "–"], running: ["wait", "…"] };

export const ap_busy = () => ap.phase === "building";

function steps_view(job) {
  return h(
    "ul",
    { class: "steps-list" },
    job.steps.map((s) => {
      const [k, c] = SYMS[s.state] || SYMS.running;
      return h("li", null, h("span", { class: `mark ${k}`, "aria-hidden": "true" }, c), h("span", null, s.label));
    }),
  );
}

async function open_patch(pick, out) {
  const f = pick.files && pick.files[0];
  pick.value = "";
  if (!f) return;
  if (f.size > 64 * 1024 * 1024) return fill(out, notice("error", "That file is too large to be an Archipelago patch."));
  const text = await f.text();
  try {
    const sum = await post("/api/ap/read", { patch_text: text });
    Object.assign(ap, { text, file: f.name, sum, name: sum.mod_name, phase: "idle", job: null, error: "", target: "" });
  } catch (e) {
    return fill(out, notice("error", e.message));
  }
  redraw();
}

async function build() {
  if (ap_busy() || !ap.sum) return;
  const miss = gen_missing();
  if (miss) {
    Object.assign(ap, { phase: "failed", error: `Fill in ${miss} on the Configuration page first.` });
    return redraw();
  }
  if (!name_ok(ap.name)) {
    Object.assign(ap, { phase: "failed", error: "Mod name: use 1 to 64 letters, digits, spaces and _ - ! . characters, with no spaces at the ends." });
    return redraw();
  }
  const p = state.paths;
  Object.assign(ap, { phase: "building", job: null, error: "", target: "" });
  redraw();
  try {
    const r = await post("/api/ap/build", {
      patch_text: ap.text,
      game_dir: p.game.trim(),
      update_dir: p.update.trim(),
      include_update: update_used() && state.include_update,
      mod_dir: p.mod.trim(),
      mod_name: ap.name,
      host: p.host || "eden",
      title_logo: state.cosmetic.title !== false,
      delete_others: !!state.delete_others,
    });
    ap.target = r.target || "";
    for (let i = 0; i < 900; i++) {
      const job = await get(`/api/jobs/${r.job_id}`);
      ap.job = job;
      redraw();
      if (job.finished) {
        if (job.state !== "done") throw new Error(job.message || "The build failed.");
        ap.phase = "done";
        return redraw();
      }
      await new Promise((ok) => setTimeout(ok, 400));
    }
    throw new Error("The build is taking too long. Check the server window.");
  } catch (e) {
    Object.assign(ap, { phase: "failed", error: e.message });
    redraw();
  }
}

function patch_section() {
  const out = h("div", { "aria-live": "polite" });
  const pick = h("input", { type: "file", accept: ".apootrr,application/json", class: "hide" });
  pick.onchange = () => open_patch(pick, out);
  const open_btn = h("button", { type: "button", class: `btn${ap.sum ? "" : " primary"}`, disabled: ap_busy(), onclick: () => pick.click() }, ap.sum ? "Open another patch" : "Open a .apootrr file");
  const s = ap.sum;
  return h(
    "div",
    { class: "msec" },
    h("h4", null, "Patch file"),
    s
      ? h(
          "dl",
          { class: "kv" },
          h("dt", null, "File"),
          h("dd", { class: "mono" }, ap.file),
          h("dt", null, "Slot"),
          h("dd", null, s.slot),
          h("dt", null, "Archipelago seed"),
          h("dd", { class: "mono" }, s.seed_name),
          h("dt", null, "Preset"),
          h("dd", null, s.preset),
          h("dt", null, "Shuffled locations"),
          h("dd", null, String(s.locations)),
          h("dt", null, "Items for other players"),
          h("dd", null, String(s.foreign)),
          h("dt", null, "Items you receive"),
          h("dd", null, String(s.receive)),
        )
      : h("p", { class: "muted" }, "The room's host gives each OOTRR player a .apootrr file after generating the multiworld. Open yours here."),
    h("div", { class: "row" }, open_btn, pick),
    out,
  );
}

function build_section() {
  const p = state.paths;
  const miss = gen_missing();
  const name = h("input", { type: "text", id: "ap_name", value: ap.name, spellcheck: "false", disabled: ap_busy() });
  name.addEventListener("input", () => (ap.name = name.value));
  const btn = h("button", { type: "button", class: "btn primary", disabled: ap_busy() || Boolean(miss), onclick: build }, "Build the mod");
  const job = ap.job;
  const done = ap.phase === "done";
  return h(
    "div",
    { class: "msec" },
    h("h4", null, "Build"),
    miss
      ? notice("warn", `Fill in ${miss} on the `, h("a", { href: "#/config", onclick: (e) => (e.preventDefault(), go("config")) }, "Configuration"), " page first. The Archipelago build uses the same game, game folder and mod folder.")
      : h(
          "dl",
          { class: "kv" },
          h("dt", null, "Game folder"),
          h("dd", { class: "mono" }, p.game),
          h("dt", null, "Mod folder"),
          h("dd", { class: "mono" }, p.mod),
          h("dt", null, "Plays on"),
          h("dd", null, p.host),
        ),
    h("label", { class: "mfield", for: "ap_name" }, "Mod name", name),
    h("div", { class: "row" }, btn),
    job && !job.finished ? h("progress", { max: "100", value: String(job.percent), "aria-label": "Build progress" }) : null,
    ap.phase === "building" && !job ? h("progress", { "aria-label": "Build progress" }) : null,
    job ? steps_view(job) : null,
    done && ap.target ? h("p", { class: "genline ready" }, `Mod written to ${ap.target}.`) : null,
    ap.phase === "failed" && ap.error ? notice("error", ap.error) : null,
    done && job && job.message ? h("details", { class: "fold" }, h("summary", null, "Build report"), h("pre", { class: "report" }, job.message)) : null,
  );
}

function settings_section() {
  const str = h("input", { type: "text", readonly: true, value: live.text || "", "aria-label": "Settings string", spellcheck: "false", class: "mono" });
  if (!live.text) refresh_string().then(() => (str.value = live.text || live.error || ""));
  return h(
    "div",
    { class: "msec" },
    h("h4", null, "Your settings in Archipelago"),
    h("p", null, "To play with the settings you picked in this app instead of a preset, put this string in your player file as ", h("code", null, "settings_string"), " under ", h("code", null, "OOTRR:"), ". The computer that generates the room then needs this app too."),
    h("div", { class: "stringbox" }, str, h("button", { type: "button", class: "btn", onclick: () => live.text && copy(live.text) }, "Copy")),
  );
}

function howto() {
  return h(
    "div",
    { class: "msec" },
    h("h4", null, "Playing in a multiworld"),
    h(
      "ol",
      null,
      h("li", null, "Put ootrr.apworld in Archipelago's custom_worlds folder (Archipelago 0.6.7 or newer). It must be the same version as this app."),
      h("li", null, "Write your player file with game: OOTRR and a preset (default, easy, keysanity, full_shuffle or open_world), or your own settings string."),
      h("li", null, "After the host generates the room, open your .apootrr file above and build the mod."),
      h("li", null, "Install the mod and start a new save. The mod marks the save with your slot's seed."),
      h("li", null, "Open the Archipelago Launcher, start OOTRR Client and connect with your slot name. The first time, it asks where ootrr.exe is."),
    ),
    notice("warn", "Archipelago support is experimental. Entrance shuffle is refused, and dungeon items stay in their own world."),
  );
}

export function ap_page(show) {
  redraw = show;
  return [patch_section(), build_section(), settings_section(), howto()];
}
