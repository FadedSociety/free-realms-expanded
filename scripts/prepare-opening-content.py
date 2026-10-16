#!/usr/bin/env python3
"""Generate private opening content from pinned community/plain-text evidence.

No game text or binary assets are written into the repository. This constructs
explicitly local objective bindings; it does not recover the missing quest DB.
Other files in the destination are preserved. Existing opening.json is refused.
"""
import argparse
import hashlib
import json
import re
from pathlib import Path


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--lab', type=Path, default=Path(__file__).resolve().parents[2] / 'lab')
    parser.add_argument('--output', type=Path)
    args = parser.parse_args()
    lab = args.lab.resolve()
    output = args.output or lab / 'references/rust-opening'
    output.mkdir(parents=True, exist_ok=True)
    target = output / 'opening.json'
    if target.exists():
        raise SystemExit('Refusing to replace existing opening.json; choose a fresh output directory.')
    legacy = lab / 'references/FreeRealms-Legacy'
    paths = [lab / 'reports/locale-id-to-text.json', legacy / 'src/Resources/Npcs.json', legacy / 'src/Resources/Models.txt', legacy / 'src/Scripts/Zone/FabledRealms.lua', legacy / 'src/Sanctuary.Game/Zones/StartingZone.cs']
    locale = {r['id']: r['text'] for r in json.loads(paths[0].read_text(encoding='utf-8-sig'))}
    npcs = {r['Id']: r for r in json.loads(paths[1].read_text(encoding='utf-8-sig'))}
    models = {int(r[0]): r[1] for line in paths[2].read_text(encoding='utf-8-sig').splitlines() if line and not line.startswith('#') and len(r := line.split('^')) > 1}
    wiki_path = lab / 'references/freerealms-wiki/quest-pages.json'
    wiki = {r['title']: r for r in json.loads(wiki_path.read_text(encoding='utf-8'))}
    weapon_page = wiki['Quest:Your First Weapon']
    wolf_page = wiki['Quest:Fighting off the Pack']
    assert weapon_page['revision_id'] == 27815 and 'coin amount: 16' in weapon_page['clean_text']
    assert wolf_page['revision_id'] == 28955 and 'coin amount: 18' in wolf_page['clean_text'] and '/5' in wolf_page['clean_text']
    catalog_path = lab / 'working/OSFR.Server/OSFR Server/Customize/ClientItemDefinitions.json'
    creator_path = lab / 'working/OSFR.Client/OSFR Client/Resources/CharacterCreate/ItemDefinitions.txt'
    catalog = json.loads(catalog_path.read_text(encoding='utf-8-sig'))
    creator_lines = creator_path.read_text(encoding='utf-8-sig').splitlines()
    creator_header = creator_lines[0].lstrip('#').split('^')
    creator_items = [dict(zip(creator_header, line.split('^'))) for line in creator_lines[1:] if line and not line.startswith('#')]
    def supplied_ids(name_id):
        return sorted({r['Id'] for r in catalog if r.get('NameId') == name_id} |
                      {int(r['ID']) for r in creator_items if int(r['NAME_ID']) == name_id})
    ingredients = [{'name': locale[name_id], 'name_id': name_id, 'quantity': quantity,
                    'supplied_definition_ids': supplied_ids(name_id)}
                   for name_id, quantity in [(104, 10), (21945, 5), (931, 5), (932, 5)]]
    assert [i['supplied_definition_ids'] for i in ingredients] == [[], [2752], [], [56]]
    assert not supplied_ids(102173), 'Original hammer recovered: review the explicit substitute before regenerating.'
    paths.extend([wiki_path, catalog_path, creator_path])
    spawns = {}
    for line in paths[3].read_text(encoding='utf-8-sig').splitlines():
        m = re.search(r'spawnNpcWithGuid\((\d+),\s*(\d+),\s*([-.\d]+),\s*([-.\d]+),\s*([-.\d]+),\s*([-.\d]+)\)', line)
        if m:
            spawns[int(m[1])] = {'guid': int(m[2]), 'position': [float(m[i]) for i in [3, 4, 5]], 'heading': float(m[6])}
    actors = []

    def actor(key, definition_id, *, position=None, model=None, name_id=None, action=None, profile=None, kind='guide'):
        original = npcs.get(definition_id, {})
        spawn = spawns.get(definition_id, {})
        model_id = model or original.get('ModelId')
        if model_id not in models:
            raise ValueError(f'Model{model_id} is not in pinned Models.txt')
        nid = name_id if name_id is not None else original.get('NameId', 0)
        if nid and nid not in locale:
            raise ValueError(f'Locale ID{nid} is missing')
        guid = 0x4F50454E00000000 + key
        item = {'guid': guid, 'definition_id': definition_id, 'name_id': nid,
                'name': locale.get(nid) or f'Local opening object{key}',
                'model_id': model_id, 'model_file_name': models[model_id], 'zone_id': 0,
                'position': position or spawn['position'], 'heading': spawn.get('heading', 0.0),
                'interaction_radius': 12.0, 'kind': kind, 'action': action,
                'required_profile': profile,
                'provenance': 'Recovered model/name; local GUID and interaction binding. ' + ('Local placement.' if position else 'Pinned community spawn position.')}
        actors.append(item)
        return guid

    darkthorne = actor(1, 6445, position=[-1897.0, -39.7, 412.6])
    journal = actor(2, 920010, position=[-1901.0, -39.7, 418.0], model=797, name_id=4419617, kind='resource', action={'kind': 'interact', 'actor_guid': 0})
    actors[-1]['action']['actor_guid'] = journal
    pappy = actor(3, 3211)
    mae = actor(4, 2538)
    cleetus = actor(5, 3232)
    bartle = actor(6, 2087)
    queen = actor(7, 31331)
    well = actor(8, 6433, position=[-2113.0, -56.5, 222.0], name_id=5100023, kind='resource', action={'kind': 'interact', 'actor_guid': 0})
    actors[-1]['action']['actor_guid'] = well
    fires = []
    for key, position in [(9, [-2123.0, -56.5, 215.0]), (10, [-2133.0, -56.5, 226.0]), (11, [-2124.0, -56.5, 237.0])]:
        guid = actor(key, 2489, position=position, name_id=5100342, kind='resource', action={'kind': 'interact', 'actor_guid': 0})
        actors[-1]['action']['actor_guid'] = guid
        fires.append(guid)
    rainleaf = actor(12, 2540, profile=45, kind='resource', action={'kind': 'gather', 'resource_id': 1001}, name_id=5100033)
    gummy = actor(13, 2546, profile=45, kind='resource', action={'kind': 'gather', 'resource_id': 1002}, name_id=5100033)
    cooking = actor(14, 2537, profile=45, kind='resource', action={'kind': 'craft', 'recipe_id': 2001})
    copper = actor(15, 920011, position=[-2072.0, -55.6, 207.0], model=607, name_id=21917, profile=14, kind='resource', action={'kind': 'gather', 'resource_id': 1003})
    furnace = actor(16, 920012, position=[-2076.0, -55.6, 190.0], model=484, name_id=5100121, profile=14, kind='resource', action={'kind': 'craft', 'recipe_id': 2002})
    anvil = actor(17, 3600, position=[-2084.0, -55.6, 190.0], profile=16, kind='resource', action={'kind': 'craft', 'recipe_id': 2003})
    actor(18, 3156, kind='enemy')
    actor(19, 920001, position=[-2173.0, -31.5, 348.0], model=3933, name_id=5101961, kind='enemy')
    actor(20, 920002, position=[-1828.0, -51.3, 365.0], model=3897, name_id=5100344, kind='enemy')
    artifacts = []
    for key, position in [(21, [-1858.0, -51.3, 355.0]), (22, [-1844.0, -51.3, 343.0]), (23, [-1838.0, -51.3, 370.0])]:
        guid = actor(key, 920020 + key, position=position, model=3934, name_id=5100327, kind='resource', action={'kind': 'interact', 'actor_guid': 0})
        actors[-1]['action']['actor_guid'] = guid
        artifacts.append(guid)
    ellandra = actor(24, 2385)
    maximus = actor(25, 2338)
    scotty = actor(26, 2351)
    captain = actor(27, 2378)
    lookup = {a['guid']: a for a in actors}
    stages = []

    def goal(locale_id, kind, required=1, description=None, **parameters):
        return {'id': 0, 'name_id': locale_id, 'description_id': locale_id,
                'description': description or locale[locale_id], 'required': required,
                'trigger': {'kind': kind, **parameters},
                'recovery': 'Original action text where locale_id is nonzero; local ordered binding/count unless explicitly five copper bars. Craft/gather interactions substitute for unimplemented original minigames.'}

    def stage(title_id, description_id, giver, turnin, goals, *, unlock=(), items=(), coins=10, title=None, provenance=None):
        sid = 910001 + len(stages)
        for i, item in enumerate(goals, 1):
            item['id'] = sid * 100 + i
        stages.append({'id': sid, 'original_quest_id': None, 'title_id': title_id,
                       'description_id': description_id, 'title': title or locale[title_id],
                       'description': locale[description_id], 'giver_guid': giver,
                       'turnin_guid': turnin, 'ordered': True, 'objectives': goals,
                       'reward': {'stage_id': sid, 'coins': coins, 'items': list(items)},
                       'unlock_profiles': list(unlock),
                       'provenance': provenance or 'Recovered title/story actions; reconstructed reserved quest ID, prerequisites, counts, reward economy and actor bindings. No missing original Quests.json claims.'})

    stage(4419559, 4419560, darkthorne, darkthorne, [goal(0, 'class_selected', profile_id=43, description='Switch to Brawler using the native Jobs menu.'), goal(4419602, 'interact', actor_guid=journal)])
    stage(4419563, 4419564, darkthorne, pappy, [goal(4419610, 'dungeon_completed', encounter_id=411), goal(4419612, 'talk', actor_guid=pappy)], coins=25)
    stage(4420042, 4420043, pappy, pappy, [goal(4420047, 'interact', actor_guid=well)] + [goal(5100043, 'interact', actor_guid=g, description=f'Extinguish farm fire{i}.') for i, g in enumerate(fires, 1)])
    stage(4420050, 4420051, mae, mae, [goal(4420055, 'talk', actor_guid=mae), goal(0, 'class_selected', profile_id=45, description='Switch to Chef.'), goal(0, 'cosmetic_equipped', definition_id=13597, description='Equip the Student Chef Shirt in the Chef equipment screen.'), goal(5100033, 'gather', 3, resource_id=1001, description='Harvest3 Rainleaf bundles at the local tree.'), goal(5100033, 'gather', 3, resource_id=1002, description='Harvest3 Gummyfruit bundles at the local vine.'), goal(5100039, 'craft', recipe_id=2001)], unlock=[45], items=[(14794, 1)])
    stage(4420058, 4420059, cleetus, cleetus, [goal(4420063, 'talk', actor_guid=cleetus), goal(0, 'class_selected', profile_id=14, description='Switch to Miner.'), goal(5100117, 'gather', 5, resource_id=1003), goal(5100121, 'craft', 5, recipe_id=2002)], unlock=[14])
    stage(4420066, 4420067, cleetus, cleetus, [goal(0, 'class_selected', profile_id=16, description='Switch to Blacksmith.'), goal(4420071, 'craft', recipe_id=2003, description='Use the anvil to forge a local substitute for the Funky/Spunky Scrapper Hammer. Original recipe: 10 Copper Bars, 5 Turquoise, 5 Wooden Planks, 5 Thorny Fasteners. This interaction substitute does not grant or consume those original materials.')], unlock=[16], items=[(75030, 1)], coins=16, provenance='Wiki Quest:Your First Weapon revision27815 proves16 coins and the10/5/5/5 material grant. Its goal says Funky, dialogue says Spunky; supplied locale calls it Spunky. Original hammer, Copper Bar and Wooden Plank definitions are absent from supplied catalogs. Local75030 hammer and anvil interaction remain explicit substitutes; no original ingredient grant/consumption is claimed. Reserved quest ID and actor binding are reconstructed.')
    stage(4420074, 4420075, cleetus, cleetus, [goal(0, 'class_selected', profile_id=43, description='Switch to Brawler.'), goal(4420079, 'cosmetic_equipped', definition_id=75030, description='Equip Brawler\'s Mallet of Sweeps (explicit local hammer substitute).'), goal(5100203, 'defeat', 5, definition_id=3156)], coins=18, title=wolf_page['title'].removeprefix('Quest:'), provenance='Wiki Quest:Fighting off the Pack revision28955 proves Brawler, five Hooligan Wolves, Cleetus turn-in and18 coins. Supplied locale title4420074 is Fighting the Pack; local visible text uses the wiki title. Numeric quest binding, enemy balance and substitute hammer remain reconstructed.')
    stage(4420082, 4420083, pappy, pappy, [goal(4420087, 'talk', actor_guid=pappy), goal(5100268, 'defeat', definition_id=920001)], coins=20)
    stage(4420090, 4420091, pappy, bartle, [goal(4420095, 'talk', actor_guid=bartle)])
    stage(4420098, 4420099, bartle, bartle, [goal(5100332, 'interact', actor_guid=g, description=f'Destroy Gloam Artifact{i}.') for i, g in enumerate(artifacts, 1)])
    stage(4420106, 4420107, bartle, bartle, [goal(4420111, 'defeat', definition_id=920002)], coins=25)
    stage(4420114, 4420115, bartle, queen, [goal(4420119, 'visit', zone_id=0, position=lookup[queen]['position'], radius=25.0), goal(5100330, 'talk', actor_guid=queen)], coins=25)
    stage(5100479, 5100480, mae, mae, [goal(0, 'class_selected', profile_id=45, description='Switch to Chef.'), goal(5100033, 'gather', 3, resource_id=1001, description='Harvest3 more Rainleaf bundles.'), goal(5100033, 'gather', 3, resource_id=1002, description='Harvest3 more Gummyfruit bundles.'), goal(5100039, 'craft', recipe_id=2001)])
    stage(5100483, 5100484, cleetus, cleetus, [goal(0, 'class_selected', profile_id=14, description='Switch to Miner.'), goal(5100117, 'gather', 10, resource_id=1003, description='Mine10 more Copper Ore for the local bonus contract.')])
    stage(5100487, 5100488, cleetus, cleetus, [goal(0, 'class_selected', profile_id=14, description='Switch to Miner.'), goal(5100121, 'craft', 5, recipe_id=2002), goal(0, 'class_selected', profile_id=16, description='Switch to Blacksmith.'), goal(5100503, 'craft', recipe_id=2003, description='Forge another local beginner hammer.')])
    stage(5100581, 5100582, queen, captain, [goal(5100586, 'talk', actor_guid=ellandra), goal(5100588, 'talk', actor_guid=maximus), goal(5100590, 'talk', actor_guid=scotty), goal(5100592, 'talk', actor_guid=captain)], unlock=[11, 35])
    stage(5100609, 5100617, captain, captain, [goal(5100611, 'class_selected', profile_id=1), goal(0, 'class_selected', profile_id=11, description='Try the Medic job in the native Jobs menu.'), goal(0, 'class_selected', profile_id=35, description='Try the Archer job in the native Jobs menu.'), goal(0, 'class_selected', profile_id=45, description='Switch to Chef and show your earned hat.'), goal(0, 'cosmetic_equipped', definition_id=14794, description='Equip the Student Chef Hat earned earlier.')])
    discoveries = []
    for ident, name_id, position, radius in [(1, 442216, lookup[darkthorne]['position'], 25.0), (2, 18735, [-2141.0, -57.0, 260.0], 100.0), (3, 5100069, lookup[bartle]['position'], 45.0), (4, 5100361, lookup[queen]['position'], 45.0)]:
        discoveries.append({'id': ident, 'name_id': name_id, 'name': locale[name_id], 'zone_id': 0, 'position': position, 'radius': radius, 'reward': {'stage_id': 920100 + ident, 'coins': 5, 'items': []}, 'provenance': 'Recovered hub name/location where available; local four-location discovery checklist/radius/reward. Cavern marker is the reconstructed entry portal, not a claim of cave traversal.'})
    content = {'schema_version': 1, 'scope': 'Recovered NPX4.0 opening story, three optional farm stickers and beginner job/appearance epilogue. Local bindings are reconstructed.', 'reconstruction_notice': 'Opening restoration: original story names/models; local quest bindings, resource interactions, combat rules, discoveries and most rewards are reconstructed. Farnum wolf count and two coin rewards are wiki-recovered. Original crafting minigames/cinematics and material grant/consumption are not restored.', 'initial_profiles': [1, 43], 'actors': actors, 'discoveries': discoveries, 'stages': stages, 'provenance': {'source_revision': '9e082025391177d5a33ee18c714ddd06b457a695', 'sources': [{'path': str(p), 'sha256': hashlib.sha256(p.read_bytes()).hexdigest()} for p in paths], 'wiki_farnum': {'quests': [{k: p[k] for k in ('title', 'source_url', 'revision_id', 'revision_timestamp')} for p in (weapon_page, wolf_page)], 'original_material_grant': ingredients, 'material_runtime_status': 'Recorded evidence only. No acceptance-time grant or anvil consumption implemented; unknown supplied IDs remain empty arrays.', 'hammer_names': {'wiki_goal': 'Funky Scrapper Hammer', 'wiki_dialogue_and_supplied_locale': 'Spunky Scrapper Hammer', 'supplied_definition_ids': supplied_ids(102173)}, 'title_variation': {'wiki': wolf_page['title'].removeprefix('Quest:'), 'supplied_locale': locale[4420074]}, 'historical_limit': 'Wiki revisions are2013; supplied client is2014. Numeric quest IDs and exact2014 recipe/rewards are not established.'}, 'missing': ['original quest DB and most reward bindings', 'native crafting minigame scripting and material grant/consumption', 'exact original Mac/Gloam Creature actor IDs'], 'local_substitutions': {'hammer': 75030, 'original_hammer_name_id': 102173, 'local_gloam_definition': 920001, 'local_mac_definition': 920002}}}
    raw = (json.dumps(content, ensure_ascii=False, indent=2) + '\n').encode('utf-8')
    with target.open('xb') as handle:
        handle.write(raw)
    print(json.dumps({'path': str(target), 'sha256': hashlib.sha256(raw).hexdigest(), 'stages': len(stages), 'objectives': sum(len(s['objectives']) for s in stages), 'actors': len(actors), 'discoveries': len(discoveries)}, indent=2))


if __name__ == '__main__':
    main()
