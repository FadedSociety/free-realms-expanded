"""B2 event shops: put every audited event shop row on its source shop, price and season.

The ten event audits (lead, 2026-11-11) found event stock on the wrong currency (Coin Shop items
on Station Cash, Marketplace items on coins), at the wrong price, missing, sold although it is
only a reward, and on sale all year. This script is the one place those fixes live, so it can be
run again after other builders add item rows (B3 adds the event items that have no row yet):

* `live_event`: the row is sold only while that `event_calendar.json` event runs
  (`EventCalendar::is_running`, read by `CoinShop` for the Coin Shop, every vendor and the
  Marketplace). A row tagged `None` here is year-round on purpose and loses any tag.
* currency and price: `coins` puts the row in the Coin Shop (member price R2, resale T633 =
  max(1, price // 10) unless the client NoSale flag is set), `sc` in the Station Cash
  Marketplace (no coin prices, no_sale, listed at round(sc_price * sc_price_scale)).
  `cost` means the item's client `Cost` (L2), which is the Legacy Coin Shop price.
* `reward`: the item is only a reward (a quest, a prize wheel, a chest): its purchase prices go,
  its resale stays; a ride keeps its Station Cash value as `earn_only` (never listed).
* limited stock (owner 2026-11-11): `stock_per_event` (one shared counter per event run, kept by
  the server beside the account store), `stock_group` (rows sharing that counter) and
  `per_player`.
* `name`: an entry for an item that has no row yet names the client NameId; every item-table or
  supplement row with that NameId gets the entry once it exists. Until then it is reported as
  waiting, not invented.

Every touched row gets source tag B2S and a `price_provenance` with the audit unit and the source
or the (R) reasoning. `stores.json` is patched by line (one compact row per line), the rest of the
file is kept byte for byte; `vendors.json` round-trips (indent 2, CRLF). Re-running is a no-op.

Run from the repository root:
    python -I scripts/event_shops.py [--check]
`--check` writes nothing and exits 1 when the data differs from what the table says.
"""
import json
import sys
from collections import Counter, OrderedDict
from pathlib import Path

sys.dont_write_bytecode = True
ROOT = Path(__file__).resolve().parents[1] / "prototype/osfr-tools/data"
STORES = ROOT / "content/stores.json"
VENDORS = ROOT / "content/vendors.json"
CALENDAR = ROOT / "content/event_calendar.json"
SNAPSHOT = ROOT / "content/startup/snapshot.json"
# The client's Coin Shop window lists the login 165/1 lists (the captured startup snapshot) and
# vendor windows list their stock. A Coin Shop row neither shows is bought nowhere, so a row the
# audit moves from the Marketplace to coins stays on the Marketplace until this is True (a
# season-aware Coin Shop list, B2 follow-up); rows that were not sold at all move to coins.
COIN_SHOP_SHOWS_ANY_ROW = False
KEPT_ON_MARKETPLACE = (
    "; (R) kept on the Marketplace at its price: the client's captured Coin Shop list (login "
    "165/1) does not hold it and no vendor stocks it, so a coin row would be shown nowhere; it "
    "moves to coins once the Coin Shop lists event stock"
)
ITEM_TABLES = (ROOT / "release/item-table.json", ROOT / "release/item-table-supplement.json")
TAG = "B2S"
LEGEND = (
    '    "B2S": "(R) B2S event shops (scripts/event_shops.py): the row follows the 2026-11-11 event '
    "audits: Coin Shop rows sell for coins and Marketplace rows for Station Cash at the source "
    "price, live_event limits it to its event, stock_per_event / stock_group / per_player limit "
    'it, and reward-only items are not sold; price_provenance names the audit and the source.",'
)
LEGEND_AFTER = '    "PGW": '
ROW_PREFIX = '    {"item_id":'
# Marketplace shelves the new Station Cash rows need (shelf id -> item categories it gains):
# 10 food on Goods and Services (243) beside 3 supplies; 107 hand-held toys (Be-Mine Bouquet) and
# 142 hair colour cards on Clothes & Armor (45); 51 floorings and 59 wallpapers (B3's Halloween
# rows) on Housing (116). (R): no Marketplace category table names these.
SHELF_CATEGORIES = {243: [10], 45: [107, 142], 116: [51, 59]}
NO_SALE_FLAG = 2  # item-table Flags bit 1 (columns: NoTrade, NoSale, ...)

FB_HW = "fanbyte Category:Super_Spooktacular 2013 tables"
FD_HW = "fandom: 'available during the Super Spooktacular'"
FB_SD = "fanbyte Category:Snow_Days 2011 Clothing and Goodies table"
FB_BB = "fanbyte Category:Birthday_Bash Event Goodies"
FB_FH = "fanbyte fr_item page"
FB_SC = "fanbyte Category:Summer_Camp"


def e(audit, element, ids, *, event, coins=None, sc=None, reward=False, keep=False, name=None,
      stock=None, group=None, per_player=None, boombox=False, basis=""):
    """One audited element. `ids` are item ids; `name` a client NameId for a row B3 adds."""
    return dict(audit=audit, element=element, ids=list(ids), event=event, coins=coins, sc=sc,
                reward=reward, keep=keep, name=name, stock=stock, group=group,
                per_player=per_player, boombox=boombox, basis=basis)


H, S, F, B, T, C, SU, SP, TW = ("halloween", "snow_days", "festival_of_hearts", "birthday_bash",
                                "turkey_dance", "childs_play", "summer_camp",
                                "spring_eggstravaganza", "twelve_days")
R = range

# Store rows an earlier Legacy price import (T638: the fandom price, matched by name) already
# holds for items our item tables do not define yet, by client NameId. B3's rows give them
# definitions; their wiki price stands, so these entries only tag them.
LEGACY_ROWS = {
    8818: [79379, 79380, 79381], 8826: [22926, 900044, 900045], 8828: [22927, 900046, 900047],
    8836: [22931], 8840: [79508, 79509, 79510], 8842: [79511, 79512, 79513],
    8844: [79391, 79392, 79393], 8846: [79394, 79395, 79396], 8850: [79400, 79401, 79402],
    6658: [10479], 14690: [17736, 79166, 79167, 79168, 79169, 79170], 91948: [17744],
}

TABLE = [
    # ---- Halloween (Super Spooktacular) -------------------------------------------------------
    *[e(H, n, [i], event=H, coins="cost", basis=f"{FB_HW} and fanbyte FR_Item_{n.replace(' ', '_')}: "
        "only in the Coin Shop; price is the Legacy Coin Shop Cost (L2); (R) themed event stock")
      for n, i in [("Graveyard Fence", 16840), ("Graveyard Fence Gate", 16842),
                   ("Graveyard Fence Post", 16841), ("Hollowed Out Tree Stump", 16860),
                   ("Spooky Fallen Log", 16858), ("Spooky Street Lamp", 18148),
                   ("Spooky Tree", 16859), ("Spooky Wall Lamp", 16837),
                   ("Standard Square Gravestone", 36147)]],
    *[e(H, n, [i], event=H, keep=True, basis=f"{FB_HW}: Marketplace; (R) themed event stock")
      for n, i in [("Giant Gravestone", 18128), ("Round Mausoleum", 18133),
                   ("Snake-Etched Gravestone", 16838), ("Spider-Etched Gravestone", 16839),
                   ("Square Mausoleum", 18132), ("Giant Jack-O-Lantern", 79441)]],
    *[e(H, n, [i], event=H, coins=p, basis=f"fandom {n}: Coin Shop {p}; fanbyte Pet Pals: 'only "
        "available during Super Spooktacular'")
      for n, i, p in [("Baby Black Widow (pet pal)", 17536, 9000), ("Candy Corn (pet pal)", 30589, 44999),
                      ("Ghostly Raven (pet pal)", 17514, 13500), ("Ghostly Snake (pet pal)", 17556, 13500),
                      ("Ghostly Toad (pet pal)", 17522, 13500), ("Grave Elemental (pet pal)", 17516, 22500),
                      ("Jack-O-Lantern (pet pal)", 10469, 9000), ("Mini Necrowart Zombie (pet pal)", 17517, 22500),
                      ("Skeleton (pet pal)", 17520, 45000)]],
    e(H, "Bat (pet)", [7882, 17842], event=None, sc=349, basis="fandom Bat: SC Marketplace 349; "
      "fanbyte Special Pets: only the black colouring (Team Vampire Bundle) is event-bound, so the "
      "regular colours are year-round"),
    e(H, "Muggy (pet)", [69824], event=H, keep=True, basis="fanbyte Special Pets: 'Muggy is only "
      "available during Super Spooktacular'"),
    *[e(H, n, [i], event=H, keep=True, basis=f"fandom Rides: '{n}' only available during the "
        "Super Spooktacular")
      for n, i in [("Broomstick", 79433), ("Dragonfire Broomstick", 79434), ("Eyeball", 79439),
                   ("Haunted Hoverboard", 10424)]],
    e(H, "Skeleton Outfit", R(69158, 69161), event=H, sc=199, basis=f"fandom Skeleton Outfit: SC "
      f"Marketplace 199, {FD_HW}"),
    *[e(H, n, ids, event=H, sc=199, basis=f"{FB_HW}: Marketplace; (R) 199, the price of the sibling "
        "Halloween outfits (Mummy, Grim Reaper, Ghost, Vampire Count); (R) themed event stock")
      for n, ids in [("Gothic Rag Doll Outfit", R(69302, 69305)), ("The Monster's Bride Outfit", [69830]),
                     ("Vampire Countess Outfit", R(69337, 69340)), ("Witch Outfit", R(69372, 69375))]],
    e(H, "T-Bone Steak Outfit", [69836], event=H, sc=199, basis="fandom T-Bone Steak Outfit: SC "
      "Marketplace 199, limited time"),
    e(H, "Yeti Mask", [57907], event=None, sc=149, basis="fandom Yeti Mask: SC Marketplace 149 "
      "(sold Feb 2014, so not Halloween-only); year-round"),
    *[e(H, n, [i], event=H, sc=150, basis=f"{FB_HW}: Marketplace; (R) 150, the wiki price of the "
        "seasonal flair shards (Holiday Lights, Ornamental); (R) themed event stock")
      for n, i in [("Creepy Crawly Flair Shard", 2099), ("Super Spooky Flair Shard", 2100)]],
    e(H, "Spooky Speedster", [69827], event=H, keep=True, basis=f"{FB_HW}: Marketplace; the kart "
      "stays at the speedway mechanics for coins (P1079 sells every buyable vehicle there); (R) "
      "themed event stock"),
    *[e(H, n, [i], event=H, sc=199, basis=f"{FB_HW}: Marketplace; (R) 199 like the Toxic Barrel, "
        "another placeable that hands out event treats (fandom 199)")
      for n, i in [("Easy Bake Cauldron", 69842), ("Haunted Headstone", 69820), ("Jack-O-Lantern", 69828)]],
    e(H, "Scaredy Cake", [69819], event=H, coins=4999, basis=f"{FB_HW}: Coin Shop; (R) 4999, the "
      "Coin Shop price of the other Halloween placeables (fandom Frankie and Ghostly Spooky Popper)"),
    *[e(H, n, [i], event=H, sc=149, basis=f"fanbyte fr_item:{n.replace(' ', '_')}: one shirt is "
        "the Team bundle's, the other a team quest reward, neither a 1000-coin Coin Shop row; (R) "
        "listed for 149 SC (the 2013 Halloween T-shirts' fandom price) until the bundle exists")
      for n, i in [("Team Vampire T-Shirt", 17738), ("Team Werewolf T-Shirt", 17740)]],
    e(H, "Halloween face paint cards", [36042, 36043, *R(69549, 69555), *R(69556, 69561)], event=H,
      keep=True, basis=f"{FB_HW}: Quest Reward (Mystery Gift prizes); (R) their 180-coin rows stay "
      "on sale, during Halloween only, so the cards stay obtainable until the Mystery Gift table "
      "(QF) gives them"),
    e(H, "Mystery Gift costumes (Crofti)", list(R(69107, 69119)), event=H, keep=True,
      basis="fandom Chatty Costume: 'available during the Super Spooktacular event'; Crofti's (R) "
      "50-coin stock of the twelve Mystery Gift costumes is sold only during the event"),
    *[e(H, n, ids, event=H, sc=p, basis=f"{FB_HW}: 2011 Trick or Treat prize, 'only available in "
        f"the Marketplace' (fanbyte item page); (R) {p} SC, the client Cost (basis C); retired "
        "content is sold again (owner 2026-11-11)")
      for n, ids, p in [("Graveyard Flambe", [2974], 180), ("Noxious Pork Chops", [2973], 180),
                        ("Prospector's Scroll", [76710], 300)]],
    e(H, "LIMITED EDITION Werewolf Outfit", R(69193, 69196), event=H, sc=499, stock=200,
      group="limited_edition_werewolf_outfit", basis="fanbyte/fandom LIMITED EDITION Werewolf "
      "Outfit: SC Marketplace, Super Spooktacular 2009 only, first 1,000 purchases; owner "
      "2026-11-11: sold every Halloween, 200 per event across all its rows; (R) 499 SC, the "
      "regular Werewolf Outfit's fandom price (no source gives this one's)"),
    e(S, "Penguin Outfit", [69072], event=S, keep=True, basis="fandom Penguin Outfit: SC "
      "Marketplace 99 during Snow Days (moved there from the Spooktacular in 2010)"),
    # Halloween rows B3 adds (client NameIds from the audit)
    *[e(H, n, LEGACY_ROWS.get(nid, []), event=ev, sc=p, name=nid, keep=nid in LEGACY_ROWS,
        basis=f"{FB_HW}: Marketplace{src}")
      for n, nid, p, ev, src in [
          ("Bat Mask", 8824, 99, H, "; (R) 99, the fandom price of the Witch Mask"),
          ("Bowling Pin Outfit", 7374, 199, None, "; (R) 199; a year-round costume"),
          ("Cat Ears", 8826, 49, None, "; (R) 49; year-round"), ("Cat Tail", 8840, 49, None, "; (R) 49; year-round"),
          ("Dog Ears", 8828, 49, None, "; (R) 49; year-round"), ("Dog Tail", 8842, 49, None, "; (R) 49; year-round"),
          ("Fluffy Boots", 8818, 49, None, "; fandom 49; year-round"),
          ("Fluffy Gloves", 8820, 49, None, "; fandom 49; year-round"),
          ("Jack-O-Lantern Mask", 8832, 199, H, "; fandom 199, during the Spooktacular"),
          ("Necrowart Feet Wraps", 8846, 49, H, "; fandom: during the Spooktacular; (R) 49"),
          ("Necrowart Pants", 8850, 99, H, "; fandom 99, during the Spooktacular"),
          ("Necrowart Shirt", 8844, 149, H, "; fandom 149, during the Spooktacular"),
          ("Necrowart Skirt", 8852, 99, H, "; (R) 99 like the pants"),
          ("Necrowart Wrist Wraps", 8848, 49, H, "; fandom: during the Spooktacular; (R) 49"),
          ("Oversized Ears", 8830, 99, None, "; fandom 99; year-round"),
          ("Robbie Outfit", 6658, 199, None, "; (R) 199; year-round"),
          ("Super Hero Outfit", 9194, 199, None, "; (R) 199; year-round"),
          ("T-Rex Mask", 8834, 99, None, "; (R) 99; year-round"),
          ("The Creature Outfit", 9196, 199, H, "; fandom: during the Spooktacular; (R) 199"),
          ("Unicorn Hat", 8836, 99, None, "; (R) 99; year-round"),
          ("Vampire Outfit", 14688, 499, H, "; fandom 499, during the Spooktacular"),
          ("Werewolf Outfit", 14690, 499, H, "; fandom 499, during the Spooktacular"),
          ("Witch Mask", 9925, 99, H, "; fandom 99, during the Spooktacular"),
          ("Bubblegum Hair Color Card", 5102495, 99, H, "; (R) 99"),
          ("Ectoplasm Hair Color Card", 5102493, 99, H, "; (R) 99"),
          ("Grape Frosting Hair Color Card", 5102496, 99, H, "; (R) 99"),
          ("Phantasm Blue Hair Color Card", 5102494, 99, H, "; (R) 99"),
          ("Pumpkin Hair Color Card", 5102492, 99, H, "; (R) 99"),
          ("Snowstorm Hair Color Card", 5102491, 99, H, "; (R) 99"),
          ("Cauldron Candy Bag", 91949, 49, H, "; fandom 49, during the Spooktacular"),
          ("Jack-O-Lantern Candy Bag", 91948, 49, H, "; fandom 49, during the Spooktacular"),
          ("Monster Candy Bag", 91952, 49, H, "; fandom 49"),
          ("Halloween Flooring", 8866, 99, H, "; (R) 99"), ("Halloween Wallpaper", 8870, 99, H, "; (R) 99"),
          ("Pumpkin Patch Flooring", 8868, 99, H, "; (R) 99"), ("Pumpkin Patch Wallpaper", 8872, 99, H, "; (R) 99"),
          ("Skulls and Bows Wallpaper", 8876, 99, H, "; (R) 99"), ("Spooky Flooring", 8864, 99, H, "; (R) 99"),
          ("Spooky Wallpaper", 8874, 99, H, "; (R) 99"),
          ("Spooky Chair", 8854, 299, H, "; (R) 299 like the Spooky Chatty Painting"),
          ("Spooky Chatty Painting", 8860, 299, H, "; fandom 299"),
          ("Spooky Coffee Table", 8858, 299, H, "; (R) 299"), ("Spooky Coffin Clock", 8856, 299, H, "; (R) 299"),
          ("Spooky Coffin Couch", 8862, 299, H, "; (R) 299"),
          ("Giant Candy Corn", 9915, 199, H, "; (R) 199"), ("Pumpkin Prince Card", 9919, 99, H, "; (R) 99"),
          ("Spectral Blaster", 9923, 199, H, "; (R) 199")]],
    *[e(H, n, [], event=H, coins=p, name=nid, basis=f"{FB_HW}: Quest Reward and Coin Shop{src}")
      for n, nid, p, src in [
          ("Big Jack-O-Lantern", 91959, 4500, "; fandom Coin Shop 4,500"),
          ("Small Jack-O-Lantern", 91957, 2500, "; fandom Coin Shop 2,500"),
          ("Coffin", 9198, "cost", "; no wiki price: the Legacy Coin Shop Cost (L2)"),
          ("Ghost Stand-Up", 91956, "cost", "; no wiki price: the Legacy Coin Shop Cost (L2)"),
          ("Jack-O-Lantern Stand-Up", 91955, "cost", "; no wiki price: the Legacy Coin Shop Cost (L2)"),
          ("Spider Web Stand-Up", 91954, "cost", "; no wiki price: the Legacy Coin Shop Cost (L2)"),
          ("Spooktacular Balloons", 91946, "cost", "; no wiki price: the Legacy Coin Shop Cost (L2)")]],

    # Halloween 2013 weekly content and bundles, priced once B3's rows exist (fandom prices).
    *[e(H, n, [], event=H, sc=p, name=nid, basis=f"fandom {n}: SC Marketplace{src}, Super "
        "Spooktacular 2013 weekly content")
      for n, nid, p, src in [
          ("Grey Werewolf Outfit", 5102477, 299, " 299"), ("Toxic Barrel", 5102546, 199, " 199"),
          ("Super Spooktacular Parachute", 5102560, 999, " 999"),
          ("Flaming Skull Wobblehead", 5102565, 99, " 99"),
          ("Headless Horseman Outfit", 5102575, 499, " 499"),
          ("Bat Wing T-Shirt", 5102653, 149, " 149"), ("Black Cat T-Shirt", 5102655, 149, " 149"),
          ("Glowing Skeleton T-Shirt", 5102659, 149, " 149"),
          ("Punky Pumpkin T-Shirt", 5102667, 149, " 149"),
          ("Creepy Crawly Cupcakes", 5102669, 49, " 49"),
          ("Gooey Caramel Candy Apples", 5102673, 49, " 49"), ("Dolly Doll", 5102671, 49, " 49")]],
    *[e(H, n, [], event=H, coins=4999, name=nid, basis=f"fandom {n}: Coin Shop 4,999 during "
        "Super Spooktacular events")
      for n, nid in [("Frankie Spooky Popper", 5102550), ("Ghostly Spooky Popper", 5102551),
                     ("White Flaming Skull Wobblehead", 5102564)]],
    e(H, "Fog Machine", [], event=H, coins=9999, name=5102511, basis="fandom Fog Machine: Coin "
      "Shop 9,999 during Super Spooktacular events"),
    e(H, "Haunted Flair Shard (2010 only)", [], event=H, sc=150, name=6632, basis=f"{FB_HW}: 2010 "
      "only; retired content is sold again (owner 2026-11-11); (R) 150 like the other event flair shards"),
    # Bundles (B3's item rows; what they open to is QF's container table).
    *[e(ev, n, [i], event=ev, sc=p, basis=f"{src}; its contents are QF's container table")
      for ev, n, i, p, src in [
          (H, "Deadman's Party Pack", 911014, 499, f"{FB_HW}: Marketplace; (R) 499"),
          (H, "Team Vampire Bundle", 911015, 599, "fanbyte fr_item:Team_Vampire_Bundle: Marketplace; (R) 599"),
          (H, "Team Werewolf Bundle", 911016, 599, "fanbyte fr_item:Team_Werewolf_Bundle: Marketplace; (R) 599"),
          (B, "Birthday Bash Furniture Bundle", 911027, 1350, "fanbyte: Marketplace only, 1350 SC"),
          (F, "Love Struck Furniture Bundle", 911036, 825, "fanbyte fr_item: Marketplace only, 825 SC"),
          (F, "Stone Hearted Furniture Bundle", 911044, 825, "fanbyte fr_item: Marketplace only, 825 SC"),
          (S, "Bow Wrapped Suit", 911091, 299, "fandom Snow Days 2013: SC 299"),
          (S, "Snowstorm Suit", 911092, 299, "fandom Snow Days 2013: SC 299"),
          (S, "2013 Snow Days Toy Collector Pack", 911093, 396, "fandom Snow Days 2013: SC 396"),
          (B, "Spring Chicken Pack", 911274, 499, "fanbyte fr_item:Spring_Chicken_Pack: Marketplace "
           "(Birthday Bash 2011); retired content is sold again (owner 2026-11-11); (R) 499")]],

    # ---- Festival of Hearts ---------------------------------------------------------------------
    *[e(F, n, ids, event=F, sc=299, basis=f"{FB_FH}: only in the Marketplace; (R) 299, the "
        "fandom price of the sibling King of Hearts costumes")
      for n, ids in [("Queen of Broken Hearts Costume", [2578, 2579]),
                     ("Queen of Hearts Costume", [76465, 76834])]],
    e(F, "Teddy Bear Costume", [76619, 76813, 76814, 76815], event=F, sc=199, basis="fandom Teddy "
      "Bear Costume: SC Marketplace 199 during Festival of Hearts events"),
    e(F, "Turkleton Treats", [77330], event=F, sc=49, basis=f"{FB_FH}: Marketplace only; (R) 49, "
      "the fandom price of the Gummy Bear treats"),
    *[e(F, n, [i], event=F, coins="cost", basis=f"{FB_FH}: only available in the Coin Shop; price "
        "is the Legacy Coin Shop Cost (L2); fandom: during Festival of Hearts events")
      for n, i in [("Be-Mine Basket", 76822), ("Be-Mine Large Floral Arrangement", 76823),
                   ("Be-Mine Small Floral Arrangement", 76824), ("Hearts Wall Sticker", 76820),
                   ("Large Heart Wall Sticker", 76618), ("Small Heart Wall Sticker", 76617),
                   ("Teddy Bear (housing)", 76816), ("Teddy Bear (housing)", 76817),
                   ("Teddy Bear (housing)", 76818)]],
    e(F, "Teddy Bear (housing, purple)", [76745], event=F, keep=True, basis=f"{FB_FH} Teddy Bear: "
      "the purple one only in the Marketplace; (R) 76745 (the odd Cost and NoSale row) is the purple one"),
    e(F, "Heart-shaped Roofing Shingles", [76783], event=F, coins="cost", basis=f"{FB_FH}: Coin "
      "Shop, only during the Festival of Hearts; price is the client Cost (L2)"),
    e(F, "Lovely T-Shirt", [76826], event=F, coins=34, basis="fandom Lovely T-Shirt: Coin Shop 34 "
      "coins during the festival (the members gate is dropped: free economy)"),
    *[e(F, n, ids, event=F, keep=True, basis=src) for n, ids, src in [
        ("King of Broken Hearts Costume", [76832, 76835], "(R) the Hearts costumes are event stock"),
        ("King of Hearts Costume", [76831, 76833], "(R) the Hearts costumes are event stock"),
        ("Stone Heart's Bear Costume", [76812], "(R) the Hearts costumes are event stock"),
        ("Lovasus, She-Rex, Snowheart (pets)", [10996, 10994, 10993], "fanbyte Special Pets: only "
         "available during Festival of Hearts"),
        ("Lovelectric Boombox", [77329], "(R) themed event stock"),
        ("Gummy Bears", [76468, 76469], "(R) themed event stock"),
        ("Wing Pattern Style Cards", [2849, 2850, 2851, 2852], "(R) themed event stock"),
        ("Festival T-shirts", [76464, 76825, 76827, 76828, 76829, 76830], "fandom Lovely T-Shirt: "
         "the festival shirts sell during Festival of Hearts events")]],
    *[e(F, n, [i], event=None, reward=True, basis=src) for n, i, src in [
        ("Candy Heart Pajama Pants", 76467, "fanbyte Softening the Stone: quest reward"),
        ("Beloved Bow", 76707, "fanbyte Stone Heart's Hideaway: Archer prize wheel"),
        ("Sweetheart Saw", 76809, "fanbyte Stone Heart's Hideaway: Medic prize wheel"),
        ("Everloving Edge", 76810, "fanbyte Stone Heart's Hideaway: Ninja prize wheel"),
        ("Heartthrob Hammer", 76708, "fanbyte Stone Heart's Hideaway: Warrior prize wheel"),
        ("Wooing Wand", 76811, "fanbyte Stone Heart's Hideaway: Wizard prize wheel"),
        ("Stone Heart's Harp", 76709, "fanbyte both Stone Heart battles: prize wheel"),
        ("Stone Heart's Bear", 76819, "fanbyte both Stone Heart battles: prize wheel"),
        ("Stone Heart's Roses", 76807, "fanbyte both Stone Heart battles: prize wheel"),
        ("Heart-Shaped Harp", 76706, "fanbyte Stone Heart's Treasure: chest prize")]],
    e(F, "Bubble Wand (2010 Limited Edition)", [1205], event=F, sc=99, basis="fanbyte Category:"
      "Festival_of_Hearts 2010: the Limited Edition Marketplace item; retired content is sold "
      "again (owner 2026-11-11); (R) 99"),
    e(F, "Be-Mine Bouquet", [76808], event=F, sc=99, basis="fanbyte fr_item:Be-Mine_Bouquet "
      "(2010): Marketplace (2011: Gifting Basket only); (R) 99"),
    *[e(F, n, [], event=F, sc=p, name=nid, basis=f"{FB_FH}: Marketplace only{src}")
      for n, nid, p, src in [
          ("Love Struck Armoire", 3544, 299, "; (R) 299 like the piano"), ("Love Struck Bed", 3546, 299, "; (R) 299"),
          ("Love Struck Coffee Table", 3548, 299, "; (R) 299"), ("Love Struck Floor Lamp", 3550, 299, "; (R) 299"),
          ("Love Struck Loveseat", 3552, 299, "; (R) 299"), ("Love Struck Piano", 3554, 299, "; fandom 299"),
          ("Love Struck Wall Lamp", 3556, 299, "; (R) 299"),
          ("Stone Hearted Armoire", 3558, 299, "; (R) 299"), ("Stone Hearted Bed", 3560, 299, "; (R) 299"),
          ("Stone Hearted Coffee Table", 3562, 299, "; (R) 299"), ("Stone Hearted Floor Lamp", 3564, 299, "; (R) 299"),
          ("Stone Hearted Hateseat", 3566, 299, "; (R) 299"), ("Stone Hearted Piano", 3568, 299, "; fandom 299"),
          ("Stone Hearted Wall Lamp", 3570, 299, "; (R) 299")]],

    # ---- Snow Days ------------------------------------------------------------------------------
    *[e(S, n, ids, event=S, sc=p, basis=f"{FB_SD}: Marketplace{src}")
      for n, ids, p, src in [
          ("Candy Cane Outfit", [76544], 199, "; (R) 199 like the Santa Claus Outfit"),
          ("Gingerbread Girl Outfit", [76543], 99, "; (R) 99, the Gingerbread Man Outfit's price"),
          ("Miss Bah Humbug Outfit", [76564], 199, "; (R) 199, the Bah Humbug Outfit's price"),
          ("Miss Santa Outfit", [76545], 199, "; (R) 199, the Santa Claus Outfit's price"),
          ("Toy Maker Outfit", [76542], 199, "; (R) 199"), ("Winter Wonderland Outfit", [76547], 199, "; (R) 199"),
          ("Santa Claus Outfit", [76548], 199, "; fandom SC 199"),
          ("Toy Soldier Outfit", [76546, 76607], 199, "; fandom SC 199"),
          ("Reindeer Antlers", [76608], 99, "; fandom SC 99"),
          ("Ornamental Flair Shard", [76576], 150, "; fandom SC 150"),
          ("Silver Sparkler", [76563], 199, "; (R) 199, the Snow Days Marketplace median"),
          ("Yule (pet)", R(76528, 76535), 249, "; fandom SC 249; fanbyte Special Pets: only during Snow Days"),
          ("Costume Tree Outfit", [76552], 199, "; (R) 199"),
          ("Fire-Works-Place", [76642], 299, "; (R) 299, an unlimited-use placeable like the boomboxes"),
          ("Snowball Creation Unit", [76643], 299, "; (R) 299")]],
    *[e(S, n, [i], event=S, coins=p, basis=f"{FB_SD}: Coin Shop{src}")
      for n, i, p in [("Garland", 2707, 200), ("Holiday Tree", 36209, 200), ("Holiday Wreath", 36208, 200),
                      ("Snowman (house item)", 225, 200)]
      for src in ["; (R) 200, the Coin Shop price of the other Snow Days decorations (no source price)"]],
    *[e(S, n, [i], event=S, coins=p, basis=f"{FB_SD}: Coin Shop{src}")
      for n, i, p, src in [
          ("Oversized Candy Cane", 76556, 13500, "; fandom Coin Shop 13,500 (members gate dropped)"),
          ("Snow Hog (pet pal)", 17557, 13500, "; fandom Coin Shop 13,500, only during Snow Days"),
          ("Snowman (pet pal)", 17521, 45000, "; fandom Coin Shop 45,000, only during Snow Days"),
          ("Stockings", 2712, "cost", "; price is the Legacy Coin Shop Cost (L2)"),
          ("Winter Holiday Light Strand", 36154, "cost", "; 'only available in the Coin Shop'; Legacy Cost (L2)"),
          ("Winter Holiday Present", 36175, "cost", "; Legacy Cost (L2)"),
          ("Winter Holiday Presents", 36176, "cost", "; 'only available in the Coin Shop'; Legacy Cost (L2)")]],
    *[e(S, n, ids, event=S, keep=True, basis=src) for n, ids, src in [
        ("Bah Humbug Hat and Outfit", [76566, 76565], "fandom: available during Snow Days events"),
        ("Gingerbread Man, Reindeer, Snowman Outfits", [76549, 76551, 76550], f"{FB_SD}: Marketplace; (R) event stock"),
        ("Garnet (pet)", [76535], "fanbyte Special Pets: Garnet only during Snow Days"),
        ("Holiday Lights Flair Shard", [76575], f"{FB_SD}: Marketplace; (R) event stock"),
        ("Jingle Bell Boombox", [878], f"{FB_SD}: Marketplace; (R) event stock"),
        ("Holiday Tree Board, Icy Hoverboard, Snow Globe, Snowball (rides)", [79460, 10383, 79440, 17179],
         "fandom Rides: only available during the Snow Days"),
        ("Snow Globe Hat (2009 Limited Edition)", [76554], "fanbyte Category:Snow_Days 2009: Limited "
         "Edition; retired content is sold again during its event (owner 2026-11-11)")]],
    *[e(S, n, [i], event=None, reward=True, basis="fanbyte Category:Snow_Days: Holiday Mystery Gift prize")
      for n, i in [("Tacky Gift Wrapped Sweater", 76538), ("Tacky Gingerbread Sweater", 76536),
                   ("Tacky Holiday Tree Sweater", 76541), ("Tacky Reindeer Sweater", 76539),
                   ("Tacky Snowman Sweater", 76540), ("Tacky Stockings Sweater", 76537)]],
    e(S, "Crafty Robgoblins wheel weapons", R(76557, 76563), event=None, reward=True,
      basis="fandom item pages: 'acquired from the prize wheel on the Crafty Robgoblins encounter'"),
    e(S, "Winterwear (Crafty Robgoblins wheel)", [*R(76577, 76587), *R(76592, 76602)], event=None, reward=True,
      basis="fanbyte Crafty Robgoblins: Winterwear Clothing Breakdown (per-job wheel prize)"),
    e(S, "Reindeer and Santa's Sleigh-mobile (rides)", [79451, 79452], event=None, reward=True,
      basis="fandom Rides: only from the Snow Days Mystery Chest (chest 910103 holds both)"),
    *[e(S, n, [], event=S, sc=p, name=nid, basis=f"{src}") for n, nid, p, src in [
        ("Toy Maker Ears", 2660, 49, f"{FB_SD}: Marketplace; fandom SC 49"),
        ("Argyle Holiday Sweater", 1669, 199, f"{FB_SD}: Marketplace; (R) 199"),
        ("Gingerbread Playhouse", 436698, 299, f"{FB_SD}: Marketplace; (R) 299"),
        *[(f"{z} Snow Globe", nid, 99, f"{FB_SD}: Marketplace; (R) 99") for z, nid in [
            ("Blackspore", 30709), ("Briarwood", 30710), ("Merry Vale", 30711), ("Sanctuary", 30712),
            ("Seaside", 30713), ("Shrouded Glade", 30714), ("Snowhill", 30715), ("Wugachug", 30716)]],
        ("Holiday Gifting Tree", 6753, 299, f"{FB_SD}: Marketplace; (R) 299"),
        ("Snowflake Crystal Dress", 5102993, 299, "fandom Snow Days 2013: SC 299"),
        ("Lacy Gift Dress", 5102994, 299, "fandom Snow Days 2013: SC 299"),
        ("Snowstorm Suit Jacket", 5103002, 299, "fandom Snow Days 2013: the suit is SC 299"),
        ("Snowstorm Suit Pants", 5103003, 299, "fandom Snow Days 2013: the suit is SC 299"),
        ("Bow Wrapped Suit Jacket", 5102999, 299, "fandom Snow Days 2013: the suit is SC 299"),
        ("Bow Wrapped Suit Pants", 5103000, 299, "fandom Snow Days 2013: the suit is SC 299"),
        ("Toy: Bert & Ernie", 5102976, 99, "fandom Snow Days 2013: Marketplace; (R) 99"),
        ("Toy: Ice Troll Scout", 5102978, 99, "fandom Snow Days 2013: Marketplace; (R) 99"),
        ("Toy: Ice Troll Guardian", 5102979, 99, "fandom Snow Days 2013: Marketplace; (R) 99"),
        ("Toy: Yeti Pounder", 5102980, 99, "fandom Snow Days 2013: Marketplace; (R) 99")]],

    # ---- Birthday Bash --------------------------------------------------------------------------
    *[e(B, n, [i], event=None, reward=True, basis=src) for n, i, src in [
        ("Birthday Party Hat", 9091, "fanbyte item page: quest reward only"),
        ("Birthday Candle", 77450, "fanbyte: Forgivable Matter reward"),
        ("Balloon Hammer (battle version)", 77445, "fanbyte: Cakenstein Attacks! prize wheel only"),
        ("Balloon Bunny Ears (quest version)", 4830, "fanbyte: Cakey Clue reward"),
        ("Balloon Hat (member-pack version)", 4831, "member-pack item: GC's claim catalog")]],
    *[e(B, n, R(a, a + 5), event=B, coins="cost", basis=f"fandom {n}: Coin Shop 1,936 coins, five "
        "colours, during the Birthday Bash (members gate dropped)")
      for n, a in [("Balloon Bow", 16340), ("Balloon Hammer", 16345), ("Balloon Saw", 16350),
                   ("Balloon Sword", 16355), ("Balloon Axe", 16360), ("Balloon Wand", 16365)]],
    *[e(B, n, R(a, a + 5), event=B, coins="cost", basis=f"{FB_BB}: Coin Shop, five colours; "
        "Legacy Cost (L2)") for n, a in [("Balloon Bunny Ears", 16330), ("Balloon Hat", 16335)]],
    e(B, "Cupcake Hat", R(16370, 16373), event=B, keep=True, basis="fandom Cupcake Hat: Coin "
      "Shop during the Birthday Bash event"),
    e(B, "Birthday T-shirts", [9092, 9093, 9094, 9095, 9096, 9097, 9098, 77456, 77457], event=B,
      keep=True, basis=f"{FB_BB}: Coin Shop and Mystery Gift; (R) event stock"),
    e(B, "Cupcake (pet pal)", R(77461, 77465), event=B, keep=True, basis="fanbyte Pet Pals: 'The "
      "Cupcake Pet Pal is only available during Birthday Bash'"),
    e(B, "Balloon Animal (pet)", [17841, *R(77465, 77471)], event=B, sc=349, basis="fandom Balloon "
      "Animal: SC Marketplace 349 during the Birthday Bash event (also from the Balloon Pet Pumper)"),
    e(B, "Cakeling (pet pal)", [77460], event=None, sc=49, basis="fandom Cakeling: SC Marketplace "
      "49; not in fanbyte's event-only pet pals, so year-round"),
    e(B, "Streamer Flair Shard", [4826], event=B, sc=250, basis="fandom Streamer Flair Shard: SC "
      "Shop 250 during the annual Birthday Bash"),
    e(B, "Balloon Flair Shard", [4827], event=B, sc=250, basis=f"{FB_BB}: Marketplace; (R) 250 like "
      "the Streamer Flair Shard"),
    e(B, "Balloon Pet Pumper", [4828], event=B, sc=700, basis="fandom Balloon Pet Pumper: 700 SC "
      "during the Birthday Bash; (R) the first of its two rows"),
    e(B, "Birthday Party Crate", [4829], event=B, sc=700, basis="fandom Birthday Crates: Marketplace "
      "700 SC"),
    e(B, "Town birthday cakes", R(7966, 7974), event=B, sc=299, basis=f"{FB_BB} and fanbyte Birthday "
      "Cake Samples: Marketplace during the event; (R) 299 (no source price); the cake sample grant "
      "on purchase is C3's"),
    *[e(B, n, [i], event=B, coins="cost", basis=f"{FB_BB}: Coin Shop; Legacy Cost (L2)")
      for n, i in [("Birthday Gift", 77451), ("Birthday Gifts", 77452),
                   ("Birthday Party Balloons 3-Pack", 77453), ("Birthday Party Balloons 4-Pack", 77454),
                   ("Birthday Party Banner", 77455)]],
    *[e(B, n, [], event=B, sc=299, name=nid, basis="fanbyte: Marketplace only (2012); (R) 299")
      for n, nid in [("Birthday Bash Bed", 436981), ("Birthday Bash Chair", 436982),
                     ("Birthday Bash Dining Table", 436980), ("Birthday Bash Floor Lamp", 437014),
                     ("Birthday Bash Sofa", 436983), ("Birthday Bash Wall Sconce", 666)]],

    # ---- Turkey Dance, Child's Play -------------------------------------------------------------
    e(T, "Thanksgiving hat, outfit and turkey rides", [41, 42, 79437, 79461, 79462, 79463], event=T,
      keep=True, basis="fandom: hat and outfit 'from mid November through late November'; Rides: "
      "'Only available during Thanksgiving'"),
    e(C, "Ziggy (pet)", [76613], event=C, keep=True, basis="fandom Ziggy: sold only during the "
      "charity drive (Dec 2009 - Jan 2010); retired content returns with its event"),

    # ---- Summer Camp (permanent per fanbyte: no gate) -------------------------------------------
    *[e(SU, n, [i], event=None, coins="cost", basis=f"{FB_SC}: Coin Shop ('sold in the Summer Camp "
        "Coin Shop', article 1498); Legacy Cost (L2); year-round (the camp is permanent)")
      for n, i in [("Camp Chatty Banner", 14299), ("Camp Robbie Banner", 14300),
                   ("Mushroom Spitfire", 14305), ("Caterpillar Spitfire", 14306), ("Nutty Spitfire", 14307),
                   ("Chatty Totem", 16039), ("Chugawug Totem", 16040), ("Troll Totem", 16041),
                   ("Man-Eating Plant Totem", 16042), ("Owl Totem", 16043), ("Robbie Totem", 16044),
                   ("Pavilion Tent", 14308)]],
    *[e(SU, n, [i], event=None, reward=True, basis="fanbyte fr_item page: Mystery Gift and prize "
        "wheel only") for n, i in [("Camp Chatty Tent", 14309), ("Camp Robbie Tent", 14310),
                                   ("Camp Chatty Flair Shard", 10328), ("Camp Robbie Flair Shard", 10329)]],
    e(SU, "Bucket of Bouncing Bait", [9295], event=None, sc=49, basis=f"{FB_SC}: Event Goodies, "
      "Marketplace only; (R) 49"),
    e(SU, "Ghost Tails Campfire", [9114], event=None, sc=50, basis=f"{FB_SC}: Event Goodies, "
      "Marketplace only; (R) 50, the client Cost (basis C)"),
    e(SU, "Marshmallow (pet pal)", R(10489, 10495), event=None, sc=49, basis=f"{FB_SC}: Event "
      "Goodies, Marketplace (article 1498); (R) 49, the Cakeling pet pal's price"),
    e(SU, "Totem Boombox", [8185], event=None, sc=299, boombox=True, basis=f"{FB_SC}: Event Goodies, "
      "Marketplace; (R) 299, the other event boomboxes' price"),
    e(SU, "Doggy Lobster Outfit", [], event=None, sc=99, name=72315, basis=f"{FB_SC}: Marketplace; (R) 99"),

    # ---- Spring Eggstravaganza, 12 Days ---------------------------------------------------------
    e(SP, "Egg Coloring Kit", [], event=SP, sc=49, name=5100047, basis="client 5100571: missed "
      "days need a bought egg coloring kit; (R) 49"),
    *[e(TW, f"12 Days Bow - Day {d}", [], event=TW, sc=49, name=441524 + d, basis="fanbyte 12 Days "
        "of Holidays 2012: 'you must then purchase the bow for SC'; (R) 49") for d in range(1, 13)],
]

# Vendor rows the audit removes: (npc_id, item_id, reason).
VENDOR_DROPS = [
    (9500000545, 9095, "birthday_bash: no source has an NPC vendor for the Sanctuary Birthday "
     "T-Shirt (it is a Coin Shop shirt); audit fix list 'drop Zobs's Sanctuary Birthday T-Shirt'"),
]


def load_items():
    items = {}
    for path in ITEM_TABLES:
        for row in json.loads(path.read_text(encoding="utf-8"))["rows"]:
            items[int(row[0])] = row
    return items


def member_price(price, discount):
    return price - price * (discount or 0) // 100


def apply(row, entry, item):
    """Return the row as the entry wants it (a new dict; key order kept, new keys appended)."""
    row = OrderedDict(row)
    ints = item[1] if item else None
    if entry["coins"] is not None:
        price = ints[4] if entry["coins"] == "cost" else entry["coins"]
        if price <= 0:
            raise SystemExit(f"{row['item_id']}: no coin price for {entry['element']}")
        no_sale = bool(ints[22] & NO_SALE_FLAG) if ints else row.get("no_sale", False)
        row["purchase_price_coins"] = price
        row["member_price_coins"] = member_price(price, row.get("member_discount_percent"))
        row["sell_price_coins"] = None if no_sale else max(1, price // 10)
        row["no_sale"] = no_sale
        for key in ("sc_price", "earn_only", "boombox", "ride_category", "category_source"):
            row.pop(key, None)
        currency, value = "coins", price
    elif entry["sc"] is not None:
        row["purchase_price_coins"] = None
        row["member_price_coins"] = None
        row["sell_price_coins"] = None
        row["no_sale"] = True
        row["sc_price"] = entry["sc"]
        row["earn_only"] = False
        if entry["boombox"]:
            row["boombox"] = True
        currency, value = "station_cash", entry["sc"]
    elif entry["reward"]:
        row["purchase_price_coins"] = None
        row["member_price_coins"] = None
        if row.get("sc_price") is not None:
            if ints and ints[9] == 19:  # a ride keeps its Station Cash value, never listed
                row["earn_only"] = True
            else:
                row.pop("sc_price", None)
                row.pop("earn_only", None)
                row.pop("boombox", None)
        currency, value = "reward", None
    else:
        currency, value = None, None
    if currency is not None:
        # The Station Cash import tags (T871, T638, T634) describe a Marketplace row: they go when
        # the row leaves the Marketplace, and stay when it is re-priced there.
        drop = (TAG,) if currency == "station_cash" else ("T871", "T638", "T634", TAG)
        tags = [t for t in row.get("source_tags", []) if t not in drop]
        row["source_tags"] = tags + [TAG]
        marked = "(R) " if "(R)" in entry["basis"] else ""
        # Earlier provenance (ride id, wiki page, Legacy coin price, coin store) is kept; only
        # the pricing words are this entry's, and a row off the Marketplace loses its SC basis.
        provenance = OrderedDict(row.get("price_provenance") or {})
        for key in ("sc_price", "sc_basis") if currency != "station_cash" else ():
            provenance.pop(key, None)
        provenance.update(kind="reconstructed", basis=f"{marked}{TAG}: {entry['basis']}",
                          audit=entry["audit"], element=entry["element"], currency=currency)
        if value is not None:
            provenance["price"] = value
            if currency == "station_cash" and not entry.get("kept_sc"):
                provenance["sc_price"] = value
                provenance["sc_basis"] = "W" if "(R)" not in entry["basis"] else "E"
        else:
            provenance.pop("price", None)
        row["price_provenance"] = provenance
        # a sc_price must stay the row's last pricing key for readability; move it after provenance
        for key in ("sc_price", "earn_only", "boombox"):
            if key in row:
                row.move_to_end(key)
    elif TAG not in row.get("source_tags", []):
        row["source_tags"] = list(row.get("source_tags", [])) + [TAG]
    for key in ("live_event", "stock_per_event", "stock_group", "per_player"):
        row.pop(key, None)
    if entry["event"] is not None:
        row["live_event"] = entry["event"]
    if entry["stock"] is not None:
        row["stock_per_event"] = entry["stock"]
    if entry["group"] is not None:
        row["stock_group"] = entry["group"]
    if entry["per_player"] is not None:
        row["per_player"] = entry["per_player"]
    return row


def new_row(item_id, item):
    ints = item[1]
    category = ints[10] or None
    return OrderedDict(item_id=item_id, category_id=category, tint_group_id=None,
                       purchase_price_coins=None, member_discount_percent=ints[19],
                       member_price_coins=None, sell_price_coins=None,
                       no_sale=bool(ints[22] & NO_SALE_FLAG), source_tags=["R1", "L4", TAG])


def dump(row):
    return json.dumps(row, separators=(",", ":"), ensure_ascii=True)


def main():
    check = "--check" in sys.argv[1:]
    items = load_items()
    by_name = {}
    for item_id, row in items.items():
        by_name.setdefault(row[1][0], []).append(item_id)
    events = {ev["key"] for ev in json.loads(CALENDAR.read_text(encoding="utf-8"))["events"]}
    coin_store = json.loads(SNAPSHOT.read_text(encoding="utf-8"))["catalogues"]["coin_store"]
    shown = {row["id"] for row in coin_store["static_items"] + coin_store["dynamic_items"]}
    shown |= {item_id for vendor in json.loads(VENDORS.read_text(encoding="utf-8"))["vendors"]
              for item_id in vendor.get("item_ids", [])}

    text = STORES.read_bytes().decode("utf-8")
    lines = text.split("\n")
    row_at = {}
    for index, line in enumerate(lines):
        if line.startswith(ROW_PREFIX):
            row_at[json.loads(line.rstrip(","))["item_id"]] = index
    store = json.loads(text)
    if len(row_at) != len(store["items"]):
        raise SystemExit("stores.json is not one compact row per line")
    rows = {item["item_id"]: item for item in store["items"]}

    wanted = {}
    waiting = []
    stats = Counter()
    for entry in TABLE:
        if entry["event"] is not None and entry["event"] not in events:
            raise SystemExit(f"{entry['element']}: unknown event {entry['event']}")
        ids = list(entry["ids"])
        if entry["name"] is not None:
            ids += [item_id for item_id in sorted(by_name.get(entry["name"], [])) if item_id not in ids]
            if not ids:
                waiting.append(entry)
                continue
        for item_id in ids:
            if item_id in wanted:
                raise SystemExit(f"{item_id} named twice ({entry['element']})")
            wanted[item_id] = entry

    changed = {}
    added = {}
    for item_id, entry in sorted(wanted.items()):
        item = items.get(item_id)
        if item_id in rows:
            before = rows[item_id]
        elif item is None:
            waiting.append(dict(entry, ids=[item_id]))
            continue
        elif entry["reward"] or (entry["keep"] and entry["sc"] is None and entry["coins"] is None):
            continue  # nothing to sell, nothing to gate
        else:
            before = new_row(item_id, item)
        unit = entry["audit"]
        use = entry
        if entry["keep"] and item_id in rows:
            # a kept row keeps its price; the entry's price is only for a row it creates
            use = dict(entry, coins=None, sc=None)
        elif (entry["coins"] is not None and not COIN_SHOP_SHOWS_ANY_ROW and item_id not in shown
              and before.get("sc_price") is not None and not before.get("earn_only")):
            use = dict(entry, coins=None, sc=before["sc_price"], kept_sc=True,
                       basis=entry["basis"] + KEPT_ON_MARKETPLACE)
            stats[(unit, "kept on SC")] += 1
        after = apply(before, use, item)
        if item_id not in rows:
            added[item_id] = after
            stats[(unit, "added")] += 1
        else:
            old_currency = ("sc" if before.get("sc_price") is not None and not before.get("earn_only")
                            else "coins" if before.get("purchase_price_coins") is not None else "none")
            new_currency = ("sc" if after.get("sc_price") is not None and not after.get("earn_only")
                            else "coins" if after.get("purchase_price_coins") is not None else "none")
            if new_currency == "none" and old_currency != "none":
                stats[(unit, "removed")] += 1
            elif old_currency != new_currency:
                stats[(unit, "currency")] += 1
            elif (before.get("purchase_price_coins"), before.get("sc_price")) != (
                    after.get("purchase_price_coins"), after.get("sc_price")):
                stats[(unit, "price")] += 1
            if dump(after) != dump(before):
                changed[item_id] = after
        if after.get("live_event"):
            stats[(unit, "gated")] += 1
        if after.get("stock_per_event"):
            stats[(unit, "limited")] += 1

    for item_id, after in changed.items():
        index = row_at[item_id]
        comma = "," if lines[index].endswith(",") else ""
        lines[index] = ROW_PREFIX[:4] + dump(after) + comma
    if added:
        # New rows go after the last row (the file is not sorted by id), in id order.
        last = max(row_at.values())
        if lines[last].endswith(","):
            raise SystemExit("the last stores.json row ends with a comma")
        new = [ROW_PREFIX[:4] + dump(after) for _, after in sorted(added.items())]
        block = [lines[last] + ","] + [line + "," for line in new[:-1]] + [new[-1]]
        lines = lines[:last] + block + lines[last + 1:]
    new_text = "\n".join(lines)
    if LEGEND[:-1] not in new_text:
        at = new_text.index("\n" + LEGEND_AFTER) + 1
        end = new_text.index("\n", at)
        line = new_text[at:end]
        if not line.endswith('",'):
            new_text = new_text[:end] + "," + new_text[end:]
            end += 1
        new_text = new_text[:end + 1] + LEGEND[:-1] + "\n" + new_text[end + 1:]
    prefix = '  "station_cash":'
    at = new_text.index("\n" + prefix) + 1
    end = new_text.index("\n", at)
    value = new_text[at + len(prefix):end].rstrip(",")
    config = json.loads(value, object_pairs_hook=OrderedDict)
    if dump(config) != value:
        raise SystemExit("station_cash does not round-trip; patch by hand")
    for shelf in config["item_shelves"]:
        extra = SHELF_CATEGORIES.get(shelf["category_id"], [])
        shelf["item_category_ids"] = sorted(set(shelf["item_category_ids"]) | set(extra))
    new_text = new_text[:at] + prefix + dump(config) + "," + new_text[end:]
    parsed = json.loads(new_text)  # the patched file must still parse, row for row
    ids = [item["item_id"] for item in parsed["items"]]
    if len(ids) != len(set(ids)) or len(ids) != len(rows) + len(added):
        raise SystemExit("stores.json rows are not unique, or a row was lost")

    vendor_text = VENDORS.read_bytes().decode("utf-8")
    vendors = json.loads(vendor_text)
    if json.dumps(vendors, indent=2, ensure_ascii=True).replace("\n", "\r\n") + "\r\n" != vendor_text:
        raise SystemExit("vendors.json does not round-trip; patch by hand")
    dropped = 0
    for npc_id, item_id, _reason in VENDOR_DROPS:
        for vendor in vendors["vendors"]:
            if vendor["npc_id"] != npc_id or item_id not in vendor.get("item_ids", []):
                continue
            vendor["item_ids"].remove(item_id)
            removed = [it for it in vendor["items"] if it["item_id"] == item_id]
            vendor["items"] = [it for it in vendor["items"] if it["item_id"] != item_id]
            coverage = vendors["coverage"]
            for it in removed:
                coverage["items_resolved"] -= 1
                if it.get("runtime_eligible"):
                    coverage["items_with_stores_price"] -= 1
                if it.get("stock_source", {}).get("kind") == "reconstructed":
                    coverage["reconstructed_stock_rows"] -= 1
            dropped += 1
    new_vendor_text = json.dumps(vendors, indent=2, ensure_ascii=True).replace("\n", "\r\n") + "\r\n"

    units = sorted({unit for unit, _ in stats})
    for unit in units:
        print(unit + ": " + ", ".join(f"{kind} {stats[(unit, kind)]}" for kind in
                                      ("currency", "price", "added", "removed", "gated", "limited",
                                       "kept on SC")))
    print(f"vendor rows dropped: {dropped}; entries waiting for an item row: {len(waiting)}")
    for unit in sorted({entry["audit"] for entry in waiting}):
        names = [entry["element"] for entry in waiting if entry["audit"] == unit]
        print(f"  waiting ({unit}, {len(names)}): " + "; ".join(names))

    stale = new_text != text or new_vendor_text != vendor_text
    if check:
        if stale:
            print(f"stale: {len(changed)} rows differ, {len(added)} rows to add, {dropped} vendor rows to drop")
            return 1
        print("event shops: data matches the table")
        return 0
    if new_text != text:
        STORES.write_bytes(new_text.encode("utf-8"))
    if new_vendor_text != vendor_text:
        VENDORS.write_bytes(new_vendor_text.encode("utf-8"))
    print(f"wrote: {len(changed)} rows changed, {len(added)} rows added, {dropped} vendor rows dropped")
    return 0


if __name__ == "__main__":
    sys.exit(main())
