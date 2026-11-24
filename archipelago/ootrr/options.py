from dataclasses import dataclass

from Options import Choice, PerGameCommonOptions

from .tables import presets


def _preset_choice() -> type:
    ps = presets()
    attrs = {"__doc__": "The OOTRR settings preset this world is built from. Each one is a full set of OOTRR settings:\n\n" + "\n".join(f"{p['name']}: {p['tip']}" for p in ps), "display_name": "Preset"}
    for k, p in enumerate(ps):
        attrs[f"option_{p['id']}"] = k
    attrs["default"] = next((k for k, p in enumerate(ps) if p["id"] == "default"), 0)
    return type("Preset", (Choice,), attrs)


Preset = _preset_choice()


@dataclass
class OOTRROptions(PerGameCommonOptions):
    preset: Preset
