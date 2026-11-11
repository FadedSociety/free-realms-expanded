"""event_items.py: item rows for the event items the 2026-11-11 event audits found with no item row (B3).

The audits (`lab/autopilot/opus/audits/events/*.json`) list event items, housing, pets, rides, containers and
bows that no item table defines, so nothing can grant, sell or open them. This script builds one release
item-table row per item into `data/release/item-table-supplement.json` (the last table `Inventory::parse_content`
reads), plus the two bindings an item needs to work at all: a `housing_fixtures.json` row for a placeable item
and a `pets.json` `items` binding for a pet (then rerun `scripts/build_pet_rewards.py`, which lists those pets as
event or retired pets rather than earn-ladder pets). It never touches effects (consumables), shop rows or container
contents; other pieces own those and read the list this script writes (`--report`).

Evidence order per row (every inferred field is marked (R) in the report):
  1. the client: locale text for the name and description, `ImageSets.txt` for the icon (its name spells the
     model and texture alias), `Models.txt` and the asset catalogue for the model, StoreBundles MarketingItemId
     for the item id;
  2. FreeRealms-Legacy@9e08202 and Sanctuary-main@10f129e ClientItemDefinitions (AGPL-3.0-or-later data, no
     code): rows copied whole; where the two differ Sanctuary-main's value is used, as the rides piece did;
     Legacy `Param1`/`Param2` land in Unknown36/Unknown37; `Comment` (a label), `ResellValue`,
     `ClientEquipReqSetId` and `Stats.*` have no table column and are dropped (the release table has none);
  3. a sibling item of the same family for every column the evidence above does not give.
Ids: the client's own id where one is known (a Legacy row, a StoreBundles MarketingItemId, or a gap in an id run
whose neighbours are known, marked (R)); otherwise the next free id in the B3 band 911000-911999. Band entries
are numbered in SPEC order, so append new entries at the end and never reorder or delete one.

  python -I scripts/event_items.py            rewrite the three files
  python -I scripts/event_items.py --check    change nothing; exit 1 if a file would change or a row fails
  python -I scripts/event_items.py --report <path>   also write the item list (id, name, event, sources)
Reads the pinned lab copies named below; run on a machine with the lab."""
import argparse
import json
import re
import sys
from pathlib import Path

sys.dont_write_bytecode = True
ROOT = Path(__file__).resolve().parents[1]
TOOLS = ROOT / "prototype/osfr-tools"
LAB = ROOT.parent / "lab"
SUPPLEMENT = TOOLS / "data/release/item-table-supplement.json"
TABLE = TOOLS / "data/release/item-table.json"
INVENTORY = TOOLS / "data/release/inventory.json"
FIXTURES = TOOLS / "data/content/housing_fixtures.json"
PETS = TOOLS / "data/content/pets.json"
LEGACY = LAB / "references/FreeRealms-Legacy/src/Resources/ClientItemDefinitions.json"
MAIN = LAB / "references/Sanctuary-main-20261019/src/Resources/ClientItemDefinitions.json"
CLIENT = LAB / "clients/community-filled"
LOCALE_MAP = LAB / "reports/locale-id-map.json"
CATALOG = LAB / "extracted/client-assets/catalog.tsv"
BAND = (911000, 911999)
LEGACY_PIN = "FreeRealms-Legacy@9e08202"
MAIN_PIN = "Sanctuary-main@10f129e"
COLUMNS = ["NameId", "DescriptionId", "IconData.Id", "IconData.TintId", "Cost", "Class", "ProfileOverride", "Slot",
           "GenderUsage", "Type", "CategoryId", "WeaponTrailEffectId", "CompositeEffectId", "PowerRating",
           "MinProfileRank", "Rarity", "ActivatableAbilityId", "PassiveAbilityId", "MaxStackSize", "MemberDiscount",
           "Unknown36", "Unknown37", "Flags"]
FLAGS = ["NoTrade", "NoSale", "MembersOnly", "NonMiniGame", "SingleUse", "IsTintable", "ForceDisablePreview"]
# Every key a donor definition carries, and where it lands (None: no table column, dropped on purpose).
DONOR_KEYS = {"Id": "id", "NameId": "NameId", "DescriptionId": "DescriptionId", "Icon": "icon", "Cost": "Cost",
              "Class": "Class", "ProfileOverride": "ProfileOverride", "Slot": "Slot", "GenderUsage": "GenderUsage",
              "Type": "Type", "CategoryId": "CategoryId", "WeaponTrailEffectId": "WeaponTrailEffectId",
              "CompositeEffectId": "CompositeEffectId", "PowerRating": "PowerRating",
              "MinProfileRank": "MinProfileRank", "Rarity": "Rarity", "ActivatableAbilityId": "ActivatableAbilityId",
              "PassiveAbilityId": "PassiveAbilityId", "MaxStackSize": "MaxStackSize",
              "MemberDiscount": "MemberDiscount", "Param1": "Unknown36", "Param2": "Unknown37",
              "ModelName": "model", "TextureAlias": "texture", "TintAlias": "alias", "Comment": None,
              "ResellValue": None, "ClientEquipReqSetId": None, **{f: "flag" for f in FLAGS}}
SURFACE_CATEGORIES = {51, 55, 59, 60}
# Housing categories that place as a fixture (housing_fixtures.json's own rule: group 50 or item type 29).
FIXTURE_CATEGORIES = {4, 52, 53, 54, 56, 57, 147}


# ---------------------------------------------------------------------------------------------------- spec
def legacy(event, rows, *ids, retired=False, note="", for_=None):
    return [dict(kind="legacy", event=event, rows=rows, id=i, retired=retired, note=note, for_=for_) for i in ids]


def already(event, rows, *ids, retired=False, for_=None):
    """Items the release already defines in `data/release/inventory.json` (captured rows): no row, listed in the report."""
    return [dict(kind="already", event=event, rows=rows, id=i, retired=retired, for_=for_) for i in ids]


def make(event, rows, name, like, id=None, desc=None, icon=None, tint=0, model=None, tex=None, alias=None,
         set=None, flags=None, idsrc=None, note="", retired=False, pet=None, for_=None):
    return [dict(kind="make", event=event, rows=rows, id=id, name=name, like=like, desc=desc, icon=icon, tint=tint,
                 model=model, tex=tex, alias=alias, set=set or {}, flags=flags, idsrc=idsrc, note=note,
                 retired=retired, pet=pet, for_=for_)]


HW, BB, CP, FH, SP = "halloween", "birthday_bash", "childs_play", "festival_of_hearts", "saint_patricks"
GC, GIFT = "gift_catalog", "gift_catalog"   # GC's gift catalog (data/content/gift_catalog.json on its branch)
SE, SC, SD, TD = "spring_eggstravaganza", "summer_camp", "snow_days", "twelve_days"
HEAD = "L22926"         # Cat Ears: costume head piece (class 173, slot 1, category 101, no trade, no sale)
OUTFIT = "L10479"       # Robbie Outfit: full costume (slot 17, category 201)
SHIRT = 76826           # Lovely T-Shirt: event T-shirt (slot 3, category 103)
HAT = 76553             # Santa Hat
FLOOR, WALL = "L22946", "L22948"   # Halloween Flooring / Wallpaper (Type 17 surfaces)
DECOR = 76745           # Teddy Bear: older housing decoration (category 54)
DECOR13 = "L78582"      # Frankie Spooky Popper: 2013 housing decoration (Type 29, category 54)
BOX = 9099              # Mystery Gift: a bag container (Type 6); its contents belong to the container table
PETPAL = "M950014"      # Cupid (Sanctuary-main): pet pal (Type 2, category 200)
VANITY = 76613          # Ziggy: full pet (Type 2)
RIDE = "M950106"        # Arctic Cat (Sanctuary-main): ride (Type 19)
TOY = "<toy>"           # a toy figure: model hsg_toy_<first word of the icon>.adr, texture the rest
# The snowflake set's window and door pieces whose icon is shared: the client's own models per piece.
WOODFRAME = {441324: "hsg_circlewindow_3x3_snowflake.adr", 441538: "hsg_singledoor_3x3_snowflake.adr",
             441539: "hsg_doubledoor_3x3_snowflake.adr", 441323: "hsg_squarewindow_3x3_snowflake.adr"}
SPEC = [
    # ------------------------------------------------------------------ Super Spooktacular (halloween.json)
    *legacy(HW, [175], 22926), *legacy(HW, [182], 22927), *legacy(HW, [207], 10479), *legacy(HW, [225], 22931),
    *already(HW, [230], 17736), *legacy(HW, [230], 79166, note="the female cut (Legacy's id); 17736 is the male"),
    *legacy(HW, [242], 17755), *legacy(HW, [247], 10472), *legacy(HW, [248], 17752), *already(HW, [254], 22946),
    *already(HW, [255], 22948), *legacy(HW, [257], 17744), *legacy(HW, [258], 17751), *already(HW, [264], 17753),
    *already(HW, [267], 17750), *legacy(HW, [268], 17742), *legacy(HW, [360], 79138), *legacy(HW, [361], 79514),
    *legacy(HW, [365], 78579), *legacy(HW, [369], 78582), *legacy(HW, [370], 78583), *legacy(HW, [373], 78585),
    # The 2010 costume block 22922-22952 (StoreBundles MarketingItemIds; Legacy re-numbered four of them to 79xxx).
    *make(HW, [185], 8818, "L79379", id=22922, idsrc="bundle 4990", icon=7621, tint=228),
    *make(HW, [186], 8820, "L79382", id=22923, idsrc="bundle 50013", desc=8819, icon=7624, tint=228),
    *make(HW, [168], 8824, HEAD, id=22925, idsrc="bundle 50071", icon=7625, tint=228),
    *make(HW, [203], 8830, HEAD, id=22928, idsrc="bundle 50105", icon=7631, tint=228),
    *make(HW, [193], 8832, HEAD, id=22929, idsrc="run 22922 + (name - 8818) / 2 (R)", icon=7632, tint=228),
    *make(HW, [217], 8834, HEAD, id=22930, idsrc="bundle 50120", icon=7634, tint=228),
    *make(HW, [176], 8840, "L79508", id=22933, idsrc="bundle 5001", icon=7639),
    *make(HW, [183], 8842, "L79511", id=22934, idsrc="bundle 5002", icon=7640),
    *make(HW, [200], 8844, "L79391", id=22935, idsrc="run (R)", desc=8845, icon=7656, tint=228),
    *make(HW, [198], 8846, "L79394", id=22936, idsrc="run (R)", desc=8845, icon=7657, tint=228),
    *make(HW, [202], 8848, "L79397", id=22937, idsrc="run (R)", desc=8845, icon=7658, tint=228),
    *make(HW, [199], 8850, "L79400", id=22938, idsrc="run (R)", desc=8845, icon=7659, tint=228),
    *make(HW, [201], 8852, "L79403", id=22939, idsrc="run (R)", desc=8845, icon=7660, tint=228),
    *make(HW, [269], 8854, 2937, id=22940, idsrc="bundle 5008", icon=7644),
    *make(HW, [272], 8856, 15587, id=22941, idsrc="bundle 5009", icon=7645),
    *make(HW, [271], 8858, 15614, id=22942, idsrc="bundle 5010", icon=7646),
    *make(HW, [270], 8860, 3017, id=22943, idsrc="bundle 5011", icon=7647),
    *make(HW, [273], 8862, 3056, id=22944, idsrc="bundle 5012", icon=7648),
    *make(HW, [275], 8864, FLOOR, id=22945, idsrc="bundle 5013", icon=7609, tint=231),
    *make(HW, [260], 8868, FLOOR, id=22947, idsrc="run (R)", icon=7611, tint=231),
    *make(HW, [261], 8872, WALL, id=22949, idsrc="run (R)", icon=7614, tint=260),
    *make(HW, [279], 8874, WALL, id=22950, idsrc="bundle 5018", icon=7651, tint=260),
    *make(HW, [263], 8876, WALL, id=22951, idsrc="bundle 5019", icon=7585, tint=260),
    *make(HW, [169], 7374, OUTFIT, id=17203, idsrc="bundle 50073", icon=6997),
    # 2013 Halloween block 78571-78585: the ids follow the name ids (AFK Hat 78571, hair cards 78573-78578 by
    # StoreBundles, Fog Machine 78579, the Poppers 78582/78583 and White Flaming Skull 78585 by Legacy).
    *make(HW, [236], 5102495, "L79466", id=78577, idsrc="bundle 5698"),
    *make(HW, [237], 5102493, "L79465", id=78575, idsrc="bundle 5696"),
    *make(HW, [238], 5102496, "L79469", id=78578, idsrc="bundle 5699"),
    *make(HW, [239], 5102494, "L79467", id=78576, idsrc="bundle 5697"),
    *make(HW, [240], 5102492, "L79468", id=78574, idsrc="bundle 5695"),
    *make(HW, [241], 5102491, "L79470", id=78573, idsrc="bundle 5694"),
    *make(HW, [364], 5102477, OUTFIT, id=78570, idsrc="client ItemGroupMappings group 17 (male costumes); 78571 AFK Hat follows (R)",
          icon=8569, tint=228, set={"Slot": 3, "CategoryId": 103}),
    *make(HW, [367], 5102546, "L78579", id=78580, idsrc="run (R)", icon=8578, model="mkt_toxic_barrel_01.adr", tex=""),
    *make(HW, [366], 5102548, "M950105", id=78581, idsrc="run (R)", set={"Unknown36": 0},
          note="ride: mounts.json has no Undead Nightmare mount row yet"),
    *make(HW, [368], 5102560, "L18092", id=78584, idsrc="run (R)", desc=0, icon=8579),
    *make(HW, [371], 5102565, "L78585", id=78586, idsrc="run (R)", desc=5102566, icon=8580, tint=0),
    *make(HW, [372], 5102575, OUTFIT, icon=8581),
    *make(HW, [374], 5102653, SHIRT, icon=8585),
    *make(HW, [375], 5102655, SHIRT, icon=8586),
    *make(HW, [376], 5102659, SHIRT, icon=8588),
    *make(HW, [377], 5102667, SHIRT, icon=8592),
    *make(HW, [378], 5102669, DECOR13, icon=8593, model="hsg_playerstudio_creepy_cupcakes.adr"),
    *make(HW, [380], 5102671, DECOR13, id=79032, idsrc="bundle 51092", icon=8594, model="hsg_playerstudio_dollydoll.adr"),
    *make(HW, [379], 5102673, DECOR13, icon=8595, model="hsg_playerstudio_gooeycaramel_apples.adr"),
    # 2009 Halloween marketplace (names 9194-9925, 14688).
    *make(HW, [215], 9194, OUTFIT, icon=4607, tint=228, set={"Slot": 3, "CategoryId": 103, "GenderUsage": 1},
          model="freestyle_<race>_m_chest_buffbodsuperhero.adr", tex="freestyle-superhero-M"),
    *make(HW, [218], 9196, OUTFIT, icon=4587, tint=228, set={"Slot": 3, "CategoryId": 103, "GenderUsage": 1},
          model="freestyle_<race>_m_chest_bigsuitfrankensteinhead.adr", tex="freestyle-frankenstein-M"),
    *make(HW, [228], 14688, OUTFIT, icon=4600, tint=228, set={"Slot": 3, "CategoryId": 103, "GenderUsage": 2},
          model="freestyle_<race>_f_chest_girlyvampire.adr", tex="freestyle-vampire-M"),
    *make(HW, [232], 9925, HEAD, icon=3277, model="caster_<race>_<gender>_head_witchhat.adr", tex="wizard-moonstars-M"),
    *make(HW, [293], 9915, DECOR, icon=8041, model="candycorn_object.adr", tex="candycorn-object"),
    *make(HW, [314], 9923, 1205, icon=8039, model="gun_ar_ag_weapon_spectral.adr", tex="spectral", set={"ActivatableAbilityId": 0},
          note="wield item; its ghost blast is an effect (B1)"),
    *make(HW, [69, 308], 9919, BOX, desc=9922, icon=8702, note="bag container (R): the TCG bundle; its cards belong to the container table"),
    *make(HW, [66], 156855, BOX, icon=4758, note="bag container (R): the original sold it as a store bundle"),
    *make(HW, [67], 432410, BOX, icon=6934),
    *make(HW, [68], 432414, BOX, icon=6935),
    *make(HW, [246], 91949, "L17744", desc=91963, icon=6841, model="hsg_candybag_halloween_02.adr", tex="halloween-cauldronbag-L1"),
    *make(HW, [259], 91952, "L17744", desc=91963, icon=4563, model="hsg_candybag_halloween_03.adr", tex=""),
    *make(HW, [78], 5102474, VANITY, desc=0, icon=8567, model="bat_m_pinkeye.adr", tex="", pet="bat_m_pinkeye.adr",
          retired=True, for_=GIFT),
    *make(HW, [79], 5102475, "M950023", pet="toeny_m_basic.adr", retired=True, for_=GIFT),
    # ------------------------------------------------------------------------------- Birthday Bash
    *make(BB, [166], 436980, 3008, desc=436984, icon=7747),
    *make(BB, [164], 436981, 2760, desc=436985, icon=7778),
    *make(BB, [165], 436982, 2937, desc=436986, icon=7746),
    *make(BB, [168], 436983, 3056, desc=436987, icon=7795),
    *make(BB, [167], 437014, 15596, icon=7780),
    *make(BB, [169], 666, 36206, icon=7804),
    *make(BB, [170], 691, BOX, icon=6356, note="bag container (R): the furniture bundle"),
    *make(BB, [163], 696, "M950006", pet="unicorn_m_pinatapet.adr"),
    # ---------------------------------------------------------------------------- Festival of Hearts
    *make(FH, [102], 3544, 15587, icon=7735), *make(FH, [103], 3546, 2760, icon=7737),
    *make(FH, [104], 3548, 15614, icon=7744), *make(FH, [105], 3550, 15596, icon=7727),
    *make(FH, [106], 3552, 15598, icon=7741), *make(FH, [107], 3554, 15598, icon=7726),
    *make(FH, [108], 3556, 36206, icon=7728), *make(FH, [109], 3572, BOX, icon=6098),
    *make(FH, [110], 3558, 15587, icon=7736), *make(FH, [111], 3560, 2760, icon=7738),
    *make(FH, [112], 3562, 15614, icon=7745), *make(FH, [113], 3564, 15596, icon=7740),
    *make(FH, [114], 3566, 15598, icon=7742), *make(FH, [115], 3568, 15598, icon=7734),
    *make(FH, [116], 3570, 36206, icon=7743), *make(FH, [117], 3574, BOX, icon=6098),
    *make(FH, [166], 5103227, 76745, desc=0, icon=5998, model="hsg_valentines_bear_01.adr", tex="valentines-teddybear-L1", tint=241),
    *make(FH, [167], 5103228, 76823, icon=5993, model="hsg_large_arrangement_01.adr", tex="valentines-largerosefloral-L1", tint=1),
    *make(FH, [168], 5103253, 1205, icon=8875),
    # ------------------------------------------------------------------------------- St Patrick's
    *make(SP, [46], 5100527, HAT, icon=8328),
    *make(SP, [47], 5100529, SHIRT, icon=8324),
    *make(SP, [48], 5100535, SHIRT, icon=8326),
    *make(SP, [49], 5100537, 1205, icon=8329, model="misc_ar_ag_weapon_clover.adr", tex="greenclover"),
    *make(SP, [50], 5100539, DECOR13, icon=1777, model="event_stpatricks_potofgold_01.adr", tex=""),
    *make(SP, [51], 5100541, DECOR13, icon=1777, desc=5100540, model="event_stpatricks_potofgold_01.adr", tex=""),
    # -------------------------------------------------------------------------- Spring Eggstravaganza
    *make(SE, [19], 5100047, BOX, desc=5100053, icon=8332, note="kit used on the eggs (R)"),
    *make(SE, [21], 5100639, BOX, desc=5100643, icon=998), *make(SE, [22], 5100640, BOX, desc=5100643, icon=998),
    *make(SE, [23], 5100641, BOX, desc=5100643, icon=998), *make(SE, [24], 5100642, BOX, desc=5100643, icon=998),
    *make(SE, [25], 5100016, BOX, desc=5100015, icon=998, note="the client's own placeholder name 'Uber Egg Name'"),
    *make(SE, [27], 5100048, 9116, desc=5100638, icon=7167, set={"ActivatableAbilityId": 0}, note="its use is an effect (B1)"),
    *make(SE, [28], 5100050, PETPAL, desc=5100056, icon=6086, model="rabbit.adr", tex="", pet="rabbit.adr"),
    *make(SE, [29], 5100099, HEAD, icon=8426),
    *make(SE, [30], 5100051, DECOR13, desc=5100057, icon=8332, model="sg_eggbasket_01.adr", tex=""),
    *make(SE, [31], 5100052, OUTFIT, desc=5100058, icon=8330, set={"Slot": 3, "CategoryId": 103}),
    *make(SE, [32], 5100049, PETPAL, desc=5100055, icon=6086, model="rabbit_thumper.adr", tex="", pet="rabbit_thumper.adr",
          note="kind unknown (R): a house pet pal"),
    # ----------------------------------------------------------------------------------- Summer Camp
    *make(SC, [96], 72315, BOX, desc=0, icon=4155, note="bag container (R): the four Doggy Lobster pieces 11027-11030"),
    # ------------------------------------------------------------------------------------ Child's Play
    *make(CP, [6], 6856, VANITY, desc=6857, icon=6990, model="babypenguin_m.adr", tex="holidaysweater", pet="babypenguin_m.adr"),
    *make(CP, [7], 2658, VANITY, desc=2659, icon=5927, model="unicorn_f.adr", tex="", pet="unicorn_f.adr",
          note="client body unicorn_f_childsplay2011.adr has no pets.json model row; drawn as unicorn_f (R)"),
    *make(CP, [8], 9297, "L18092", icon=8135),
    # ------------------------------------------------------------------------------------- Snow Days
    *already(SD, [158, 159], 17179),
    *make(SD, [160], 2660, HEAD, icon=7708),
    *make(SD, [161], 1669, 76536, icon=96),
    *make(SD, [162], 436698, DECOR, icon=7725, model="hsg_gingerbread_playhouse.adr", tex="hsg-gingerbread-01"),
    *[e for n, ic in zip(range(30709, 30717), range(6960, 6968)) for e in make(SD, [n - 30709 + 163], n, DECOR, desc=n + 8, icon=ic)],
    *make(SD, [171], 6753, DECOR, icon=6994),
    *make(SD, [173], 441921, 1205, icon=8146, model="whip_ar_ag_weapon_xmaslights.adr", tex="xmaslights"),
    *make(SD, [195], 5102844, DECOR13, desc=5102845, icon=5327, model="hsg_xmas_tree_01.adr", tex="xmas-xmastree-L1"),
    *make(SD, [196], 5102843, HAT, desc=5102840, icon=5690, tex="freestyle-fluffy-M", tint=231),
    *make(SD, [201], 5102993, SHIRT, desc=5102995, icon=8748, set={"GenderUsage": 2}),
    *make(SD, [202], 5102994, SHIRT, desc=5102996, icon=8749, set={"GenderUsage": 2}),
    *make(SD, [207], 5102999, SHIRT, desc=5103001, icon=8750),
    *make(SD, [208], 5103000, SHIRT, desc=5103001, icon=8752, set={"Slot": 4, "CategoryId": 104}),
    *make(SD, [204], 5103002, SHIRT, desc=5103004, icon=8751),
    *make(SD, [205], 5103003, SHIRT, desc=5103004, icon=8753, set={"Slot": 4, "CategoryId": 104}),
    *make(SD, [206], 5103005, BOX, desc=5103001, icon=8750), *make(SD, [203], 5103006, BOX, desc=5103004, icon=8751),
    *make(SD, [209], 5102982, BOX, desc=5102983, icon=8747),
    *make(SD, [210], 5102976, DECOR13, desc=5102981, icon=8742, model="hsg_toy_berternie.adr", tex="berternie"),
    *make(SD, [214], 5102977, DECOR13, desc=5102981, icon=8743, model="hsg_toy_bruce.adr", tex="bruce"),
    *make(SD, [211], 5102978, DECOR13, desc=5102981, icon=8728, model="hsg_toy_trollscout.adr", tex="scout"),
    *make(SD, [212], 5102979, DECOR13, desc=5102981, icon=8744, model="hsg_toy_icetroll.adr", tex="icetroll02"),
    *make(SD, [213], 5102980, DECOR13, desc=5102981, icon=8729, model="hsg_toy_yeti.adr", tex="yeti_01"),
    *make(SD, [216], 5102838, "L10479", desc=5102840, icon=5703, tint=231, set={"Slot": 3, "CategoryId": 103, "GenderUsage": 1},
          model="freestyle_<race>_m_chest_santasuit.adr", tex="freestyle-santa-M"),
    *make(SD, [216], 5102839, "L10479", desc=5102840, icon=5680, tint=231, set={"Slot": 3, "CategoryId": 103, "GenderUsage": 2},
          model="freestyle_<race>_f_chest_girlysanta.adr", tex="freestyle-santa-M"),
    *make(SD, [217], 5102750, 70455, icon=4868),
    # ------------------------------------------------------------------------------- 12 Days of Presents
    # Bows open the day's present (Mystery Key-like, Type 1 category 3); presents and Big Presents are bag
    # containers. The client's own gift icons are 8195-8223 (a bow and a box per day, three 'uber' gifts).
    *[e for d in range(12) for e in make(TD, [36 + d], 441525 + d, "L78137", id=78248 + d, idsrc=f"bundle {5516 + d}",
                                         desc=0, icon=8195 + 2 * d, set={"CategoryId": 3})],
    *[e for d in range(12) for e in make(TD, [48 + d], 441513 + d, BOX, desc=441950 + d, icon=8196 + 2 * d)],
    *[e for d in range(12) for e in make(TD, [60 + d], 5102810 + d, "L78137", id=79063 + d, idsrc=f"bundle {5717 + d}",
                                         desc=0, icon=8195 + 2 * d, set={"CategoryId": 3})],
    *[e for d in range(12) for e in make(TD, [72 + d], 5102798 + d, BOX, desc=441950 + d, icon=8196 + 2 * d)],
    *[e for n in range(3) for e in make(TD, [84 + n], 441947 + n, BOX, desc=441944 + n, icon=8221 + n)],
    *make(TD, [87], 441211, 49, desc=441226, icon=8235, set={"Unknown36": 0}, note="flair shard; its chill is an effect (B1)"),
    *make(TD, [88], 441214, "M950010", pet="fruitcake_m_basic.adr"),
    *make(TD, [89], 441209, BOX, desc=441224, icon=7096, note="bag container (R): the 16 Snow Days building pieces"),
    *make(TD, [90], 441207, BOX, desc=441222, icon=8230), *make(TD, [91], 441208, BOX, desc=441223, icon=8231),
    *make(TD, [94], 441206, BOX, desc=441221, icon=8229),
    *make(TD, [92, 130], 441217, 76536, desc=441232, icon=5763, tint=243, tex="freestyle-snowman-M"),
    *make(TD, [93], 441216, 2919, desc=441231, icon=8141),
    *make(TD, [95], 441215, 2595, desc=441230, icon=5602, set={"ActivatableAbilityId": 0}, note="illusion mask; the Snowman illusion is an effect (B1)"),
    *make(TD, [98], 441213, "L18092", desc=441228, icon=8136),
    *make(TD, [99], 441218, HEAD, desc=441233, icon=8234),
    *make(TD, [100], 441219, VANITY, desc=441234, icon=8237, model="dog_large_stbernard.adr", tex="", pet="dog_large_stbernard.adr"),
    # The 16 pieces of the Snow Days Builders Pack follow the Brick set (Type 29, category 4) in the snowflake
    # textures the client's icons name.
    *[e for n, (name, brick, ic) in enumerate([(441332, 36909, 8182), (441325, 36873, 8157), (441318, 36334, 8156),
                                              (441326, 36878, 8158), (441328, 36889, 8168), (441329, 36894, 8179),
                                              (441330, 36899, 8180), (441331, 36904, 8181), (441541, 36354, 8193),
                                              (441327, 36883, 8159), (441333, 36929, 8194), (441540, 36349, 8192),
                                              (441324, 36364, 8183), (441538, 36339, 8147), (441539, 36344, 8147),
                                              (441323, 36359, 8183)])
      for e in make(TD, [111 + n], name, f"L{brick}", desc=0, icon=ic, tint=0, model=WOODFRAME.get(name),
                    tex="snowflake-woodframe" if name in WOODFRAME else None)],
    *make(TD, [127], 5102963, "M950027", pet="fruitcake_m_basic.adr"),
    *make(TD, [128], 5102960, 2595, desc=5102961, icon=8713, set={"ActivatableAbilityId": 0}, note="illusion mask; the Holigoblin illusion is an effect (B1)"),
    *make(TD, [129], 5102965, 2919, desc=5102966, icon=8141, tint=1),
    *already(TD, [133], 78397),
    *[e for n, (name, ic, model, tex) in enumerate([(442673, 8738, "hsg_toy_tank.adr", "rangertank"),
                                                   (442672, 8733, "hsg_toy_cybercycle.adr", "cybercycle"),
                                                   (442674, 8736, "hsg_toy_jet.adr", "jet"),
                                                   (442671, 8734, "hsg_toy_herocopter.adr", "herocopter"),
                                                   (442479, 8735, "hsg_toy_hoverring.adr", "hoverring")])
      for e in make(TD, [131 + n], name, DECOR13, desc=0, icon=ic, model=model, tex=tex,
                    note="the toy, not the ride")],
    *make(TD, [136], 442675, DECOR13, desc=442680, icon=8737, model="hsg_toy_mech.adr", tex="mech", note="the toy, not the ride"),
    *make(TD, [138], 5102967, RIDE, set={"Unknown36": 0}, note="ride: mounts.json has no Arctic Cat mount row yet"),
    *legacy(TD, [137], 79385),
    # The 2013 Snow Days toy run: names 5102850-5102955 (5102897 is their shared description) follow the client's
    # toy icons 8627-8731 one for one, in the same order (Pinkeye 5102925 is icon 8701, Yeti Mystic 5102955 8731).
    *[e for n in range(106) if 5102850 + n != 5102897
      for e in make(SD, [215], 5102850 + n, DECOR13, desc=5102897, icon=8627 + n - (5102850 + n > 5102897),
                    model=TOY)],
    # --------------------------------------- retired or one-year items (owner 2026-11-11: everything stays available)
    *already(BB, [177], 15976, retired=True, for_=GIFT),
    *make(BB, [179], 435311, DECOR, icon=7394, retired=True),
    *make(BB, [179], 435313, DECOR, icon=7393, retired=True),
    *make(BB, [180], 435357, BOX, desc=435358, icon=7388, retired=True, for_=GIFT,
          note="bag container (R): the pack's chicken items"),
    *make(BB, [180], 435322, OUTFIT, desc=435358, icon=7389, retired=True, for_=GIFT, set={"Slot": 3, "CategoryId": 103}),
    *make(BB, [180], 435320, PETPAL, desc=435358, icon=7391, retired=True, for_=GIFT, model="chicken.adr", tex="",
          pet="chicken.adr"),
    *make(FH, [132], 6882, DECOR, icon=7041, retired=True),
    *make(FH, [132], 6941, DECOR, desc=6942, icon=7042, retired=True),
    *make(FH, [134], 15549, "L22926", desc=15553, icon=3471, tint=254, model="freestyle_<race>_<gender>_head_shapedglasses.adr",
          tex="freestyle-heart-L1", retired=True),
    *make(FH, [135], 15548, SHIRT, desc=15552, icon=7046, model="", tex="", set={"Slot": 6, "CategoryId": 106},
          retired=True, note="necklace; the client has an icon (item_heartnecklace) but no worn model (R)"),
    *make(FH, [136], 10823, PETPAL, pet="fairy_m_cupid.adr", retired=True),
    *make(FH, [137], 10822, "M950015", pet="heart_critter.adr", retired=True),
    *make(HW, [385], 6632, 49, desc=0, icon=7987, set={"Unknown36": 0}, retired=True,
          note="flair shard; its haunting is an effect (B1)"),
    *make(HW, [386], 416632, "L79441", desc=0, icon=6939, set={"Unknown36": 0}, retired=True,
          note="ride: mounts.json has no Jack-O-Lantern Ride mount row yet"),
    *make(SP, [58], 442306, SHIRT, desc=442307, icon=8324, retired=True),
    *make(SP, [59], 27037, SHIRT, desc=0, icon=8327, retired=True),
    *make(SD, [227], 5102793, 1205, desc=5102794, icon=5581, model="gun_ar_ag_weapon_snowball.adr", retired=True, for_=GIFT, set={"ActivatableAbilityId": 0},
          note="December 2013 member pack item: a snowball thrower; its throw is an effect (B1)"),
    # -------------------------- retired promotional gifts the gift catalog (GC) names (owner 2026-11-11: all available)
    *already(GC, [], 14184, 77986, 22952, 27959, retired=True, for_=GIFT), *legacy(GC, [], 17925, retired=True, for_=GIFT),
    *make(GC, [], 10466, "M950012", pet="deer_bambi.adr", retired=True, for_=GIFT),
    *make(GC, [], 10468, "M950013", pet="skunk_flower.adr", retired=True, for_=GIFT),
    *make(GC, [], 435710, "M950009", pet="dog_large_dukey.adr", retired=True, for_=GIFT),
    *make(GC, [], 609, "M950024", pet="toad_walgreens.adr", retired=True, for_=GIFT),
    *make(GC, [], 21620, VANITY, desc=0, icon=5617, tex="stripes", pet="triceratops_m_basic.adr", retired=True, for_=GIFT),
    *make(GC, [], 435300, BOX, desc=435301, icon=7300, retired=True, for_=GIFT,
          note="bag container (R): the three Hoodwinked Too posters"),
    *[e for n in range(3) for e in make(GC, [], 435295 + n, DECOR, desc=435301, icon=7300 + n, retired=True, for_=GIFT)],
    *make(GC, [], 435939, 1205, desc=435946, icon=7498, model="sword_ar_ag_weapon_swordofomens.adr",
          tex="freestyle-thundercats-P", retired=True, for_=GIFT),
    *make(GC, [], 435935, 3017, desc=435940, icon=7496, model="hsg_painting_thundercats_01.adr", tex="thundercats-P",
          retired=True, for_=GIFT),
    *make(GC, [], 435936, 3017, desc=0, icon=7497, model="hsg_painting_thundercats_02.adr", tex="thundercats-P",
          retired=True, for_=GIFT),
    *make(GC, [], 435945, SHIRT, desc=435937, icon=7495, retired=True, for_=GIFT),
    *make(GC, [], 6772, SHIRT, icon=6993, retired=True, for_=GIFT),
    *[e for name, ic in ((435337, 7340), (435338, 7341), (435339, 7342), (435351, 7354), (435352, 7355))
      for e in make(GC, [], name, SHIRT, desc=435355, icon=ic, retired=True, for_=GIFT)],
    *make(GC, [], 9132, SHIRT, icon=8032, retired=True, for_=GIFT),
    *make(GC, [], 435323, RIDE, desc=435358, icon=7800, model="chicken_m_default.adr", tex="white-tips",
          set={"Unknown36": 0}, retired=True, for_=GIFT,
          note="the Spring Chicken Pack's ride (R); mounts.json has no mount row for it yet"),
    *make(GC, [], 5103033, 2919, icon=8755, retired=True, for_=GIFT),
    *make(GC, [], 5103009, HAT, icon=8754, retired=True, for_=GIFT),
    *make(GC, [], 9150, WALL, icon=85, tint=0, retired=True, for_=GIFT),
    *make(GC, [], 147628, 2595, desc=0, icon=4326, set={"ActivatableAbilityId": 0}, retired=True, for_=GIFT,
          note="illusion consumable; the Cloudy Meatball illusion is an effect (B1)"),
    *make(GC, [], 147629, 2595, desc=0, icon=4139, set={"ActivatableAbilityId": 0}, retired=True, for_=GIFT,
          note="illusion consumable; the Meatball illusion is an effect (B1)"),
]


# ---------------------------------------------------------------------------------------------- evidence
def jenkins(key):
    m = 0xFFFFFFFF

    def mix(a, b, c):
        a = (a - b - c) & m; a ^= c >> 13
        b = (b - c - a) & m; b ^= (a << 8) & m
        c = (c - a - b) & m; c ^= b >> 13
        a = (a - b - c) & m; a ^= c >> 12
        b = (b - c - a) & m; b ^= (a << 16) & m
        c = (c - a - b) & m; c ^= b >> 5
        a = (a - b - c) & m; a ^= c >> 3
        b = (b - c - a) & m; b ^= (a << 10) & m
        c = (c - a - b) & m; c ^= b >> 15
        return a, b, c

    a = b = 0x9E3779B9
    c = 0
    rest = key
    while len(rest) >= 12:
        a = (a + int.from_bytes(rest[0:4], "little")) & m
        b = (b + int.from_bytes(rest[4:8], "little")) & m
        c = (c + int.from_bytes(rest[8:12], "little")) & m
        a, b, c = mix(a, b, c)
        rest = rest[12:]
    c = (c + len(key)) & m
    for i, shift in ((10, 24), (9, 16), (8, 8)):
        if len(rest) > i:
            c = (c + (rest[i] << shift)) & m
    for i, shift in ((7, 24), (6, 16), (5, 8), (4, 0)):
        if len(rest) > i:
            b = (b + (rest[i] << shift)) & m
    for i, shift in ((3, 24), (2, 16), (1, 8), (0, 0)):
        if len(rest) > i:
            a = (a + (rest[i] << shift)) & m
    return mix(a, b, c)[2]


class Evidence:
    def __init__(self):
        rows = {}
        for line in (CLIENT / "Locale/en_us_data.dat").read_bytes().splitlines():
            p = line.split(b"\t", 2)
            if len(p) == 3 and p[0].isdigit():
                rows[int(p[0])] = p[2].decode("utf-8", "replace")
        self._locale = rows
        self.legacy = {d["Id"]: d for d in json.loads(LEGACY.read_text(encoding="utf-8-sig"))}
        # The release's captured definitions win over every table (Inventory::parse_content), so an id there needs no row.
        self.captured = {d["id"] for d in json.loads(INVENTORY.read_text(encoding="utf-8"))["definitions"]}
        self.main = {d["Id"]: d for d in json.loads(MAIN.read_text(encoding="utf-8-sig"))}
        table = lambda path: [line.split("^") for line in path.read_text(encoding="latin-1").splitlines()]
        self.images = {int(p[0]): p[1] for p in table(CLIENT / "Resources/Images/ImageSets.txt") if p[0].isdigit()}
        self.models = {}
        for p in table(CLIENT / "Resources/Models.txt"):
            if p[0].isdigit():
                self.models.setdefault(p[1].lower(), int(p[0]))
        self.assets = {p[2].lower() for p in (line.split("\t") for line in
                                               CATALOG.read_text(encoding="latin-1").splitlines()) if len(p) > 2}

    def text(self, text_id):
        return self._locale.get(jenkins(b"Global.Text.%d" % text_id), "") if text_id > 0 else ""

    def model_exists(self, model):
        m = model.lower()
        if "<race>" in m or "<descriptor>" in m:
            forms = [m.replace("<race>", r).replace("<gender>", g).replace("<descriptor>", d)
                     for r in ("human", "fairy") for g in ("m", "f") for d in ("large", "small", "medium")]
            return any(f in self.assets for f in forms)
        return m in self.models or m in self.assets


def icon_look(desc):
    """The model and texture alias an icon's image-set name spells (`item_<model parts>_<texture-alias>`)."""
    d = re.sub(r"^(icon_)?(item_)?", "", desc)
    parts = d.split("_")
    k = next((n for n, w in enumerate(parts) if "-" in w), len(parts))
    model = "_".join(parts[:k])
    for a, b in (("_race_gender_", "_<race>_<gender>_"), ("_race_f_", "_<race>_f_"), ("_race_m_", "_<race>_m_"),
                 ("pet_dog_descriptor_", "pet_dog_<descriptor>_"), ("pet_cat_descriptor_", "pet_cat_<descriptor>_")):
        model = model.replace(a, b)
    return model + ".adr", "_".join(parts[k:])


def donor_row(d, main=None):
    """The table row of a donor definition; where Sanctuary-main differs, its value wins."""
    merged = dict(d)
    if main is not None:
        merged = {**{k: v for k, v in d.items() if k in main}, **main}
    unknown = [k for k in merged if k not in DONOR_KEYS and not k.startswith("Stats.")]
    if unknown:
        raise SystemExit(f"donor {d['Id']} carries fields with no destination: {unknown}")
    icon = merged.get("Icon") or {}
    value = {c: 0 for c in COLUMNS}
    for key, dest in DONOR_KEYS.items():
        if dest in COLUMNS and merged.get(key) is not None:
            value[dest] = merged[key]
    value["IconData.Id"], value["IconData.TintId"] = icon.get("Id") or 0, icon.get("TintId") or 0
    value["Flags"] = sum(1 << i for i, f in enumerate(FLAGS)
                         if merged.get(f) or (f == "ForceDisablePreview" and merged.get("Type") == 16))
    return [d["Id"], [value[c] for c in COLUMNS], merged.get("ModelName") or "", merged.get("TextureAlias") or "",
            merged.get("TintAlias") or ""]


# ------------------------------------------------------------------------------------------------- build
def build(ev, base_rows):
    col = {c: n for n, c in enumerate(COLUMNS)}
    taken = set(base_rows)
    rows, report, problems = {}, [], []
    already = []
    next_band = BAND[0]

    def template(like):
        if isinstance(like, str) and like[0] in "LM":
            source = (ev.legacy if like[0] == "L" else ev.main)[int(like[1:])]
            main = ev.main.get(source["Id"]) if like[0] == "L" else None
            return donor_row(source, main), f"{LEGACY_PIN if like[0] == 'L' else MAIN_PIN} {source['Id']}"
        if like in rows:
            return rows[like], f"B3 row {like}"
        return base_rows[like], f"release table {like}"

    for entry in SPEC:
        inferred = []
        if entry["kind"] == "already":
            if entry["id"] not in ev.captured:
                problems.append(f"{entry['id']}: listed as captured but data/release/inventory.json lacks it")
            name = ev.text((ev.legacy.get(entry["id"]) or {}).get("NameId") or 0)
            already.append({"id": entry["id"], "name": re.sub(r"\{pl=[^}]*\}|\{[a-z]+\}", "", name),
                            "event": entry["event"], "audit_rows": entry["rows"], "retired": entry["retired"],
                            "source": ["data/release/inventory.json captured definition (no supplement row needed)"],
                            **({"for": entry["for_"]} if entry.get("for_") else {})})
            continue
        if entry["kind"] == "legacy":
            d = ev.legacy[entry["id"]]
            main = ev.main.get(entry["id"])
            row = donor_row(d, main)
            diff = sorted(k for k in set(d) | set(main or {}) if (main or {}).get(k) != d.get(k)) if main else []
            sources = [f"{LEGACY_PIN} ClientItemDefinitions {d['Id']}"
                       + (f"; Sanctuary-main differs in {', '.join(diff)} and is used" if diff else
                          (f"; identical in {MAIN_PIN}" if main else ""))]
            item_id = entry["id"]
        else:
            base, how = template(entry["like"])
            values = list(base[1])
            model, tex, alias = base[2], base[3], base[4]
            values[col["NameId"]] = entry["name"]
            desc = entry["desc"]
            if desc is None:
                # A donor row of the same name keeps its own description; otherwise the text after the name.
                desc = base[1][col["DescriptionId"]] if base[1][col["NameId"]] == entry["name"] else entry["name"] + 1
            values[col["DescriptionId"]] = desc
            look = None
            if entry["icon"] is not None:
                values[col["IconData.Id"]], values[col["IconData.TintId"]] = entry["icon"], entry["tint"]
                look = icon_look(ev.images.get(entry["icon"], ""))
            # The icon's own name spells the model and texture; a model-less family (containers, cards) stays so.
            if entry["model"] == TOY:
                head, _, tex = re.sub(r"^icon_(item_)?hsg_toy_", "", ev.images.get(entry["icon"], "")).partition("_")
                model = f"hsg_toy_{head}.adr"
            elif entry["model"] is not None:
                model = entry["model"]
                tex = look[1] if look and look[0] == model else ""
            elif base[2] and look and ev.model_exists(look[0]):
                model, tex = look
            if entry["tex"] is not None:
                tex = entry["tex"]
            if entry["alias"] is not None:
                alias = entry["alias"]
            for key, value in entry["set"].items():
                values[col[key]] = value
            if entry["flags"] is not None:
                values[col["Flags"]] = sum(1 << FLAGS.index(f) for f in entry["flags"])
            item_id = entry["id"]
            if item_id is None:
                while next_band in taken or next_band in rows:
                    next_band += 1
                item_id = next_band
                inferred.append(f"id: next free id in the B3 band {BAND[0]}-{BAND[1]} (R)")
            elif "(R)" in (entry["idsrc"] or ""):
                inferred.append(f"id: {entry['idsrc']}")
            row = [item_id, values, model, tex, alias]
            sources = [f"name {entry['name']} and description {desc}: client locale"]
            if entry["icon"] is not None:
                sources.append(f"icon {entry['icon']}: client ImageSets.txt '{ev.images.get(entry['icon'], '')}'")
            if entry["idsrc"] and "(R)" not in entry["idsrc"]:
                sources.append(f"id: StoreBundles {entry['idsrc']} MarketingItemId")
            inferred.append(f"every other column from {how} (R)")
            if entry["icon"] is not None and entry["tint"]:
                inferred.append("icon tint from the family's tint (R)")
            if entry["set"]:
                inferred.append("set " + ", ".join(f"{k}={v}" for k, v in entry["set"].items()) + " for this piece (R)")
            if entry["desc"] is None:
                inferred.append("description id: the text after the name (R)")
        if item_id in taken or item_id in rows or item_id in ev.captured:
            problems.append(f"{item_id}: id already defined")
        rows[item_id] = row
        v = row[1]
        name = ev.text(v[col["NameId"]])
        if not name:
            problems.append(f"{item_id}: name id {v[col['NameId']]} has no client text")
        # 1114 is the table's shared 'no description' text id (thousands of rows carry it).
        if v[col["DescriptionId"]] > 0 and v[col["DescriptionId"]] != 1114 and not ev.text(v[col["DescriptionId"]]):
            problems.append(f"{item_id}: description id {v[col['DescriptionId']]} has no client text")
        if v[col["IconData.Id"]] and v[col["IconData.Id"]] not in ev.images:
            problems.append(f"{item_id}: icon {v[col['IconData.Id']]} is not a client image set")
        if row[2] and not ev.model_exists(row[2]):
            problems.append(f"{item_id}: model {row[2]} is not a client model")
        if entry["kind"] == "make" and not (BAND[0] <= item_id <= BAND[1]) and entry["id"] is None:
            problems.append(f"{item_id}: outside the band")
        report.append({"id": item_id, "name": re.sub(r"\{pl=[^}]*\}|\{[a-z]+\}", "", name), "event": entry["event"],
                       "audit_rows": entry["rows"], "type": v[col["Type"]], "category": v[col["CategoryId"]],
                       "source": sources, "inferred": inferred, "retired": entry["retired"],
                       **({"for": entry["for_"]} if entry.get("for_") else {}),
                       **({"note": entry["note"]} if entry["note"] else {}),
                       **({"pet_model": entry.get("pet")} if entry.get("pet") else {})})
    report.extend(dict(item, already_defined=True) for item in already)
    return rows, report, problems


def fixture_rows(ev, rows):
    """housing_fixtures.json rows for the placeable new items (the fixture builder's own row shape)."""
    out = []
    for item_id, (_, v, model, tex, alias) in sorted(rows.items()):
        kind, category = v[COLUMNS.index("Type")], v[COLUMNS.index("CategoryId")]
        placeable = kind == 29 or (kind == 1 and category in FIXTURE_CATEGORIES)
        model_id = ev.models.get(model.lower())
        if not placeable or category in SURFACE_CATEGORIES or model_id is None:
            continue
        out.append({"fixture_definition_id": item_id, "item_definition_id": item_id, "model_id": model_id,
                    "category": str(category), "lua_call": "", "texture_alias": tex, "tint_alias": alias,
                    "default_tint_id": v[COLUMNS.index("IconData.TintId")],
                    "provenance": f"B3 scripts/event_items.py row {item_id}; client Models.txt id {model_id} (R)"})
    return out


def pet_rows(ev, rows, report, pets):
    models = {m["adr"].lower(): m["model_id"] for m in pets["models"]}
    out = []
    for item in report:
        adr = item.get("pet_model")
        if not adr:
            continue
        _, v, _, tex, alias = rows[item["id"]]
        out.append({"item_id": item["id"], "model_id": models[adr.lower()], "name_id": v[0], "name": item["name"],
                    "image_set_id": v[2] or None, "icon_tint_id": v[3] or None, "tint_alias": alias or None,
                    "texture_alias": tex or None, "members_only": False,
                    "src": f"B3 scripts/event_items.py row {item['id']} (R): pets.json model {models[adr.lower()]} {adr}"})
    return out


def dumps(document):
    return json.dumps(document, indent=2, ensure_ascii=False) + "\n"


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--check", action="store_true")
    parser.add_argument("--report", type=Path)
    args = parser.parse_args()
    ev = Evidence()
    supplement_text = SUPPLEMENT.read_text(encoding="utf-8")
    supplement = json.loads(supplement_text)
    spec_ids = {e["id"] for e in SPEC if e["id"] is not None}
    mine = {r[0] for r in supplement["rows"] if BAND[0] <= r[0] <= BAND[1] or r[0] in spec_ids}
    base = {r[0]: r for r in json.loads(TABLE.read_text(encoding="utf-8"))["rows"]}
    base.update({r[0]: r for r in supplement["rows"] if r[0] not in mine})
    rows, report, problems = build(ev, base)

    supplement["rows"] = [r for r in supplement["rows"] if r[0] not in mine] + [rows[i] for i in sorted(rows)]
    supplement["source"]["event_items"] = (
        f"B3 (2026-11-11): {len(rows)} event items the event audits found with no item row, built by "
        "scripts/event_items.py: rows copied whole from the FreeRealms-Legacy@9e08202 and Sanctuary-main@10f129e "
        "ClientItemDefinitions (AGPL-3.0-or-later data), or rebuilt from the client's own name text, icon, model "
        "and StoreBundles id with every other column from a sibling item of the same family (R). New ids are "
        f"in the B3 band {BAND[0]}-{BAND[1]}.")
    supplement["source"]["event_item_ids"] = sorted(rows)
    fixtures_text = FIXTURES.read_text(encoding="utf-8")
    fixtures = json.loads(fixtures_text)
    # A row the fixture builder already derived (Legacy items) stays; only items with no fixture row get one.
    kept = [r for r in fixtures["rows"] if not r["provenance"].startswith("B3 scripts/event_items.py")]
    placed = {r["item_definition_id"] for r in kept}
    fixtures["rows"] = sorted(kept + [r for r in fixture_rows(ev, rows) if r["item_definition_id"] not in placed],
                              key=lambda r: r["item_definition_id"])
    pets_text = PETS.read_text(encoding="utf-8")
    pets = json.loads(pets_text)
    # Only this script's own bindings are rebuilt; a Legacy binding (Thumper) stays as the pets import wrote it.
    pets["items"] = [r for r in pets["items"] if not r["src"].startswith("B3 ")] + pet_rows(ev, rows, report, pets)

    changed = [p.name for p, old, new in ((SUPPLEMENT, supplement_text, dumps(supplement)),
                                          (FIXTURES, fixtures_text, dumps(fixtures)),
                                          (PETS, pets_text, dumps(pets))) if old != new]
    for line in problems:
        print("PROBLEM", line)
    print(f"{len(rows)} rows ({sum(1 for r in rows if BAND[0] <= r <= BAND[1])} in the band, "
          f"{sum(1 for r in report if r.get('already_defined'))} already captured), "
          f"{len(fixtures['rows']) - len(kept)} fixture rows, {len(pet_rows(ev, rows, report, pets))} pet bindings; "
          f"would change: {changed or 'nothing'}")
    if args.report:
        args.report.write_text(json.dumps({"schema_version": 1, "band": list(BAND), "items": report}, indent=1,
                                          ensure_ascii=False) + "\n", encoding="utf-8")
    if args.check:
        return 1 if problems or changed else 0
    if problems:
        return 1
    for path, document in ((SUPPLEMENT, supplement), (FIXTURES, fixtures), (PETS, pets)):
        path.write_bytes(dumps(document).encode("utf-8"))
    return 0


if __name__ == "__main__":
    sys.exit(main())
