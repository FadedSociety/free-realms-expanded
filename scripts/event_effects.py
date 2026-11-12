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


def restore(health=0, energy=0, over_ms=0, shared=False, cast=0, instant=0):
    # C3: a health, energy, replenishment or shared potion; zero fields are left out.
    body = {"health_percent": health, "energy_percent": energy, "instant_percent": instant,
            "over_ms": over_ms, "shared": shared, "cast_effect_id": cast}
    return {"restore": {k: v for k, v in body.items() if v}}


def strike(**fields):
    # C3: a battle item's hold (stun_ms, sleep_ms) and radius, as the L088/L089 rows.
    return {"strike": fields}


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
    (3084, "codes", restore(health=50, energy=35, cast=16326), 30_000,
     "item text 'restores some of your health and energy instantly'; fandom wiki Medium Replenishment Potion 'Instantly restores 35 energy. Instantly heals you for 50% health.' (C3); cast 16326 and a 30 s recast (R, " + FORK + " ability 4700)"),
    (60563, "codes", restore(health=50, energy=35, cast=16326), 30_000,
     "item text 'restores some of your health and energy instantly'; fandom wiki Medium Replenishment Potion 'Instantly restores 35 energy. Instantly heals you for 50% health.' (C3); cast 16326 and a 30 s recast (R, " + FORK + " ability 4700)"),
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

# C3: the item-behaviour audit's usable consumables that did nothing
# (lab/autopilot/opus/audits/systems/item_behaviour.json, 2026-11-12). The fork's ItemAbilities.json
# (BubbaJoeX ClientMods/extract_item_abilities.py) reads its tiers from the item names and text
# (Small 25, Medium 50, Large 75 ...), so each value it gives is (R) as well; a row names the
# ability it took. Foods keep the B1 recast (the effect length); potions, scrolls, fireworks and
# orbs take the fork's.
C3 = "C3 item_behaviour"


def transform_outcome(model):
    return {"transform": {"model_id": model, "poof_effect_id": POOF}}


def aura_outcome(effect):
    return {"aura": {"composite_effect_id": effect}}


C3_ROWS = [
    (30, C3, restore(health=40, over_ms=15_000, cast=16324), 30000,
     "item text 'This potion heals over a set amount of time.'; 40% health over 15 s, cast 16324; recast 30 s (" + FORK + " ability 3093)"),
    (32, C3, restore(health=55, over_ms=15_000, cast=16324), 30000,
     "item text 'This potion heals over a set amount of time.'; 55% health over 15 s, cast 16324; recast 30 s (" + FORK + " ability 1599)"),
    (34, C3, restore(health=60, over_ms=15_000, cast=16324), 30000,
     "item text 'This potion heals over a set amount of time.'; 60% health over 15 s, cast 16324; recast 30 s (" + FORK + " ability 1601)"),
    (83, C3, aura(15148, 30 * MIN), None,
     "item text 'Eating this food will cause purple bubbles to appear around the player.'; effect 15148 for 30 minutes (" + FORK + " ability 1971)"),
    (175, C3, restore(health=50, cast=16324), 30000,
     "item text 'Health FTW!'; 50% health at once, cast 16324; recast 30 s (" + FORK + " ability 1207)"),
    (176, C3, restore(energy=50, cast=16325), 30000,
     "item text 'Revitalizes your mana.'; 50% energy at once, cast 16325; recast 30 s (" + FORK + " ability 1208)"),
    (1146, C3, stat(1, 712, 30 * MIN), None,
     "item text 'Carton of milk that boosts your health when consumed.'; fandom wiki Whole Milk Carton 'increases your health by 712 in combat', the max-health foods' line (Honey Roasted Moonbean's too): MaxHealth +712 for 30 minutes (R: the food length); the buff row shows, the encounter engine has no max-health bonus yet"),
    (1147, C3, stat(27, 2, 30 * MIN), None,
     "item text 'Carton of low fat milk that temporarily increases your timing as a Blacksmith, Chef, Miner, and Postman.'; stat 27 Timing +2 for 30 minutes (" + FORK + " ability 236)"),
    (1148, C3, stat(24, 2, 30 * MIN), None,
     "item text 'Carton of non-fat milk that temporarily increases your talent as a Blacksmith, Chef, Miner, and Postman.'; stat 24 Talent +2 for 30 minutes (" + FORK + " ability 237)"),
    (1167, C3, burst(15658, 10_000), 10000,
     "item text 'This launches a red pinwheel firework into the air.'; effect 15658 for 10 s (" + FORK + " ability 4068)"),
    (1168, C3, burst(15659, 10_000), 10000,
     "item text 'This launches a green pinwheel firework into the air.'; effect 15659 for 10 s (" + FORK + " ability 4069)"),
    (1169, C3, burst(15660, 10_000), 10000,
     "item text 'This launches a purple pinwheel firework into the air.'; effect 15660 for 10 s (" + FORK + " ability 4070)"),
    (1194, C3, stat(24, 2, 30 * MIN), None,
     "item text 'All you need to do now is shuffle!'; stat 24 Talent +2 for 30 minutes (" + FORK + " ability 228)"),
    (1196, C3, restore(energy=35, instant=2, over_ms=3_000, cast=16325), 30000,
     "item text 'A potion that restores some energy upon consumption.'; fandom wiki Wheeler Energy Potion 'Restores 11 energy/second for 3 seconds. Instantly restores 2 energy.': 35 of max energy 100, 2 at once and the rest over the wiki's 3 s (the item text gives no length); cast 16325 and recast 30 s (" + FORK + " ability 4694, R)"),
    (1914, C3, stat(18, 50, 30_000), 3_600_000,
     "item text 'reduces the damage you take from opponents for 30 seconds ... will slowly refill itself'; fandom wiki Darkthorne's Embrace 'Increases damage reduction by 50%' and 'a cool down period of one hour': 50% less damage taken for 30 s, recast one hour"),
    (2265, C3, burst(5376, 3_000), 5000,
     "item text 'Caution!: Extremely Potent.'; effect 5376 for 3 s (R: the fork plays it once) (" + FORK + " ability 1932)"),
    (2271, C3, restore(health=35, instant=2, over_ms=3_000, cast=16324), 10000,
     "item text 'Eating this pizza will heal you.'; fandom wiki Mushroom Pizza Slice 'Heals you for 11% health/second for 3 seconds. Instantly heals you for 2% health.': 35%, 2% at once, the rest over 3 s; cast 16324; recast 10 s (R: the Small Health Potion 2173's)"),
    (2580, C3, transform(29, 15 * MIN), None,
     "item text 'Eating this cookie makes you batty.'; fandom wiki Batwing Cookie card 1L54 'Creates the illusion that you are a shrieker for 15 minutes.'; model 29 screecher_01.adr (R: the card's shrieker; the fork's bat 28 bat_01.adr is the other bat; check in model-view)"),
    (2581, C3, transform(212, 15 * MIN), None,
     "item text 'Chicken Feed Brand Soup... It'll make you feel younger!'; fandom wiki Chicken Feed card 'Creates the illusion that you are a baby chick for 15 minutes.'; model 212 chick.adr (" + FORK + " ability 1914)"),
    (2582, C3, scale(0.5, 15 * MIN), None,
     "item text 'Eating this cupcake makes the world get bigger.'; fandom wiki Cupcake of Tim the Tiny card 1L63 'Eating this cupcake will make you tiny like Tim for 15 minutes.'; scale 0.5 (R, " + FORK + " ability 1915)"),
    (2585, C3, transform(47, 15 * MIN), None,
     "item text 'This cold magical treat reminds you of penguins.'; fandom wiki Ice Cream Sandwich card 1L59 'Creates the illusion that you are a baby penguin for 15 minutes.'; model 47 penguin_baby_01.adr (the fork's 16 is the grown penguin)"),
    (2589, C3, transform(4, 15 * MIN), None,
     "item text 'You'll feel funny after eating this apple...'; fandom wiki Goblin Smith Apple card 1L67 'Creates the illusion that you are a robgoblin for 15 minutes.'; model 4 robgoblin_m_basic.adr (" + FORK + " ability 1922)"),
    (2590, C3, transform(199, 15 * MIN), None,
     "item text 'Thugawugs love to burgle burgers.'; fandom wiki Bandit Burger card 1L53 'Creates the illusion that you are a thugawug for 15 minutes.'; model 199 chugawug_m_bandit_01.adr (" + FORK + " ability 1923)"),
    (2591, C3, restore(health=75, over_ms=5_000, cast=16324), 30000,
     "item text 'A potion that restores some of your health over 5 seconds.'; fandom wiki Large Health Potion 'Heals you for 25% health/second for 3 seconds.': 75% over our client's 5 s; cast 16324 and recast 30 s (" + FORK + " ability 4698, R)"),
    (2592, C3, restore(energy=50, instant=2, over_ms=5_000, cast=16325), 30000,
     "item text 'A potion that restores some of your energy over 5 seconds.'; fandom wiki Large Energy Potion 'Restores 16 energy/second for 3 seconds. Instantly restores 2 energy.': 50, 2 at once, the rest over our client's 5 s; cast 16325 and recast 30 s (" + FORK + " ability 4695, R)"),
    (2593, C3, restore(health=75, energy=50, cast=16326), 30000,
     "item text 'A potion that restores some of your health and energy instantly.'; fandom wiki Large Replenishment Potion 'Instantly restores 50 energy. Instantly heals you for 75% health.'; cast 16326 and recast 30 s (" + FORK + " ability 4701, R)"),
    (2595, C3, transform(187, 30 * MIN), None,
     "item text 'A 'Qualitee' Robgoblin disguise.'; model 187 for 30 minutes (" + FORK + " ability 1927)"),
    (2596, C3, transform(88, 30 * MIN), None,
     "item text 'A 'Qualitee' Plant disguise.'; model 88 for 30 minutes (" + FORK + " ability 1928)"),
    (2598, C3, transform(47, 30 * MIN), None,
     "item text 'A 'Qualitee' Baby Penguin disguise.'; model 47 for 30 minutes (" + FORK + " ability 1930)"),
    (2602, C3, aura(15459, 15 * MIN), None,
     "item text 'This magical candy fills you with love.'; fandom wiki Heart Candy card 1L58 'Causes hearts to float from your head for 15 minutes.'; effect 15459 (" + FORK + " ability 1934)"),
    (2646, C3, aura(15362, 15 * MIN), None,
     "item text 'Open the bottle to let out the butterflies!'; fandom wiki Bottle of Butterflies card 1L56 'they will fly around you for 15 minutes'; effect 15362 PFX_butterflies_swirl (R: the Butterfly Swirls Potion's loop; the fork's one-shot 5456 flies away)"),
    (2654, C3, boost("coins", 100, 120 * MIN), 5000,
     "item text 'Feeling lucky?  Read this scroll to increase your fortune!'; fandom wiki Crazy Coin Scroll card 1L61 'Increases coin rewards by 100% for two hours.'; recast 5 s (R, the L087 boosters')"),
    (2655, C3, boost("coins", 100, 120 * MIN), 5000,
     "item text 'Feeling lucky? Read this scroll to increase your fortune!'; fandom wiki Crazy Coin Scroll card 1L61 'Increases coin rewards by 100% for two hours.'; recast 5 s (R, the L087 boosters')"),
    (2656, C3, aura(15531, 15 * MIN), None,
     "item text 'It feels like spring time.'; fandom wiki Crimson Rose card 1L62 'Causes rose petals to follow your footsteps for 15 minutes.'; effect 15531 PFX_rose_petal_trail_feet (" + FORK + " ability 224)"),
    (2659, C3, boost("stars", 100, 120 * MIN), 5000,
     "item text 'Learning just got a little bit easier.'; fandom wiki Super Star Scroll card 1L98 'Increases your experience by 100% for two hours.'; recast 5 s (R, the L087 boosters')"),
    (2660, C3, boost("stars", 100, 120 * MIN), 5000,
     "item text 'Learning just got a little bit easier.'; fandom wiki Super Star Scroll card 1L98 'Increases your experience by 100% for two hours.'; recast 5 s (R, the L087 boosters')"),
    (2675, C3, restore(health=45, cast=16324), 30000,
     "item text 'This jumbo potion restores a large portion of your health.'; no wiki page: 45% at once (R: the Jumbo Energy Potion's wiki 42% - 47%); cast 16324 and recast 30 s (" + FORK + " ability 230, R)"),
    (2676, C3, restore(energy=45, cast=16325), 30000,
     "item text 'This jumbo potion restores a large portion of your energy.'; fandom wiki Jumbo Energy Potion 'Restores up to 42% - 47% of your energy.': 45 at once (R: within the wiki's range); cast 16325 and recast 30 s (" + FORK + " ability 229, R)"),
    (2677, C3, restore(health=20, energy=20, cast=16326), 30000,
     "item text 'This jumbo potion restores a large portion of your health and energy.'; fandom wiki Jumbo Replenishing Potion 'Restores up to 19% - 21% of your energy. Heals up to 19% - 21% of your health.': 20 and 20 at once (R: within the range); cast 16326 and recast 30 s (" + FORK + " ability 231, R)"),
    (2697, C3, aura(5568, 15 * MIN), None,
     "item text 'You're in the spotlight now!'; fandom wiki Superstar Spotlight card 1L99 'Directs the spotlight at you for 15 minutes.'; effect 5568 PFX_light_white_root_god-beam_sm_loop (" + FORK + " ability 3794)"),
    (2956, C3, transform(219, 30 * MIN), None,
     "item text 'Eating this food will temporarily change the player into a Bixie.'; model 219 for 30 minutes (" + FORK + " ability 1981)"),
    (2964, C3, aura(15891, 30 * MIN), None,
     "item text 'Eating this food will cause a storm to appear around the player for thirty minutes.'; effect 15891 for 30 minutes (" + FORK + " ability 1974)"),
    (2966, C3, aura(47, 30 * MIN), None,
     "item text 'Eating this food will cause blue lightning to appear around the player for thirty minutes.'; effect 47 for 30 minutes (" + FORK + " ability 1972)"),
    (3021, C3, restore(health=50, instant=2, over_ms=5_000, cast=16324), 30000,
     "item text 'A potion that restores some of your health over 5 seconds.'; fandom wiki Medium Health Potion 'Heals you for 16% health/second for 3 seconds. Instantly heals you for 2% health.': 50%, 2% at once, the rest over our client's 5 s; cast 16324 and recast 30 s (" + FORK + " ability 4697, R)"),
    (3023, C3, restore(energy=35, instant=2, over_ms=5_000, cast=16325), 30000,
     "item text 'A potion that restores some of your energy over 5 seconds.'; fandom wiki Medium Energy Potion 'Restores 11 energy/second for 3 seconds. Instantly restores 2 energy.': 35 of max energy 100, 2 at once, the rest over our client's 5 s; cast 16325 and recast 30 s (" + FORK + " ability 4694, R)"),
    (3082, C3, restore(health=25, energy=20, cast=16326), 30000,
     "item text 'A potion that restores some of your health and energy instantly.'; no wiki page: 25% health and 20 energy (R: below the Medium Replenishment Potion's wiki 50% and 35 and the Large's 75% and 50); cast 16326 and recast 30 s (" + FORK + " ability 4699, R)"),
    (3992, C3, burst(15729, 10_000), 10000,
     "item text 'Launches fireworks into the sky!'; effect 15729 for 10 s (R: the fork plays it once) (" + FORK + " ability 3955)"),
    (3995, C3, restore(health=75, over_ms=3_000, cast=16324), 30000,
     "item text 'A huge potion that heals lots of health.'; fandom wiki Ginormous Health Potion 'Heals you for 25% health/second for 3 seconds.': 75% over 3 s; cast 16324 and recast 30 s (" + FORK + " ability 4698, R)"),
    (11005, C3, restore(energy=20, instant=2, over_ms=5_000, cast=16325), 30000,
     "item text 'A potion that restores some of your energy over 5 seconds.'; no wiki page: 20, 2 at once (R: one step below the Medium Energy Potion's wiki 35, as the Large is one above at 50), over our client's 5 s; cast 16325 and recast 30 s (" + FORK + " ability 4693, R)"),
    (16033, C3, burst(15656, 10_000), 10000,
     "item text 'This firework launches golden sparkles into the air.'; effect 15656 for 10 s (" + FORK + " ability 4066)"),
    (16034, C3, burst(15658, 10_000), 10000,
     "item text 'This launches a red pinwheel firework into the air.'; effect 15658 for 10 s (" + FORK + " ability 4068)"),
    (16035, C3, burst(15659, 10_000), 10000,
     "item text 'This launches a green pinwheel firework into the air.'; effect 15659 for 10 s (" + FORK + " ability 4069)"),
    (16036, C3, burst(15660, 10_000), 10000,
     "item text 'This launches a purple pinwheel firework into the air.'; effect 15660 for 10 s (" + FORK + " ability 4070)"),
    (21807, C3, stat(1, 712, 30 * MIN), None,
     "item text 'Eating this food temporarily increases your health.'; MaxHealth +712 for 30 minutes (R: fandom wiki Honey Roasted Moonbean 'increases your health by 712 in combat', its sibling); the buff row shows, the encounter engine has no max-health bonus yet (" + FORK + " ability 2626 gives +250)"),
    (21809, C3, stat(5, 25, 30 * MIN), None,
     "item text 'Eating this food temporarily increases your energy.'; stat 5 MaxMana +25 (the buff row shows; the encounter engine has no max health or energy bonus yet) for 30 minutes (" + FORK + " ability 2627)"),
    (21810, C3, stat(14, 8, 30 * MIN), None,
     "item text 'Eating this food temporarily increases your attack.'; stat 14 Attack: the fork's +20 weapon damage is 8% more damage dealt (R: +20 on the tier-1 basic hit of 254) for 30 minutes (" + FORK + " ability 2622)"),
    (21811, C3, stat(24, 2, 30 * MIN), None,
     "item text 'Eating this food temporarily increases your talent as a Blacksmith, Chef, Miner, and Postman.'; stat 24 Talent +2 for 30 minutes (" + FORK + " ability 438)"),
    (21812, C3, stat(24, 2, 30 * MIN), None,
     "item text 'Eating this food temporarily increases your talent as a Blacksmith, Chef, Miner, and Postman.'; stat 24 Talent +2 for 30 minutes (" + FORK + " ability 439)"),
    (21814, C3, stat(24, 2, 30 * MIN), None,
     "item text 'Eating this food temporarily increases your talent as a Blacksmith, Chef, Miner, and Postman.'; stat 24 Talent +2 for 30 minutes (" + FORK + " ability 440)"),
    (21815, C3, stat(24, 2, 30 * MIN), None,
     "item text 'Eating this food temporarily increases your talent as a Blacksmith, Chef, Miner, and Postman.'; stat 24 Talent +2 for 30 minutes (" + FORK + " ability 429)"),
    (21816, C3, stat(24, 2, 30 * MIN), None,
     "item text 'Eating this food temporarily increases your talent as a Blacksmith, Chef, Miner, and Postman.'; stat 24 Talent +2 for 30 minutes (" + FORK + " ability 430)"),
    (21817, C3, stat(24, 2, 30 * MIN), None,
     "item text 'Eating this food temporarily increases your talent as a Blacksmith, Chef, Miner, and Postman.'; stat 24 Talent +2 for 30 minutes (" + FORK + " ability 2628)"),
    (21818, C3, stat(1, 712, 30 * MIN), None,
     "item text 'Eating this food temporarily increases your health.'; MaxHealth +712 for 30 minutes (R: fandom wiki Honey Roasted Moonbean 'increases your health by 712 in combat', its sibling); the buff row shows, the encounter engine has no max-health bonus yet (" + FORK + " ability 2635 gives +250)"),
    (21819, C3, stat(5, 25, 30 * MIN), None,
     "item text 'Eating this food temporarily increases your energy.'; stat 5 MaxMana +25 (the buff row shows; the encounter engine has no max health or energy bonus yet) for 30 minutes (" + FORK + " ability 2636)"),
    (21820, C3, stat(14, 8, 30 * MIN), None,
     "item text 'Eating this food temporarily increases your attack.'; stat 14 Attack: the fork's +20 weapon damage is 8% more damage dealt (R: +20 on the tier-1 basic hit of 254) for 30 minutes (" + FORK + " ability 2631)"),
    (21821, C3, stat(24, 2, 30 * MIN), None,
     "item text 'Eating this food temporarily increases your talent as a Blacksmith, Chef, Miner, and Postman.'; stat 24 Talent +2 for 30 minutes (" + FORK + " ability 453)"),
    (21822, C3, stat(24, 2, 30 * MIN), None,
     "item text 'Eating this food temporarily increases your talent as a Blacksmith, Chef, Miner, and Postman.'; stat 24 Talent +2 for 30 minutes (" + FORK + " ability 441)"),
    (21823, C3, stat(27, 2, 30 * MIN), None,
     "item text 'Eating this food temporarily increases your timing as a Blacksmith, Chef, Miner, and Postman.'; stat 27 Timing +2 for 30 minutes (" + FORK + " ability 2637)"),
    (21825, C3, stat(24, 2, 30 * MIN), None,
     "item text 'Eating this food temporarily increases your talent as a Blacksmith, Chef, Miner, and Postman.'; stat 24 Talent +2 for 30 minutes (" + FORK + " ability 454)"),
    (21826, C3, stat(24, 2, 30 * MIN), None,
     "item text 'Eating this food temporarily increases your talent as a Blacksmith, Chef, Miner, and Postman.'; stat 24 Talent +2 for 30 minutes (" + FORK + " ability 455)"),
    (21827, C3, stat(24, 2, 30 * MIN), None,
     "item text 'Eating this food temporarily increases your talent as a Blacksmith, Chef, Miner, and Postman.'; stat 24 Talent +2 for 30 minutes (" + FORK + " ability 445)"),
    (21828, C3, scale(1.15, 30 * MIN), None,
     "item text 'Eating this food will make you larger.'; scale 1.15 for 30 minutes (" + FORK + " ability 1950)"),
    (21829, C3, stat(1, 712, 30 * MIN), None,
     "item text 'Eating this food temporarily increases your health.'; fandom wiki Honey Roasted Moonbean 'increases your health by 712 in combat': MaxHealth +712 for 30 minutes (R: the food length); the buff row shows, the encounter engine has no max-health bonus yet"),
    (21830, C3, stat(5, 25, 30 * MIN), None,
     "item text 'Eating this food temporarily increases your energy.'; stat 5 MaxMana +25 (the buff row shows; the encounter engine has no max health or energy bonus yet) for 30 minutes (" + FORK + " ability 2645)"),
    (21831, C3, stat(14, 8, 30 * MIN), None,
     "item text 'Eating this food temporarily increases your attack.'; stat 14 Attack: the fork's +20 weapon damage is 8% more damage dealt (R: +20 on the tier-1 basic hit of 254) for 30 minutes (" + FORK + " ability 2640)"),
    (21832, C3, stat(24, 2, 30 * MIN), None,
     "item text 'Eating this food temporarily increases your talent as a Blacksmith, Chef, Miner, and Postman.'; stat 24 Talent +2 for 30 minutes (" + FORK + " ability 456)"),
    (21833, C3, stat(24, 2, 30 * MIN), None,
     "item text 'Eating this food temporarily increases your talent as a Blacksmith, Chef, Miner, and Postman.'; stat 24 Talent +2 for 30 minutes (" + FORK + " ability 459)"),
    (21834, C3, scale(0.85, 30 * MIN), None,
     "item text 'Eating this food will make you smaller.'; scale 0.85 for 30 minutes (" + FORK + " ability 1945)"),
    (31221, C3, stat(24, 2, 30 * MIN), None,
     "item text 'Eating this food temporarily increases your talent as a Blacksmith, Chef, Miner, and Postman.'; stat 24 Talent +2 for 30 minutes (" + FORK + " ability 391)"),
    (31223, C3, stat(24, 2, 30 * MIN), None,
     "item text 'Eating this food temporarily increases your talent as a Blacksmith, Chef, Miner, and Postman.'; stat 24 Talent +2 for 30 minutes (" + FORK + " ability 393)"),
    (31224, C3, stat(27, 2, 30 * MIN), None,
     "item text 'Eating this food temporarily increases your timing as a Blacksmith, Chef, Miner, and Postman.'; stat 27 Timing +2 for 30 minutes (" + FORK + " ability 394)"),
    (31230, C3, stat(24, 2, 30 * MIN), None,
     "item text 'Eating this food temporarily increases your talent as a Blacksmith, Chef, Miner, and Postman.'; stat 24 Talent +2 for 30 minutes (" + FORK + " ability 400)"),
    (31231, C3, stat(24, 2, 30 * MIN), None,
     "item text 'Eating this food temporarily increases your talent as a Blacksmith, Chef, Miner, and Postman.'; stat 24 Talent +2 for 30 minutes (" + FORK + " ability 401)"),
    (31232, C3, stat(27, 2, 30 * MIN), None,
     "item text 'Eating this food temporarily increases your timing as a Blacksmith, Chef, Miner, and Postman.'; stat 27 Timing +2 for 30 minutes (" + FORK + " ability 402)"),
    (31240, C3, stat(24, 2, 30 * MIN), None,
     "item text 'Eating this food temporarily increases your talent as a Blacksmith, Chef, Miner, and Postman.'; stat 24 Talent +2 for 30 minutes (" + FORK + " ability 410)"),
    (31244, C3, stat(27, 2, 30 * MIN), None,
     "item text 'Eating this food temporarily increases your timing as a Blacksmith, Chef, Miner, and Postman.'; stat 27 Timing +2 for 30 minutes (" + FORK + " ability 414)"),
    (31250, C3, stat(24, 2, 30 * MIN), None,
     "item text 'Eating this food temporarily increases your talent as a blacksmith, chef, miner, and postman.'; stat 24 Talent +2 for 30 minutes (" + FORK + " ability 420)"),
    (31253, C3, stat(24, 2, 30 * MIN), None,
     "item text 'Eating this food temporarily increases your talent as a blacksmith, chef, miner, and postman.'; stat 24 Talent +2 for 30 minutes (" + FORK + " ability 423)"),
    (31255, C3, stat(27, 2, 30 * MIN), None,
     "item text 'Eating this food temporarily increases your timing as a blacksmith, chef, miner, and postman.'; stat 27 Timing +2 for 30 minutes (" + FORK + " ability 425)"),
    (31256, C3, stat(24, 2, 30 * MIN), None,
     "item text 'Eating this food temporarily increases your talent as a Blacksmith, Chef, Miner, and Postman.'; stat 24 Talent +2 for 30 minutes (" + FORK + " ability 426)"),
    (31301, C3, stat(24, 2, 30 * MIN), None,
     "item text 'Eating this food temporarily increases your talent as a blacksmith, chef, miner, and postman.'; stat 24 Talent +2 for 30 minutes (" + FORK + " ability 471)"),
    (31313, C3, stat(24, 2, 30 * MIN), None,
     "item text 'Eating this food temporarily increases your talent as a blacksmith, chef, miner, and postman.'; stat 24 Talent +2 for 30 minutes (" + FORK + " ability 483)"),
    (31315, C3, stat(27, 2, 30 * MIN), None,
     "item text 'Eating this food temporarily increases your timing as a blacksmith, chef, miner, and postman.'; stat 27 Timing +2 for 30 minutes (" + FORK + " ability 485)"),
    (48155, C3, transform(4470, 30 * MIN), None,
     "item text 'This amazing potion turns you into a bug!'; model 4470 for 30 minutes (" + FORK + " ability 3712)"),
    (48156, C3, strike(stun_ms=5000), 10000,
     "item text 'Use this orb to cocoon your opponent.'; stun 5 s (" + FORK + " ability 3717)"),
    (49997, C3, restore(health=75, over_ms=3_000, shared=True, cast=16324), 30000,
     "item text 'A special potion that heals the Health of you and all your pals nearby.'; fandom wiki Shared Health Potion 'Heals you for 25% health/second for 3 seconds.': 75% over 3 s for the user; the user's group in the same place gets the 75% at once through the group-heal route (R: the fork's 15 m pals); cast 16324 and recast 30 s (" + FORK + " ability 4722, R)"),
    (49998, C3, restore(energy=50, instant=2, over_ms=3_000, cast=16325), 30000,
     "item text 'use this potion during Combat to restore Energy for everyone'; fandom wiki Shared Energy Potion 'Restores 16 energy/second for 3 seconds. Instantly restores 2 energy.': 50, 2 at once, over 3 s, for the user; the 'for everyone' half is not applied (R: the group-heal route carries health only); cast 16325 and recast 30 s (" + FORK + " ability 4723, R)"),
    (49999, C3, restore(health=75, energy=50, shared=True, cast=16326), 30000,
     "item text 'Sharing is caring with this cure all potion for both Energy and Health.'; fandom wiki Shared Replenishment Potion 'Instantly restores 50 energy. Instantly heals your for 75% health.'; the user's group in the same place gets the 75% health (R: the group-heal route carries health only); cast 16326 and recast 30 s (" + FORK + " ability 4724, R)"),
    (55404, C3, aura(15208, 30 * MIN), None,
     "item text 'This potion gives humans button eyes!'; effect 15208 for 30 minutes (" + FORK + " ability 3855)"),
    (55405, C3, aura(15222, 30 * MIN), None,
     "item text 'This potion gives pixies button eyes!'; effect 15222 for 30 minutes (" + FORK + " ability 3856)"),
    (55406, C3, aura(15291, 30 * MIN), None,
     "item text 'This potion gives humans button eyes!'; effect 15291 for 30 minutes (" + FORK + " ability 3857)"),
    (55407, C3, aura(15292, 30 * MIN), None,
     "item text 'This potion gives pixies button eyes!'; effect 15292 for 30 minutes (" + FORK + " ability 3858)"),
    (55408, C3, aura(15294, 30 * MIN), None,
     "item text 'This potion gives humans button eyes!'; effect 15294 for 30 minutes (" + FORK + " ability 3859)"),
    (55409, C3, aura(15296, 30 * MIN), None,
     "item text 'This potion gives pixies button eyes!'; effect 15296 for 30 minutes (" + FORK + " ability 3860)"),
    (55410, C3, aura(15297, 30 * MIN), None,
     "item text 'This potion gives humans button eyes!'; effect 15297 for 30 minutes (" + FORK + " ability 3861)"),
    (55411, C3, aura(15298, 30 * MIN), None,
     "item text 'This potion gives pixies button eyes!'; effect 15298 for 30 minutes (" + FORK + " ability 3862)"),
    (55412, C3, aura(15299, 30 * MIN), None,
     "item text 'This potion gives humans button eyes!'; effect 15299 for 30 minutes (" + FORK + " ability 3863)"),
    (55413, C3, aura(15300, 30 * MIN), None,
     "item text 'This potion gives pixies button eyes!'; effect 15300 for 30 minutes (" + FORK + " ability 3864)"),
    (55414, C3, aura(15303, 30 * MIN), None,
     "item text 'This potion gives humans button eyes!'; effect 15303 for 30 minutes (" + FORK + " ability 3866)"),
    (55415, C3, aura(15301, 30 * MIN), None,
     "item text 'This potion gives pixies button eyes!'; effect 15301 for 30 minutes (" + FORK + " ability 3867)"),
    (55416, C3, aura(15304, 30 * MIN), None,
     "item text 'This potion gives humans button eyes!'; effect 15304 for 30 minutes (" + FORK + " ability 3868)"),
    (55417, C3, aura(15305, 30 * MIN), None,
     "item text 'This potion gives pixies button eyes!'; effect 15305 for 30 minutes (" + FORK + " ability 3869)"),
    (55418, C3, aura(15306, 30 * MIN), None,
     "item text 'This potion gives humans button eyes!'; effect 15306 for 30 minutes (" + FORK + " ability 3870)"),
    (55419, C3, aura(15307, 30 * MIN), None,
     "item text 'This potion gives pixies button eyes!'; effect 15307 for 30 minutes (" + FORK + " ability 3871)"),
    (55420, C3, aura(15363, 30 * MIN), None,
     "item text 'This potion attracts fireflies to you!'; effect 15363 for 30 minutes (" + FORK + " ability 3873)"),
    (55421, C3, aura(15362, 30 * MIN), None,
     "item text 'This potion attracts butterflies to you!'; effect 15362 for 30 minutes (" + FORK + " ability 3874)"),
    (55422, C3, aura(15369, 30 * MIN), None,
     "item text 'This potion showers you in flowers!'; effect 15369 for 30 minutes (" + FORK + " ability 3875)"),
    (55850, C3, strike(stun_ms=5000, radius=10.0), 10000,
     "item text 'Freezes up to five opponents for five seconds.'; stun 5 s within 10 m (the fork's grenade centres on the user; ours on the enemy it lands on, R) (the fork's cap of 5 targets is not applied, R) (" + FORK + " ability 3954)"),
    (57022, C3, burst(15656, 10_000), 10000,
     "item text 'This firework launches golden sparkles into the air.'; effect 15656 for 10 s (" + FORK + " ability 4066)"),
    (57023, C3, burst(15657, 10_000), 10000,
     "item text 'This launches a blue pinwheel firework into the air.'; effect 15657 for 10 s (" + FORK + " ability 4067)"),
    (57024, C3, burst(15658, 10_000), 10000,
     "item text 'This launches a red pinwheel firework into the air.'; effect 15658 for 10 s (" + FORK + " ability 4068)"),
    (57025, C3, burst(15659, 10_000), 10000,
     "item text 'This launches a green pinwheel firework into the air.'; effect 15659 for 10 s (" + FORK + " ability 4069)"),
    (57026, C3, burst(15660, 10_000), 10000,
     "item text 'This launches a purple pinwheel firework into the air.'; effect 15660 for 10 s (" + FORK + " ability 4070)"),
    (57027, C3, burst(15664, 10_000), 10000,
     "item text 'This firework launches golden sparkles into the air.'; effect 15664 for 10 s (" + FORK + " ability 4071)"),
    (57028, C3, burst(15661, 10_000), 10000,
     "item text 'This firwork launches stars and stripes into the air.'; effect 15661 for 10 s (R: the fork plays it once) (" + FORK + " ability 4072)"),
    (60259, C3, stat(18, 25, 1 * MIN), None,
     "item text 'Spider Anti-venom reduces damage taken.'; stat 18 Damage Reduction +25 for 1 minute (" + FORK + " ability 4073)"),
    (60262, C3, restore(health=35, instant=2, over_ms=3_000, cast=16324), 10000,
     "item text 'This tasty pizza will restore some health!'; fandom wiki Mushroom Pizza Slice 'Heals you for 11% health/second for 3 seconds. Instantly heals you for 2% health.': 35%, 2% at once, the rest over 3 s; cast 16324; recast 10 s (R: the Small Health Potion 2173's)"),
    (60271, C3, aura(5232, 30 * MIN), None,
     "item text 'Eating this food will cause shadow flames to appear around the player for thirty minutes.'; effect 5232 for 30 minutes (" + FORK + " ability 4091)"),
    (60275, C3, transform(250, 30 * MIN), None,
     "item text 'Eating this food will change the player into a female chugawug farmer for thirty minutes.'; model 250 for 30 minutes (" + FORK + " ability 4095)"),
    (60986, C3, transform(1385, 30 * MIN), None,
     "no item text; model 1385 for 30 minutes (" + FORK + " ability 4132)"),
    (62529, C3, burst(15813, 10_000), 10000,
     "item text 'This party popper blasts confetti into the air.'; effect 15813 for 10 s (" + FORK + " ability 4161)"),
    (65820, C3, restore(health=35, instant=2, over_ms=3_000, cast=16324), 10000,
     "item text 'Potion that heals the player on consumption.'; fandom wiki Wheeler Health Potion 'Heals you for 11% health/second for 3 seconds. Instantly heals you for 2% health.': 35%, 2% at once, the rest over the wiki's 3 s, as the Small Health Potion 2173's 35%; cast 16324; recast 10 s (R: 2173's)"),
    (68152, C3, stat(48, 10, 10 * MIN), 5000,
     "item text 'This lure gives a 10% better chance of catching Winter Pirahna, Toothy Tetra and Blind Swurglefish.'; stat 48 FishingLuck +10 for 10 minutes (the buff row and stat show; fishing_service::set_lure has no caller yet) (" + FORK + " ability 4287)"),
    (68153, C3, stat(48, 10, 10 * MIN), 5000,
     "item text 'This lure gives a 10% better chance of catching Chilly Grouper, Butter Flyfish and Briar Nibbler.'; stat 48 FishingLuck +10 for 10 minutes (the buff row and stat show; fishing_service::set_lure has no caller yet) (" + FORK + " ability 4288)"),
    (68154, C3, stat(48, 10, 10 * MIN), 5000,
     "item text 'This lure gives a 10% better chance of catching Blubracuda, Baconfish and Changed Salmon.'; stat 48 FishingLuck +10 for 10 minutes (the buff row and stat show; fishing_service::set_lure has no caller yet) (" + FORK + " ability 4289)"),
    (68155, C3, stat(48, 10, 10 * MIN), 5000,
     "item text 'This lure gives a 10% better chance of catching Frostgill Smelt, Flutterfish and Ink Cod.'; stat 48 FishingLuck +10 for 10 minutes (the buff row and stat show; fishing_service::set_lure has no caller yet) (" + FORK + " ability 4290)"),
    (68156, C3, stat(48, 10, 10 * MIN), 5000,
     "item text 'This lure gives a 10% better chance of catching Goofy Grouper, Finless Fish and Fanged Grouper.'; stat 48 FishingLuck +10 for 10 minutes (the buff row and stat show; fishing_service::set_lure has no caller yet) (" + FORK + " ability 4291)"),
    (68157, C3, stat(48, 10, 10 * MIN), 5000,
     "item text 'This lure gives a 10% better chance of catching Plattypus, Lady Tetra and Roach Loach.'; stat 48 FishingLuck +10 for 10 minutes (the buff row and stat show; fishing_service::set_lure has no caller yet) (" + FORK + " ability 4292)"),
    (68158, C3, stat(48, 10, 10 * MIN), 5000,
     "item text 'This lure gives a 10% better chance of catching Pacu Pacu, Peachy Perch and Globfish.'; stat 48 FishingLuck +10 for 10 minutes (the buff row and stat show; fishing_service::set_lure has no caller yet) (" + FORK + " ability 4293)"),
    (68159, C3, stat(48, 10, 10 * MIN), 5000,
     "item text 'This lure gives a 10% better chance of catching Coach Loach, Feral Catfish and Creeping Cod.'; stat 48 FishingLuck +10 for 10 minutes (the buff row and stat show; fishing_service::set_lure has no caller yet) (" + FORK + " ability 4294)"),
    (68160, C3, stat(48, 10, 10 * MIN), 5000,
     "item text 'This lure gives a 10% better chance of catching Frozen Char, Slugmud Skipper and Old Sole.'; stat 48 FishingLuck +10 for 10 minutes (the buff row and stat show; fishing_service::set_lure has no caller yet) (" + FORK + " ability 4295)"),
    (68161, C3, stat(48, 10, 10 * MIN), 5000,
     "item text 'This lure gives a 10% better chance of catching Talking Bass, Cheery Salmon and Bitter Betta.'; stat 48 FishingLuck +10 for 10 minutes (the buff row and stat show; fishing_service::set_lure has no caller yet) (" + FORK + " ability 4296)"),
    (68162, C3, stat(48, 10, 10 * MIN), 5000,
     "item text 'This lure gives a 10% better chance of catching Blue Thornfin, Chipsen Fish and Thorny Trout.'; stat 48 FishingLuck +10 for 10 minutes (the buff row and stat show; fishing_service::set_lure has no caller yet) (" + FORK + " ability 4297)"),
    (68163, C3, stat(48, 10, 10 * MIN), 5000,
     "item text 'This lure gives a 10% better chance of catching Spineless Stickleback, Calico Catfish and Purplenosed Shark.'; stat 48 FishingLuck +10 for 10 minutes (the buff row and stat show; fishing_service::set_lure has no caller yet) (" + FORK + " ability 4298)"),
    (68164, C3, stat(48, 10, 10 * MIN), 5000,
     "item text 'This magnet gives a 10% better chance of catching Treasure.'; stat 48 FishingLuck +10 for 10 minutes (the buff row and stat show; fishing_service::set_lure has no caller yet) (" + FORK + " ability 4299)"),
    (68783, C3, aura(15710, 30 * MIN), None,
     "item text 'Places a halo above the player for thirty minutes.'; effect 15710 for 30 minutes (" + FORK + " ability 4276)"),
    (68908, C3, burst(15813, 10_000), 10000,
     "item text 'This party popper blasts confetti into the air.'; effect 15813 for 10 s (" + FORK + " ability 4161)"),
    (70328, C3, boost("coins", 50, 30 * MIN), 5000,
     "no item text; +50% coins for 30 minutes (" + FORK + " ability 206)"),
    (70329, C3, restore(health=50, instant=2, over_ms=5_000, cast=16324), 30000,
     "no item text; its ability 4697 is the Medium Health Potion's, so its wiki 50% (2% at once, the rest over 5 s, R); cast 16324 and recast 30 s (" + FORK + " ability 4697, R)"),
    (76713, C3, transform(1667, 15 * MIN), None,
     "item text 'Secretly sold from a strange old sea salt in Seaside.  Transforms you into a crab for 15 minutes.'; model 1667 for 15 minutes (" + FORK + " ability 252)"),
    (77041, C3, strike(sleep_ms=8000, radius=10.0), 3_600_000,
     "item text 'This magical flute will put your opponents straight to sleep!'; fandom wiki Flute of Slumber 'a cool down period of one hour'; sleep 8 s within 10 m (R, " + FORK + " ability 270; the fork's grenade centres on the user, ours on the enemy it lands on)"),
    (77417, C3, restore(health=75, over_ms=5_000, cast=16324), 30000,
     "item text 'A potion that restores some of your health over 5 seconds.'; fandom wiki Large Health Potion 'Heals you for 25% health/second for 3 seconds.': 75% over our client's 5 s; cast 16324 and recast 30 s (" + FORK + " ability 4698, R)"),
    # No fork ability, or a reading of the item's own text the fork lacks.
    (35, C3, scale(1.3, 30 * MIN), None,
     "item text 'Eating this food increases the health and size of the player.'; scale 1.3 for 30 minutes (" + FORK + " ability 3264); its +500 MaxHealth has no encounter-engine hook yet, so the size is what shows (R)"),
    (213, C3, aura(15049, MIN), 120_000,
     "item text 'increasing run speed for 60 seconds ... slowly refills itself'; effect 15049 PFX_beam_red_feet_speed-run_loop for 60 s and a 2 minute refill (" + FORK + " ability 250); the +4 m/s run speed is not applied (R: stat 2 belongs to the ride, as for the Gingerbread Cookie)"),
    (1195, C3, stat(24, 2, 30 * MIN), None,
     "item text 'Now made with real corn!'; ability 232 has no fork row; Talent +2 for 30 minutes (R: Sparkling Corn on the Cob shares ability 232 and its text reads 'increases your talent')"),
    (31259, C3, stat(24, 2, 30 * MIN), None,
     "item text 'Eating this food temporarily increases your talent as a Blacksmith, Chef, Miner, and Postman.'; ability 232 has no fork row; Talent +2 for 30 minutes (R: the fork's talent foods)"),
    (2586, C3, transform(1669, 15 * MIN), None,
     "item text 'Eating this candy makes you feel like jumping.'; fandom wiki Lilipad Lollipop card 1L75 'Creates the illusion that you are a frog for 15 minutes.'; model 1669 toad_purple.adr, which Models.txt describes as 'Purple toad duplicate for a tcg item' (B1's frog 2959 is toad.adr 141; check in model-view)"),
    (48104, C3, transform(208, MIN), 60_000,
     "item text 'It's a piece of cake to eat a piece of cake. [You can only eat 1 slice per minute.]'; Sanctuary RandomTransformFoods 48104 rolls the OSFR abilities 900120-900124; the first, model 208 changeling_f_cook_01.adr for 1 minute (R: the roll is not wired); recast 60 s from the text"),
    (56776, C3, transform(88, 30 * MIN), None,
     "item text 'Using this fungus will turn the player into a nettleseed plant for thirty minutes.' (fanbyte Nettleseed_Fungus: unlimited use; its note says one hour); model 88 maneatingplant.adr (R: Nettleseed's man-eating plants); 30 minutes from the item text"),
    (57011, C3, transform(418, 5 * MIN), None,
     "item text 'Don't let your pet lick your face after feeding from the bucket!'; ability 4007 has no fork row; model 418 fish.adr for 5 minutes (R: the Bucket of Bouncing Bait 9295 'transforms you into a fish', client locale 17043 'Illusion: Flopping Fish - 5 minutes')"),
    (60280, C3, restore(health=35, shared=True, cast=16324), 3_600_000,
     "item text 'Nothing says combat party like a mushroom pizza.' (unlimited use); fandom wiki Mushroom Pizza 'a cool down period of one hour' and 'Players who click on a Mushroom Pizza in a combat mini-game receive one Mushroom Pizza Slice': the slice's 35% at once for the user and the user's group in the same place (R: the slices are not handed out); cast 16324"),
    (62516, C3, burst(15825, SPRAY, True), FIREWORK,
     "item text 'Use this to spray silly string all over your favorite things!'; effect 15825 PFX_silly-string_beam_blue_p2p at the aimed player (" + FORK + " PartyFavors ItemId 62516); cooldown (R)"),
    (67961, C3, burst(15818, SPRAY, True), FIREWORK,
     "item text 6 is the client's line-of-sight message (a table slip); effect 15818 red silly string at the aimed player (" + FORK + " PartyFavors ItemId 67961); cooldown (R)"),
    (67962, C3, burst(15826, SPRAY, True), FIREWORK,
     "item text 'Use this to spray silly string all over your favorite things!'; effect 15826 green silly string at the aimed player (" + FORK + " PartyFavors ItemId 67962); cooldown (R)"),
    (67963, C3, burst(15827, SPRAY, True), FIREWORK,
     "item text 'Use this to spray silly string all over your favorite things!'; effect 15827 PFX_silly-string_beam_orange_p2p at the aimed player (" + FORK + " PartyFavors ItemId 67963); cooldown (R)"),
    (67964, C3, burst(15828, SPRAY, True), FIREWORK,
     "item text 'Use this to spray silly string all over your favorite things!'; effect 15828 yellow silly string at the aimed player (" + FORK + " PartyFavors ItemId 67964); cooldown (R)"),
    (68909, C3, burst(15818, SPRAY, True), FIREWORK,
     "item text 'Use this to spray silly string all over your favorite things!'; effect 15818 red silly string at the aimed player (" + FORK + " PartyFavors ItemId 68909); cooldown (R)"),
    (68910, C3, burst(15826, SPRAY, True), FIREWORK,
     "item text 'Use this to spray silly string all over your favorite things!'; effect 15826 green silly string at the aimed player (" + FORK + " PartyFavors ItemId 68910); cooldown (R)"),
    (68911, C3, burst(15828, SPRAY, True), FIREWORK,
     "item text 'Use this to spray silly string all over your favorite things!'; effect 15828 yellow silly string at the aimed player (" + FORK + " PartyFavors ItemId 68911); cooldown (R)"),
    (77039, C3, burst(5021, 3_000), 3_600_000,
     "item text 'These stinky bombs cause opponents to run away from you in fear.'; fandom wiki Stink Bombs 'a cool down period of one hour'; stink cloud 5021 on the user for 3 s (R) and every enemy within 10 m runs for 6 s (R, " + FORK + " ability 269)"),
    (77043, C3, transform(75, 60 * MIN), None,
     "item text 'This mask is so fun, it's scary!'; fanbyte Masher_Mask 'Illusion: Forest Giant Boss - 1 hour', unlimited use; model 75 forestgiantboss.adr"),
    (910206, C3, burst(16517, FIREWORK), FIREWORK,
     "item text 'It's a firework extravaganza!'; its ability 345 is the Birthday Firework's: effect 16517 for 10 s (" + FORK + " ability 345)"),
    (910224, C3, restore(energy=50, cast=16325), 30_000,
     "item text 'Sure to brighten up your day.'; its ability 1208 is the Mana Potion's: 50% energy at once, cast 16325 (" + FORK + " ability 1208)"),
    (911312, C3, transform(1386, 15 * MIN), None,
     "client name 'Illusion: Cloudy Meatball  - 15 minutes' (locale 147628); model 1386 cloudy_meatball_01.adr"),
    (911313, C3, transform(1385, 30 * MIN), None,
     "client name 'Illusion: Meatball - 30 minutes' (locale 147629); model 1385 meatball.adr (the Meatball Costume's, " + FORK + " ability 4132)"),
    (38822, C3, transform(1521, 30 * MIN), None,
     "item text 'Drink this bottle of juice to turn into a Coyote.' (inventory.json row, ability 653); model 1521 wolf_coyote.adr for 30 minutes (" + FORK + " ability 653)"),
    (38823, C3, transform(1516, 30 * MIN), None,
     "item text 'Drink this bottle of juice to turn into a Road Runner.' (inventory.json row, ability 654); model 1516 birds_roadrunner.adr for 30 minutes (" + FORK + " ability 654)"),
    (38827, C3, transform(1692, 30 * MIN), None,
     "item text 'Looks like something a Vulture would cough up.' (inventory.json row, ability 658, no fork row, no wiki page); model 1692 skeleton_m_01.adr for 30 minutes (R: its siblings 38822 and 38823 turn you into their namesake, so Dusty Bones a skeleton)"),
]

# C3: a scare on a row, keyed by item id (the Scare Orb's L092 shape).
SCARES = {77039: {"radius": 10.0, "flee_ms": 6_000}}

# C3: placed props, as PLACED (the sixth field, when present, is the item's recast in ms).
C3_PLACED = [
    (3007, C3, 2115, [(transform_outcome(2121), 30 * MIN)],
     "item text 'If it be cake you want, you've come to the right place.'; prop 2115 mkt_cakes_pirate_01.adr; a click turns the clicker into a pirate skeleton 2121 skeleton_m_pirate.adr for 30 minutes (R: 'cursed booty', the regional birthday cakes' length)"),
    (4829, C3, 1836, [(aura_outcome(16517), 10_000), (aura_outcome(16516), 10_000), (aura_outcome(16441), 10_000)],
     "item text 'It's full of birthday party supplies! Place this crate on the ground to get the party started.'; prop 1836 hsg_crate_01.adr and its fireworks 16517, 16516 and 16441 (Sanctuary-BubbaJoeX Consumables.json Cakes ItemId 4829); a click sets one off on the clicker for 10 s (R); the balloons it hands out are not wired"),
    (9296, C3, 1066, [(transform_outcome(1667), 15 * MIN)],
     "item text 'Look out!  Once sand gets in everything, people start to get crabby!'; prop 1066 mkt_sandcastle_01.adr; a click turns the clicker into a crab 1667 crab_m_basic.adr for 15 minutes (R: the Crustay Ocean Cake's crab and length)"),
    (57010, C3, 1066, [(transform_outcome(1667), 15 * MIN)],
     "item text 'Look out!  Once sand gets in everything, people start to get crabby!'; prop 1066 mkt_sandcastle_01.adr; a click turns the clicker into a crab 1667 crab_m_basic.adr for 15 minutes (R: the Crustay Ocean Cake's crab and length)"),
    (15915, C3, 1836, [(aura_outcome(e), 10_000) for e in (15656, 15657, 15658, 15661, 15664, 15717, 15727, 15729)],
     "item text 'Dazzle your friends and wake your neighbors with this box of explosive fun!'; prop 1836 hsg_crate_01.adr; Sanctuary-BubbaJoeX Consumables.json Cakes ItemId 15915 sets off 15656, 15657, 15658, 15661, 15664, 15717, 15727, 15729 and 15731; a click sets one of the first eight off on the clicker for 10 s (R: eight outcomes is the table's cap)"),
    (57013, C3, 1836, [(aura_outcome(e), 10_000) for e in (15656, 15657, 15658, 15661, 15664, 15717, 15727, 15729)],
     "item text 'Dazzle your friends and wake your neighbors with this box of explosive fun!'; prop 1836 hsg_crate_01.adr; Sanctuary-BubbaJoeX Consumables.json Cakes ItemId 57013 sets off 15656, 15657, 15658, 15661, 15664, 15717, 15727, 15729 and 15731; a click sets one of the first eight off on the clicker for 10 s (R: eight outcomes is the table's cap)"),
    (48100, C3, 897, [(aura_outcome(5733), MIN), (aura_outcome(16), MIN), (transform_outcome(208), MIN), (transform_outcome(145), MIN)],
     "item text 'A large cake full of hidden surprises. [This cake can only be used once per hour. Click on the cake to take a slice!]'; prop 897 (Sanctuary-BubbaJoeX Consumables.json Cakes ItemId 48100, GiftCake sg_pretty_cake); a click gives one of the slice's surprises at once for 1 minute (fandom wiki Slice of Mystery Cake: an orange aura, a foggy black mist, a female changeling with a frying pan, bigger, or a cow): 5733 WFX_glow_orange_skel_loop and 16 PFX_smoke_black-yellow_skel_loop (R), changeling cook 208 and cow 145 (Sanctuary OSFR abilities 900120 and 900121); 'bigger' is not an outcome kind, and the Slice item is not handed out (R); recast one hour from the text",
     3_600_000),
    (67888, C3, 1646, [(aura_outcome(16441), 10_000), (aura_outcome(16417), 10_000)],
     "item text 'Break out the confetti and silly string! It's time to party!'; prop 1646 5m_crate_01 and its confetti 16441 and 16417 (Sanctuary-BubbaJoeX Consumables.json Cakes ItemId 67888); a click sets one off on the clicker for 10 s (R); the silly string and poppers it hands out are not wired"),
    (68932, C3, 1646, [(aura_outcome(16441), 10_000), (aura_outcome(16417), 10_000)],
     "item text 'Break out the confetti and silly string! It's time to party!'; prop 1646 5m_crate_01 and its confetti 16441 and 16417 (Sanctuary-BubbaJoeX Consumables.json Cakes ItemId 68932); a click sets one off on the clicker for 10 s (R); the silly string and poppers it hands out are not wired"),
    (69819, C3, 1724, [(transform_outcome(1702), 30 * MIN)],
     "item text 'It's just a cake... don't be scared!  Place this cake on the ground and it'll surely spook your friends!'; prop 1724 (Sanctuary-BubbaJoeX Consumables.json Cakes ItemId 69819, ScaredyCake); a click turns the clicker into the Pumpkin Prince 1702 pumpkinprince_m_boss.adr for 30 minutes (its TransformAbilityIds 4372); its scare effects are not wired"),
    (69820, C3, 1750, [(transform_outcome(1692), 3 * MIN)],
     "item text 'Place this headstone on the ground and click on it to turn into a skeleton.'; prop 1750 mkt_gravestone_01.adr; skeleton 1692 skeleton_m_01.adr for 3 minutes (R: the Skeleton treat 69845's 'skeleton for 3 minutes')"),
    (69828, C3, 1722, [(transform_outcome(1689), 30 * MIN)],
     "item text 'Place this pumpkin on the ground and click on it to receive a smaller one'; prop 1722 evnt_halloween_jackolantern_01.adr; a click turns the clicker into a jack-o-lantern 1689 jackolantern_m_01.adr for 30 minutes (R: the Jack-o-Lantern treat 69818's look; the Mini Jack-O-Lantern item is not handed out)"),
    (69842, C3, 1725, [(transform_outcome(3305), 30 * MIN), (aura_outcome(15909), 30 * MIN), (transform_outcome(198), 30 * MIN)],
     "item text 'Place this cauldron on the ground and click on it to receive a spooky treat.'; prop 1725 evnt_halloween_cauldron_01.adr; a click gives one spooky cupcake's effect at once: bat 3305, ghosts 15909 or raven 198 for 30 minutes (R: B1's 69839-69841 rows; the treat item is not handed out)"),
    (76642, C3, 1971, [(aura_outcome(16413), 10_000), (aura_outcome(16380), 10_000), (aura_outcome(16381), 10_000)],
     "item text 'Stockings come stuffed with holiday fireworks!'; prop 1971 mkt_holiday_fireplace_01.adr; a click sets off one holiday firework on the clicker for 10 s: 16413, 16380 or 16381 (R: B1's 2011 firework effects; the firework items are not handed out)"),
]

# Audited elements with no row, and why. Keyed like ROWS.
LEFT = {
    69829: "Ectoplasmic Goo is not usable: its only use is trading 15 for the Goo'd Candy minigame (fanbyte).",
    76556: "Oversized Candy Cane is a weapon (category 107), not a consumable.",
    76643: "Snowball Creation Unit is placed and hands out Snowballs, which have no item row (B3); needs an item-grant outcome.",
    4828: "Balloon Pet Pumper gives a balloon pet: needs a trial-pet outcome (pets).",
    8157: "Balloon Pet Pumper gives a balloon pet: needs a trial-pet outcome (pets).",
    9114: "Ghost Tails Campfire is placed and gives a trial ghost pet (fanbyte): needs a trial-pet outcome (pets).",
    "Crispy Croissant": "No client item or locale name is Crispy Croissant; GC must name its item id.",
    # C3
    2112: "Bravery Biscuit summons a battle pet for 1 minute: needs a combat ally in the encounter engine (battle owner).",
    10352: "Creepy Construct Caller calls a battle companion: needs a combat ally in the encounter engine (battle owner).",
    2687: "Robgoblin Treasure Trove rolls a loot table (the fork's 'treasure-trove'): containers are QF's mystery_chests.json.",
    48154: "Doll Sizing Scroll changes the head size: the client's 35/30 PlayerUpdatePacketHeadInflationScale (ctor 008b4a80, three dwords at +0x10..+0x18) has no decoded reader, and no source gives the size.",
    55831: "Gold Badge (unlimited use) shows a badge: no client effect, model or text names what it draws (fanbyte: 'Activatable Effects: Gold Badge' only).",
    68849: "Snowhill Protector Badge: as the Gold Badge, nothing names what it draws.",
    65831: "Spaghetti with Meatballs Recipe teaches a Chef recipe (fanbyte: a cooking minigame): recipes.json has no row for it (cooking owner).",
    65833: "Honey Milk Sundae Recipe teaches a Chef recipe (fanbyte: a cooking minigame): recipes.json has no row for it (cooking owner).",
    10482: "Battle Item Mystery Pack is a container (wiki: three Medium Energy and three Medium Health Potions): containers are QF's mystery_chests.json.",
    78137: "Mystery Key is spent by opening a Mystery Chest ('Click Use from the Mystery Chest you want to unlock'), mystery_chest.rs; it has no use of its own.",
    21824: "Royal Stew Du Jour: the wiki gives 'increases movement speed by 50%'; there is no consumable speed buff yet (the ride owns stat 2, mounts.rs).",
    21813: "Crispy Choychoy Delight: no wiki page; its sibling Royal Stew Du Jour (ability 2654 next to its 2653) raises movement speed by 50% (R), and there is no consumable speed buff yet.",
    76641: "Jingle Bell Boombox 76641 is a Class 9 twin of 878 (same name and ability 5087): the boombox catalogue is pinned to the client's Class 171 items (tests/boombox_catalogue.rs), its owner's call.",
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
    # C3: then the captured inventory's own definitions (a few consumables only it holds).
    path = Path(directory) / "inventory.json"
    if path.exists():
        for row in load_json(path)["definitions"]:
            if row["id"] not in items:
                items[row["id"]] = inventory_definition(bytes.fromhex(row["serialized_hex"]))
                INVENTORY_ONLY.add(row["id"])
    return items


INVENTORY_ONLY = set()


def inventory_definition(record):
    """(NameId, DescriptionId, IconId, ActivatableAbilityId, SingleUse) from a serialized item
    definition: eleven words, NoTrade and NoSale, the model and texture strings, three words,
    MembersOnly and NonMinigame, seven effect words (the sixth is the ability), then SingleUse
    (`restoration/piece_096.rs`, the client's `008e3760`)."""
    word = lambda at: int.from_bytes(record[at:at + 4], "little", signed=True)
    at = 46
    for _ in range(2):
        at += 4 + word(at)
    at += 12 + 2
    return (word(4), word(8), word(12), word(at + 20), bool(record[at + 28]))


def names(locale_path):
    if not locale_path or not Path(locale_path).exists():
        return {}
    return {row["id"]: row["text"] for row in load_json(locale_path)}


def resolve(key, items, locale, b3):
    """The item id behind a ROWS/LEFT key, or None while it has no item row."""
    if isinstance(key, int):
        return key if key in items else None
    if key in b3:
        found = b3[key] in items and b3[key] not in INVENTORY_ONLY
        return b3[key] if found else None
    wanted = key.lower()
    # Names resolve in the item tables only: the captured inventory holds other items of the
    # same name (its Snowball 17179 is the snowball fight's, not the treat B1 means).
    hits = sorted(i for i, row in items.items()
                  if i not in INVENTORY_ONLY and locale.get(row[0], "").lower() == wanted)
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
    tag = event if event.startswith("C3") else f"B1 {event}"
    source = (
        f"{tag}: {evidence}. Client item table: Id {item_id} NameId {name_id} "
        f"DescriptionId {description_id} ActivatableAbilityId {ability} SingleUse "
        f"{'true' if single_use else 'false'}; Sanctuary-main@{SANCTUARY_REV} Consumables.json "
        f"{sanctuary_note(sanctuary, item_id, ability)}; {cross}. Recast {cooldown} ms"
        f"{'' if cooldown != duration else ', the effect length'} (R)."
    )
    if not description_id:
        source += " DescriptionId 0: the confirm window's body is the item's name (R)."
    return {
        "definition": item_id,
        "name_id": name_id,
        "description_id": description_id or name_id,
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
    for key, event, effect, cooldown, evidence in ROWS + C3_ROWS:
        item_id = resolve(key, items, locale, b3)
        if item_id is None:
            pending.append(key)
            continue
        if item_id in rows:
            raise SystemExit(f"B1: item {item_id} has two rows")
        rows[item_id] = build_row(item_id, event, effect, cooldown, evidence, items, legacy, sanctuary)
        if item_id in SCARES:
            rows[item_id]["scare"] = SCARES[item_id]
    props = {}
    for key, event, model, outcomes, evidence, *recast in PLACED + C3_PLACED:
        item_id = resolve(key, items, locale, b3)
        if item_id is None:
            pending.append(key)
            continue
        props[item_id], rows[item_id] = build_prop(
            item_id, event, model, outcomes, evidence, items, legacy, sanctuary, *recast)
    return rows, props, pending, items, locale, b3


def build_prop(item_id, event, model, outcomes, evidence, items, legacy, sanctuary, recast=60_000):
    """The placed_cakes.json row and its consumables.json placed_cake row. Lifetime, click
    window, range, offset and (unless the row names its own) recast are the regional birthday
    cakes' (Sanctuary defaults, R)."""
    row = build_row(item_id, event, {"placed_cake": {"row": item_id}}, recast, evidence, items, legacy, sanctuary)
    row["single_use"] = False
    tag = event if event.startswith("C3") else f"B1 {event}"
    prop = {
        "item_id": item_id,
        "name_id": row["name_id"],
        "model_id": model,
        "cursor_id": 5,
        "animation_id": 1,
        "spawn_effect_ids": [POOF],
        "lifetime_ms": 60_000,
        "item_cooldown_ms": recast,
        "click_window_ms": 5_000,
        "click_range_m": 5.0,
        "spawn_offset_m": 1.5,
        "icon_id": CHANGEFORM_ICON,
        "outcomes": [{"effect": effect, "duration_ms": ms} for effect, ms in outcomes],
        "source": f"{tag}: {evidence}; cursor 5 cursor_talk, spawn poof 21, the 60 s lifetime, "
                  + ("5 s click window and 60 s recast are the regional birthday cakes' (R)" if recast == 60_000
                     else "5 s click window are the regional birthday cakes' (R) and the recast the item text's"),
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
        for key in LEFT:  # an item with a reason has no row
            merged.pop(resolve(key, items, locale, b3), None)
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
    for item in left:
        if item in shipped:
            problems.append(f"item {item} has a LEFT reason and a shipped row (run --apply)")
    if Path(args.audits).exists():
        keys = {k: k for k, *_ in ROWS + PLACED + C3_ROWS + C3_PLACED}
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
