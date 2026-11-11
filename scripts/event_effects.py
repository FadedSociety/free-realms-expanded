"""B1: the use effects of the event, code and free-item consumables, as consumables.json rows.

Run with ``python -I``. Standard library only; an offline content tool the server never runs.

  event_effects.py --apply   write every row below whose item exists into consumables.json
  event_effects.py --check   fail when a shipped row drifts from this table, or when an audited
                             event consumable has neither a row nor a reason in LEFT

Each row is keyed by item id, or by the item's English name for an item that has no client row
yet (B3 adds those: ``lab/autopilot/opus/audits/events/B3-new-items.json`` and the release item
table). A re-run after B3's merge picks those up. The spec of every row is the item's own text
(description, wiki tooltip or the client's 'Illusion: X - N minutes' ability strings); a value the
evidence does not give is marked (R) in the row's source.

Every input has a default inside the repository or the lab; each can be passed explicitly.
"""

import argparse
import json
import re
import sys
from pathlib import Path

REPO = Path(__file__).resolve().parent.parent
LAB = REPO.parent / "lab"
SANCTUARY_REV = "10f129e"
LEGACY_REV = "9e08202"
CHANGEFORM_ICON = 3843  # icon_status_changeform_32.dds, the consumables.json buff convention
POOF = 21  # PFX_smoke_black_explosion
MIN = 60_000
SINGLE_USE_FLAG = 1 << 4  # item-table "flags" order: NoTrade NoSale MembersOnly NonMiniGame SingleUse


def transform(model, ms):
    return {"transform": {"model_id": model, "poof_effect_id": POOF, "duration_ms": ms, "icon_id": CHANGEFORM_ICON}}


def aura(effect, ms):
    return {"aura": {"composite_effect_id": effect, "duration_ms": ms, "delay_ms": 0, "icon_id": CHANGEFORM_ICON}}


def scale(value, ms):
    return {"scale": {"scale": value, "duration_ms": ms, "icon_id": CHANGEFORM_ICON}}


def burst(effect, ms, at_target=False):
    body = {"composite_effect_id": effect, "duration_ms": ms}
    if at_target:
        body["at_target"] = True
    return {"burst": body}


def stat(stat_id, value, ms):
    # icon_id is filled with the item's own icon (R, as the L087 boosters).
    return {"stat": {"stat_id": stat_id, "value": value, "duration_ms": ms, "icon_id": 0}}


def heal(percent):
    return {"heal": {"percent": percent}}


def boost(raises, percent, ms):
    # icon_id is filled with the item's own icon, as the L087 boosters.
    return {"boost": {"raises": raises, "percent": percent, "duration_ms": ms, "icon_id": 0}}


FIREWORK = 10_000  # (R) BubbaJoeX fork FIREWORK_COOLDOWN_MS and LOOP_CAST_DURATION_MS
SPRAY = 5_000  # the p2p silly-string beams' defaultLifeTime (ActorCompositeEffectDefinitions)
FORK = "BubbaJoeX fork ItemAbilities.json"

# (key, event, effect, cooldown_ms or None for the duration, evidence). The key is an item id or
# an English item name.
ROWS = [
    # Halloween (Super Spooktacular)
    (10433, "halloween", transform(3316, 5 * MIN), None,
     "fanbyte Vampire_Cake_Slice 'Illusion: Black Bat - 5 minutes' (client locale 6630); model 3316 bat_black.adr"),
    (16537, "halloween", transform(177, 5 * MIN), None,
     "fanbyte Werewolf_Cake_Slice 'Illusion: Black Wolf - 5 minutes' (client locale 7338); model 177 wolf_evil.adr, the client's dark wolf (R: no model is named black wolf)"),
    (69822, "halloween", burst(15905, SPRAY, True), FIREWORK,
     "item text 'spray spooky string all over your favorite things'; effect 15905 PFX_silly-string_beam_black_p2p (" + FORK + " PartyFavors); cooldown (R)"),
    (69823, "halloween", burst(15827, SPRAY, True), FIREWORK,
     "item text 'spray spooky string all over your favorite things'; effect 15827 PFX_silly-string_beam_orange_p2p (" + FORK + " PartyFavors); cooldown (R)"),
    (69839, "halloween", transform(3305, 30 * MIN), None,
     "item text 'sure to drive you batty', client spell name 'halloween_consumable_bat_cupcake' (audit: the old row borrowed Sanctuary's Cakenstein ability 4373); model 3305 halloweenbat_01.adr; 30 minutes (R, its sibling cupcakes)"),
    (69840, "halloween", aura(15909, 30 * MIN), None,
     "item text 'a haunting good time'; effect 15909 PFX_halloween_ghosts_up_loop (" + FORK + " ability 4374); 30 minutes (R, the fork's food default)"),
    (69841, "halloween", transform(198, 30 * MIN), None,
     "item text 'leave you wanting Nevermore'; model 198 birds_raven.adr (" + FORK + " ability 4375); 30 minutes (R)"),
    (69821, "halloween", burst(15914, 3_000, True), FIREWORK,
     "fanbyte Mini_Jack-O-Lantern 'a very brief orange streamer effect on the person you toss it at'; effect 15914 PFX_halloween_confetti_orange-black_loop (R), 3 s (R)"),
    (69818, "halloween", transform(1689, 30 * MIN), None,
     "item text 'Turns the user into a Jack-o-Lantern for 30 minutes' (client locale 147631); model 1689 jackolantern_m_01.adr"),
    (69845, "halloween", transform(1692, 3 * MIN), None,
     "item text 'Turns the user into a skeleton for 3 minutes' (client locale 147632); model 1692 skeleton_m_01.adr"),
    (69846, "halloween", transform(751, 3 * MIN), None,
     "item text 'Turns the user into a wraith for 3 minutes' (client locale 147633); model 751 wraith_m_cloakedskelly.adr"),
    (69847, "halloween", transform(1339, 3 * MIN), None,
     "item text 'Turns the user into a tarantula for 3 minutes' (client locale 147634); model 1339 tarantula_01.adr"),
    (69848, "halloween", transform(52, 3 * MIN), None,
     "item text 'Turns the user into a black widow for 3 minutes' (client locale 147635); model 52 blackwidow_01.adr"),
    (76710, "halloween", boost("coins", 50, 60 * MIN), 5_000,
     "item text 'boosts the amount of coin you earn for an hour'; +50% (R, " + FORK + " ability 251 GoldBoostPercent 50), the L087 booster recast"),
    (77420, "halloween", boost("coins", 50, 60 * MIN), 5_000,
     "item text 'boosts the amount of coin you earn for an hour'; +50% (R, " + FORK + " ability 251 GoldBoostPercent 50), the L087 booster recast"),
    # Snow Days and the 12 Days
    (76620, "snow_days", aura(16418, 2 * MIN), None,
     "fanbyte Bell_Cut-Out_Cookie 'Showers you with jingling holiday cheer for two minutes'; effect 16418 PFX_bells_gold_falling_head_medium_loop"),
    (76621, "snow_days", aura(16411, 2 * MIN), None,
     "fanbyte Candy_Cane 'Displays animated peppermints around your hands for two minutes'; effect 16411 PFX_beam_red-white_feet_candycane (" + FORK + " ability 5038)"),
    (76622, "snow_days", scale(0.5, 30_000), None,
     "fanbyte Gingerbread_Cookie 'Makes you small and fast like the gingerbread man for thirty seconds'; scale 0.5 (R, the Cupcake of Tim the Tiny); the speed half is not applied (R: stat 2 belongs to the ride)"),
    (76624, "snow_days", transform(535, 10 * MIN), None,
     "fanbyte Eggnog 'turn you into a chicken for a few minutes' (client locale 147636 'Illusion: Chicken - 10 minutes'); model 535 chicken.adr; the second-drink barf is not wired"),
    (76625, "snow_days", transform(1943, 2 * MIN), None,
     "fanbyte Ornament_Cut-Out_Cookie 'a big, bouncing, rolling Christmas tree ornament for two minutes' (client locale 147637); model 1943 giantornament.adr"),
    (76626, "snow_days", transform(1932, 2 * MIN), None,
     "fanbyte Tree_Cut-Out_Cookie 'Turns you into a Christmas Floren for two minutes' (client locale 147638); model 1932 floren_christmastree.adr"),
    (76627, "snow_days", transform(1964, 2 * MIN), None,
     "fanbyte Fruit_Cake 'turn into a fruit cake for a couple minutes' (client locale 147639 'Illusion: Fruitcake - 2 minutes'); model 1964 fruitcake_m_basic.adr"),
    (76634, "snow_days", transform(1942, 2 * MIN), None,
     "fanbyte Penguin_Cut-Out_Cookie 'Turns you into a snowballed penguin for two minutes' (client locale 147640 'Illusion: Giant Snowball - 2 minutes'); model 1942 giantsnowball.adr (R)"),
    (76609, "snow_days", burst(16375, FIREWORK), FIREWORK,
     "item text 'Make your holiday season a blast'; effect 16375 PFX_party-popper_multi_loop_l-hand_xmas (" + FORK + " ability 5033)"),
    (76610, "snow_days", burst(16381, FIREWORK), FIREWORK,
     "item text 'Deck the sky with garland'; effect 16381 EFX_fireworks_green-gold_christmas-tree (" + FORK + " ability 5034)"),
    (76611, "snow_days", burst(16380, FIREWORK), FIREWORK,
     "item text 'Paint the sky with candy stripes'; effect 16380 EFX_fireworks_red-white_candy-cane (" + FORK + " ability 5035)"),
    (76612, "snow_days", burst(16413, FIREWORK), FIREWORK,
     "item text 'Every effect is actually unique'; effect 16413 EFX_fireworks_white_snowflakes (" + FORK + " ability 5036)"),
    ("Snow Flake Firework", "twelve_days", burst(16413, FIREWORK), FIREWORK,
     "fanbyte Snow_Flake_Firework (the 2012 singular row): the 2011 Snowflake Fireworks effect 16413 (R)"),
    ("Candy Cane Firework", "twelve_days", burst(16380, FIREWORK), FIREWORK,
     "fanbyte Candy_Cane_Firework (the 2012 singular row): the 2011 Candy Cane Fireworks effect 16380 (R)"),
    ("Garland Firework", "twelve_days", burst(16381, FIREWORK), FIREWORK,
     "fanbyte Garland_Firework (the 2012 singular row): the 2011 Garland Fireworks effect 16381 (R)"),
    ("Snowman Mask", "twelve_days", transform(140, 60 * MIN), None,
     "fanbyte Snowman_Mask 'Turns you into a snowman. Use it as many times as you want!' 'Illusion: Snowman - 1 hour' (client locale 441237); model 140 snowman.adr"),
    ("Holigoblin Mask", "twelve_days", transform(1928, 60 * MIN), None,
     "audit: 'Illusion: Holigoblin 1 hour' (client locale 5102962); model 1928 robgoblin_m_xmas.adr (R)"),
    ("Snowball", "snow_days", burst(5236, 3_000, True), FIREWORK,
     "fanbyte Snowball: single use, thrown (the Snowball Creation Unit's); effect 5236 PFX_snowball_white_cog_land on the aimed player, 3 s (R)"),
    # Birthday Bash
    (77435, "birthday_bash", burst(16517, FIREWORK), FIREWORK,
     "item text 'Shoot off some orange and blue'; effect 16517 PFX_fireworks_blue-orange_birthday (" + FORK + " ability 345)"),
    (77436, "birthday_bash", burst(16519, FIREWORK), FIREWORK,
     "fanbyte Anniversary_Firework 'Short duration fireworks'; effect 16519 PFX_fireworks_blue-orange_birthday-giant-candle (" + FORK + " ability 346)"),
    (77442, "birthday_bash", burst(16518, FIREWORK), FIREWORK,
     "item text 'It really says, Free Realms'; effect 16518 PFX_fireworks_blue-orange_birthday_with_logo (" + FORK + " ability 347)"),
    (77438, "birthday_bash", burst(16504, FIREWORK), FIREWORK,
     "fanbyte Birthday_Boomer 'Short duration streamers and confetti'; effect 16504 WFX_streamer-confetti_blue-orange_falling_skel_loop (R)"),
    (77439, "birthday_bash", burst(15825, SPRAY, True), FIREWORK,
     "item text 'Spray your friends with this blue birthday spray'; effect 15825 PFX_silly-string_beam_blue_p2p (" + FORK + " PartyFavors)"),
    (77440, "birthday_bash", burst(15827, SPRAY, True), FIREWORK,
     "item text 'Spray your friends with this orange anniversary spray'; effect 15827 PFX_silly-string_beam_orange_p2p (" + FORK + " PartyFavors)"),
    (77437, "birthday_bash", heal(20), 5_000,
     "a single-use treat; 20% health and 5 s recast (R, " + FORK + " ability 348)"),
    (77443, "birthday_bash", heal(20), 5_000,
     "a single-use treat; 20% health and 5 s recast (R, " + FORK + " ability 349)"),
    # Festival of Hearts
    (76800, "festival_of_hearts", aura(15555, MIN), 120_000,
     "item text 'This magical potion fills back up over time' (not single use); effect 15555 PFX_hearts_multi_out_sph_head_loop, 1 minute and a 2 minute refill (R, " + FORK + " ability 5150)"),
    (76801, "festival_of_hearts", aura(16462, 2 * MIN), None,
     "fanbyte Candy_Hearts 'Places an aura of hearts around yourself for 2 minutes'; effect 16462 PFX_candy-hearts_red_cog_valentines_day_loop"),
    (76802, "festival_of_hearts", aura(16465, 2 * MIN), None,
     "item text 'a sucker for love'; effect 16465 PFX_candy-hearts_rounded_red_cog_valentines_day_loop (" + FORK + " ability 5152); 2 minutes (R, as Candy Hearts)"),
    (76803, "festival_of_hearts", transform(2042, 2 * MIN), None,
     "item text 'winged Cupid cookie'; client locale 147644 'Illusion: Cupid - 2 minutes'; model 2042 fairy_m_cupid.adr (R: the ability tooltip is not bound to an item)"),
    (76804, "festival_of_hearts", burst(16461, FIREWORK), FIREWORK,
     "item text 'Love is explosive'; effect 16461 PFX_party-popper_red_loop_l-hand_valentines (" + FORK + " ability 5154)"),
    (76805, "festival_of_hearts", burst(5475, SPRAY, True), FIREWORK,
     "item text 'Ready... Aim... Love!'; effect 5475 PFX_hearts_red_p2p at the aimed player (R)"),
    (76806, "festival_of_hearts", transform(2056, 2 * MIN), None,
     "fanbyte Stone_Heart_s_Candy 'Turns you into a Stone Heart Changeling for 2 minutes' (client locale 147645); model 2056 changeling_emo_m.adr (item text 'Emolicious')"),
    (2466, "festival_of_hearts", aura(16460, 30_000), None,
     "effect 16460 PFX_rose-petals_red_falling_med_sph_loop, 30 s (R, " + FORK + " ability 5157)"),
    (2467, "festival_of_hearts", aura(16487, 30_000), None,
     "effect 16487 PFX_rose-petals_dark-purple_falling_med_cylinder, 30 s (R, " + FORK + " ability 5158)"),
    (77331, "festival_of_hearts", burst(15456, SPRAY, True), FIREWORK,
     "item text 'Toss your friends a little bit of love'; effect 15456 PFX_petals-sparkles_up on the aimed player (R)"),
    # Summer Camp
    (9116, "summer_camp", transform(4, 30 * MIN), None,
     "fanbyte Robgoblin_Goody 'Illusion: Robgoblin - 30 minutes' (client locale 6838); model 4 robgoblin_m_basic.adr"),
    (9117, "summer_camp", transform(376, 30 * MIN), None,
     "fanbyte Treeble_Treat 'Illusion: Treeble - 30 minutes' (client locale 6837); model 376 treeble.adr"),
    (15896, "summer_camp", transform(2194, 2 * MIN), None,
     "fanbyte Tasty_Marshmallow 'Illusion: Marshmallow - 2 minutes' (client locale 5048); model 2194 marshmallow_critter.adr"),
    (9295, "summer_camp", transform(418, 5 * MIN), None,
     "fanbyte Bucket_of_Bouncing_Bait 'transforms you into a fish' (unlimited use); client locale 17043 'Illusion: Flopping Fish - 5 minutes'; model 418 fish.adr"),
    # Spring Eggstravaganza
    ("Handful of Jelly Beans", "spring_eggstravaganza", transform(403, 3 * MIN), None,
     "fanbyte Handful_of_Jelly_Beans 'Eating these will turn you into a rabbit for 3 minutes'; model 403 rabbit.adr"),
    # Codes and free items
    (2959, "codes", transform(141, 30 * MIN), None,
     "item text 'change into a frog'; model 141 toad.adr (" + FORK + " ability 1978); 30 minutes (R)"),
    (3005, "codes", transform(135, 30 * MIN), None,
     "item text 'change the player into a Goat'; model 135 billygoat.adr (" + FORK + " ability 2030); 30 minutes (R)"),
    (4059, "codes", transform(412, 30 * MIN), None,
     "item text 'change the player into a Shaggy Spider' (client locale 147597 'Illusion: Shaggy Spider - 30 minutes'); model 412 shaggyspider_01.adr (audit: the Sanctuary row drew a black widow)"),
    (4067, "codes", transform(160, 30 * MIN), None,
     "item text 'change the player into a Fungal Man-Eating Plant' (client locale 147605 'Illusion: Man-Eating Fungus - 30 minutes'); model 160 maneatingfungus.adr (audit: the Sanctuary row drew a vine-golem wolf)"),
    (60277, "codes", transform(169, 30 * MIN), None,
     "item text 'a large floppy eared dog for thirty minutes'; model 169 dog_large_floppy.adr"),
    (2957, "codes", transform(434, 30 * MIN), None,
     "item text 'change the player into an Animated Plant'; model 434 maneatingplant_greenpurple.adr (" + FORK + " ability 1980); 30 minutes (R)"),
    (79, "codes", scale(0.65, 30 * MIN), None,
     "item text 'Eating this food will make you smaller!'; scale 0.65 and 30 minutes (R, " + FORK + " ability 1943 'Shrink Food III')"),
    (2979, "codes", aura(12, 30 * MIN), None,
     "item text 'cause the player's feet to be on fire'; effect 12 PFX_fire_orange_feet_sprint_loop (" + FORK + " ability 1959); 30 minutes (R)"),
    (60272, "codes", aura(15550, 30 * MIN), None,
     "item text 'pixie dust to appear around the player for thirty minutes'; effect 15550 PFX_pixiedust_gold_loop_feet"),
    (60273, "codes", aura(53, 30 * MIN), None,
     "item text 'snow flakes to appear around the player for thirty minutes'; effect 53 PFX_snowflakes_white_sphere_loop"),
    (60269, "codes", stat(27, 2, 30 * MIN), None,
     "item text 'Buffs your timing': stat 27 Timing (AbilityEffectTypeSettings 24 '+ N Timing'); +2 and 30 minutes (R, " + FORK + " ability 4089 M3Timer)"),
    (60267, "codes", stat(27, 2, 30 * MIN), None,
     "item text 'Buffs your timing': stat 27 Timing (AbilityEffectTypeSettings 24 '+ N Timing'); +2 and 30 minutes (R, " + FORK + " ability 4087 M3Timer)"),
    (3084, "codes", heal(50), 30_000,
     "item text 'restores some of your health and energy instantly'; 50% health and a 30 s recast (R, " + FORK + " ability 4700); the energy half is not applied"),
    (60563, "codes", heal(50), 30_000,
     "item text 'restores some of your health and energy instantly'; 50% health and a 30 s recast (R, " + FORK + " ability 4700); the energy half is not applied"),
]

# Placed props: the item puts a clickable prop down (placed_cakes.json, T557) and each click gives
# the clicker one outcome. (key, event, prop model, [(outcome effect, ms)], evidence)
PLACED = [
    (77330, "festival_of_hearts", 2061, [({"transform": {"model_id": 2033, "poof_effect_id": POOF}}, 10 * MIN)],
     "fanbyte Turkleton_Treats 'you and your friends will turn into a chocolate bear', 'When clicked on, you will turn into a small chocolate bear for 10 minutes'; prop 2061 evnt_valentines_treat_basket_01.adr; bear 2033 teddybear_m_basic.adr (R)"),
    (77459, "birthday_bash", 407, [({"transform": {"model_id": 152, "poof_effect_id": POOF}}, 30 * MIN)],
     "item text 'It's made out of real cakelings...'; prop 407 mkt_cakes_glerga_01.adr; outcome cakeling 152 cakeling_m_basic.adr and 30 minutes (R, as the regional birthday cakes)"),
    ("Toxic Barrel", "halloween", 4592, [({"aura": {"composite_effect_id": 15974}}, 30 * MIN)],
     "fandom Toxic Barrel 'Place the barrel and then click it to acquire some toxic effects!', 'a green fire effect appears around the player's head'; prop 4592 mkt_toxic_barrel_01.adr; effect 15974 PFX_fire_green_head_pumpkin-prince_loop (R) for 30 minutes (R)"),
]

# Audited elements with no row, and why. Keyed like ROWS.
LEFT = {
    69829: "Ectoplasmic Goo is not usable: its only use is trading 15 for the Goo'd Candy minigame (fanbyte).",
    76556: "Oversized Candy Cane is a weapon (category 107), not a consumable.",
    76642: "Fire-Works-Place is placed and hands out fireworks (fanbyte): needs an item-grant outcome on placed props.",
    76643: "Snowball Creation Unit is placed and hands out Snowballs, which have no item row (B3); needs an item-grant outcome.",
    4829: "Birthday Party Crate is placed and hands out balloons: needs an item-grant outcome on placed props.",
    4828: "Balloon Pet Pumper gives a balloon pet: needs a trial-pet outcome (pets).",
    8157: "Balloon Pet Pumper gives a balloon pet: needs a trial-pet outcome (pets).",
    9114: "Ghost Tails Campfire is placed and gives a trial ghost pet (fanbyte): needs a trial-pet outcome (pets).",
    "Crispy Croissant": "No client item or locale name is Crispy Croissant; GC must name its item id.",
}


def load_json(path):
    return json.loads(Path(path).read_text(encoding="utf-8-sig"))


def item_table(directory):
    """id -> (NameId, DescriptionId, IconId, ActivatableAbilityId, SingleUse), table then supplement."""
    items = {}
    for name in ("item-table.json", "item-table-supplement.json"):
        path = Path(directory) / name
        if not path.exists():
            continue
        table = load_json(path)
        column = {name: index for index, name in enumerate(table["columns"])}
        for row in table["rows"]:
            values = row[1]
            items.setdefault(row[0], (
                values[column["NameId"]],
                values[column["DescriptionId"]],
                values[column["IconData.Id"]],
                values[column["ActivatableAbilityId"]],
                bool(values[column["Flags"]] & SINGLE_USE_FLAG),
            ))
    return items


def names(locale_path):
    if not locale_path or not Path(locale_path).exists():
        return {}
    return {row["id"]: row["text"] for row in load_json(locale_path)}


def resolve(key, items, locale, b3):
    """The item id behind a ROWS/LEFT key, or None while it has no item row."""
    if isinstance(key, int):
        return key if key in items else None
    if key in b3:
        return b3[key] if b3[key] in items else None
    wanted = key.lower()
    hits = sorted(i for i, row in items.items() if locale.get(row[0], "").lower() == wanted)
    return hits[0] if hits else None


def b3_items(path):
    """B3's {name: item id} list, when it has landed (any shape with name and id fields)."""
    if not path or not Path(path).exists():
        return {}
    found = {}

    def walk(node):
        if isinstance(node, dict):
            name = node.get("name") or node.get("element")
            ident = node.get("item_id", node.get("id"))
            if isinstance(name, str) and isinstance(ident, int):
                found[name] = ident
            for value in node.values():
                walk(value)
        elif isinstance(node, list):
            for value in node:
                walk(value)

    walk(load_json(path))
    return found


def build_row(item_id, event, effect, cooldown, evidence, items, legacy, sanctuary):
    name_id, description_id, icon_id, ability, single_use = items[item_id]
    effect = json.loads(json.dumps(effect))
    body = next(iter(effect.values()))
    if "stat" in effect or "boost" in effect:
        body["icon_id"] = icon_id
    duration = body.get("duration_ms", 0)
    cooldown = cooldown if cooldown is not None else max(duration, 5_000)
    other = legacy.get(item_id)
    if other is None:
        cross = f"Legacy@{LEGACY_REV} ClientItemDefinitions has no Id {item_id}"
    else:
        same = all(other.get(k) == v for k, v in (
            ("NameId", name_id), ("DescriptionId", description_id), ("ActivatableAbilityId", ability)))
        cross = f"Legacy@{LEGACY_REV} ClientItemDefinitions {'agrees' if same else 'differs'}"
    source = (
        f"B1 {event}: {evidence}. Client item table: Id {item_id} NameId {name_id} "
        f"DescriptionId {description_id} ActivatableAbilityId {ability} SingleUse "
        f"{'true' if single_use else 'false'}; Sanctuary-main@{SANCTUARY_REV} Consumables.json "
        f"{sanctuary_note(sanctuary, item_id, ability)}; {cross}. Recast {cooldown} ms"
        f"{'' if cooldown != duration else ', the effect length'} (R)."
    )
    return {
        "definition": item_id,
        "name_id": name_id,
        "description_id": description_id,
        "cooldown_ms": cooldown,
        "single_use": single_use,
        "effect": effect,
        "source": source,
    }


def sanctuary_rows(path):
    """Ability and item ids in Sanctuary's Consumables.json -> its comment, when the file is there."""
    if not path or not Path(path).exists():
        return {}
    found = {}
    for section, rows in load_json(path).items():
        for row in rows:
            for field in ("AbilityId", "ItemId"):
                if field in row:
                    found[(field, row[field])] = f"{section} {field} {row[field]} ({row.get('Comment', '')})"
    return found


def sanctuary_note(sanctuary, item_id, ability):
    hit = sanctuary.get(("ItemId", item_id)) or sanctuary.get(("AbilityId", ability))
    return f"lists {hit}; the item's own text wins" if hit else "has no row for it"


def wanted_rows(args):
    items = item_table(args.items)
    locale = names(args.locale)
    b3 = b3_items(args.b3)
    legacy_path = Path(args.legacy) if args.legacy else None
    legacy = {}
    if legacy_path and legacy_path.exists():
        legacy = {row["Id"]: row for row in load_json(legacy_path)}
    sanctuary = sanctuary_rows(args.sanctuary)
    rows, pending = {}, []
    for key, event, effect, cooldown, evidence in ROWS:
        item_id = resolve(key, items, locale, b3)
        if item_id is None:
            pending.append(key)
            continue
        if item_id in rows:
            raise SystemExit(f"B1: item {item_id} has two rows")
        rows[item_id] = build_row(item_id, event, effect, cooldown, evidence, items, legacy, sanctuary)
    props = {}
    for key, event, model, outcomes, evidence in PLACED:
        item_id = resolve(key, items, locale, b3)
        if item_id is None:
            pending.append(key)
            continue
        props[item_id], rows[item_id] = build_prop(
            item_id, event, model, outcomes, evidence, items, legacy, sanctuary)
    return rows, props, pending, items, locale, b3


def build_prop(item_id, event, model, outcomes, evidence, items, legacy, sanctuary):
    """The placed_cakes.json row and its consumables.json placed_cake row. Lifetime, click
    window, range, offset and recast are the regional birthday cakes' (Sanctuary defaults, R)."""
    row = build_row(item_id, event, {"placed_cake": {"row": item_id}}, 60_000, evidence, items, legacy, sanctuary)
    row["single_use"] = False
    prop = {
        "item_id": item_id,
        "name_id": row["name_id"],
        "model_id": model,
        "cursor_id": 5,
        "animation_id": 1,
        "spawn_effect_ids": [POOF],
        "lifetime_ms": 60_000,
        "item_cooldown_ms": 60_000,
        "click_window_ms": 5_000,
        "click_range_m": 5.0,
        "spawn_offset_m": 1.5,
        "icon_id": CHANGEFORM_ICON,
        "outcomes": [{"effect": effect, "duration_ms": ms} for effect, ms in outcomes],
        "source": f"B1 {event}: {evidence}; cursor 5 cursor_talk, spawn poof 21, the 60 s lifetime, "
                  f"5 s click window and 60 s recast are the regional birthday cakes' (R)",
    }
    return prop, row


def audited_ids(audits):
    """Every item id (or name) the event audits list as a consumable without a working use."""
    want = set()
    for path in sorted(Path(audits).glob("*.json")):
        if not re.fullmatch(r"[a-z_]+\.json", path.name):  # the event audits, not B1-/B3- files
            continue
        data = load_json(path)
        if not isinstance(data, dict):
            continue
        for row in data.get("elements") or data.get("rows") or []:
            if not isinstance(row, dict):
                continue
            text = " ".join(str(row.get(k, "")) for k in ("note", "ours", "field")).lower()
            if row.get("kind") != "consumable" and not re.search(
                r"use effect|does nothing|no consumables\.json row|no consumable row", text
            ):
                continue
            if row.get("status") in ("ok", "n/a"):
                continue
            raw = row.get("ids")
            if isinstance(raw, dict):  # only its "item"/"items" are item ids ("name" is text)
                picked = json.dumps([raw[k] for k in ("item", "items") if k in raw])
                ids = [int(n) for n in re.findall(r"\d{2,6}", picked)]
            else:
                ids = [int(n) for n in re.findall(r"item\D{0,4}(\d{2,6})", str(raw))]
                ids = ids or [int(n) for n in re.findall(r"\b(\d{2,6})\b", str(raw))]
            want.update(ids or [row.get("element", "")])
    return want


def main():
    parser = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    mode = parser.add_mutually_exclusive_group(required=True)
    mode.add_argument("--apply", action="store_true")
    mode.add_argument("--check", action="store_true")
    osfr = REPO / "prototype" / "osfr-tools"
    parser.add_argument("--content", default=osfr / "data" / "content")
    parser.add_argument("--items", default=osfr / "data" / "release")
    parser.add_argument("--locale", default=LAB / "reports" / "locale-id-to-text.json")
    parser.add_argument("--legacy", default=LAB / "references" / "FreeRealms-Legacy" / "src" / "Resources" / "ClientItemDefinitions.json")
    parser.add_argument("--sanctuary", default=LAB / "references" / "Sanctuary-main-20261019" / "src" / "Resources" / "Consumables.json")
    parser.add_argument("--audits", default=LAB / "autopilot" / "opus" / "audits" / "events")
    parser.add_argument("--b3", default=LAB / "autopilot" / "opus" / "audits" / "events" / "B3-new-items.json")
    args = parser.parse_args()

    path = Path(args.content) / "consumables.json"
    cake_path = Path(args.content) / "placed_cakes.json"
    data, cakes = load_json(path), load_json(cake_path)
    rows, props, pending, items, locale, b3 = wanted_rows(args)
    shipped = {row["definition"]: row for row in data["consumables"]}
    shipped_props = {row["item_id"]: row for row in cakes["cakes"]}
    if args.apply:
        potions, rest = data["consumables"][:2], data["consumables"][2:]
        merged = {row["definition"]: row for row in rest}
        merged.update(rows)
        data["consumables"] = potions + [merged[k] for k in sorted(merged)]
        path.write_bytes((json.dumps(data, indent=2) + "\n").encode())
        kept = [row for row in cakes["cakes"] if row["item_id"] not in props]
        cakes["cakes"] = kept + [props[k] for k in sorted(props)]
        cake_path.write_bytes((json.dumps(cakes, indent=2) + "\n").encode())
        print(f"B1: {len(rows)} rows written ({len(props)} placed), {len(data['consumables'])} in all")
        for key in pending:
            print(f"B1: pending (no item row yet): {key}")
        return 0

    problems = []
    for item_id, row in rows.items():
        if shipped.get(item_id) != row:
            problems.append(f"row {item_id} differs from the table (run --apply)")
    for item_id, row in props.items():
        if shipped_props.get(item_id) != row:
            problems.append(f"placed row {item_id} differs from the table (run --apply)")
    left = {resolve(k, items, locale, b3) or k: k for k in LEFT}
    if Path(args.audits).exists():
        keys = {k: k for k, *_ in ROWS + PLACED}
        for wanted in sorted(audited_ids(args.audits), key=str):
            covered = wanted in shipped or wanted in left or wanted in keys
            if not covered and isinstance(wanted, str):
                covered = any(isinstance(k, str) and k.lower() in wanted.lower() for k in list(keys) + list(LEFT))
            if not covered:
                problems.append(f"audited consumable {wanted!r} has no row and no LEFT reason")
    for problem in problems:
        print(f"B1: {problem}")
    print(f"B1: {len(rows)} rows checked, {len(LEFT)} left with reasons, "
          f"{len(pending)} waiting for an item row: {', '.join(map(str, pending)) or 'none'}")
    return 1 if problems else 0


if __name__ == "__main__":
    sys.exit(main())
