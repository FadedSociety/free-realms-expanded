"""Polite, resumable public MediaWiki research archive and SQLite/vec search.

Run with lab/tools/wiki-env/Scripts/python.exe. No credentials, browser
impersonation, challenge solving or proxy rotation is used. Sources are data.
"""
from __future__ import annotations
import argparse
import hashlib
import importlib.metadata
import json
from pathlib import Path
import re
import sqlite3
import sys
import time
from datetime import datetime, timezone
from urllib.parse import quote, urlencode

import mwparserfromhell
from scrapling import Selector
from scrapling.fetchers import Fetcher
import sqlite_vec

PROJECT = Path(__file__).resolve().parents[1]
LAB = PROJECT.parent / 'lab'
ROOT = LAB / 'references/freerealms-wiki'
API = 'https://freerealms.fandom.com/api.php'
USER_AGENT = 'FreeRealmsResearch/1.0 (personal game restoration research; single worker)'
MODEL = 'BAAI/bge-small-en-v1.5'
MODEL_REPOSITORY = 'Qdrant/bge-small-en-v1.5-onnx-Q'
MODEL_REVISION = 'aa8f8b060edb00e03bfdd08813a2949946c8ba55'
DIMENSIONS = 384
PRIORITY = [
    'Starting The Game', 'Starting the Game', 'Main Quest', 'Quests', 'Quest',
    "Hero's Journal", 'The Last Best Hope', "Saving Farnum's Farm", 'Cooking 101',
    'Gear Up!', 'Your First Weapon', 'Fighting the Pack', 'One More Gloam',
    "Where's the Help?", 'Pull the Plug', 'Cobblestone Showdown', 'Off to the Queen',
    'Darkthorne', 'Pappy', 'Farnum', 'Bartle', 'Mac', 'Cobblestone Village',
    'Dungeons', 'Battles', 'Memorial Caverns', 'Classes', 'Jobs', 'Adventurer',
    'Brawler', 'Warrior', 'Ninja', 'Wizard', 'Medic', 'Archer', 'Chef',
    'Collections', 'Discovery', 'Discoveries', 'Cosmetics', 'Equipment',
    'Sanctuary', 'Sacred Glade', 'Fabled Realms', 'Gloam',
]

def utc():
    return datetime.now(timezone.utc).isoformat()

def digest(data):
    return hashlib.sha256(data).hexdigest()

def connect(read_only=False):
    ROOT.mkdir(parents=True, exist_ok=True)
    if read_only:
        db = sqlite3.connect((ROOT/'wiki.sqlite').as_uri()+'?mode=ro',uri=True)
    else:
        db = sqlite3.connect(ROOT/'wiki.sqlite')
    db.row_factory = sqlite3.Row
    db.enable_load_extension(True)
    sqlite_vec.load(db)
    db.enable_load_extension(False)
    db.execute('PRAGMA foreign_keys=ON')
    if read_only: return db
    db.executescript('''
      CREATE TABLE IF NOT EXISTS metadata(key TEXT PRIMARY KEY,value TEXT NOT NULL);
      CREATE TABLE IF NOT EXISTS requests(
        id INTEGER PRIMARY KEY,url TEXT NOT NULL,fetched_at TEXT NOT NULL,
        status INTEGER NOT NULL,sha256 TEXT NOT NULL,raw_path TEXT NOT NULL,
        purpose TEXT NOT NULL);
      CREATE TABLE IF NOT EXISTS titles(
        title TEXT PRIMARY KEY,page_id INTEGER,namespace INTEGER NOT NULL DEFAULT 0,
        status TEXT NOT NULL DEFAULT 'pending');
      CREATE TABLE IF NOT EXISTS pages(
        id INTEGER PRIMARY KEY,title TEXT UNIQUE NOT NULL,source_url TEXT NOT NULL,
        page_id INTEGER,namespace INTEGER NOT NULL,revision_id INTEGER,
        revision_timestamp TEXT,fetched_at TEXT NOT NULL,request_id INTEGER NOT NULL,
        sha256 TEXT NOT NULL,raw_wikitext_path TEXT NOT NULL,
        clean_text TEXT NOT NULL,redirect_target TEXT,
        FOREIGN KEY(request_id) REFERENCES requests(id));
      CREATE TABLE IF NOT EXISTS chunks(
        id INTEGER PRIMARY KEY,page_id INTEGER NOT NULL,chunk_index INTEGER NOT NULL,
        text TEXT NOT NULL,sha256 TEXT NOT NULL,UNIQUE(page_id,chunk_index),
        FOREIGN KEY(page_id) REFERENCES pages(id));
      CREATE VIRTUAL TABLE IF NOT EXISTS chunks_fts USING fts5(
        title,text,content='',tokenize='unicode61 remove_diacritics 2');
      CREATE VIRTUAL TABLE IF NOT EXISTS chunk_vectors USING vec0(
        chunk_id INTEGER PRIMARY KEY,embedding FLOAT[384] distance_metric=cosine);
    ''')
    db.execute('INSERT OR REPLACE INTO metadata VALUES(?,?)',('schema_version','1'))
    db.execute('INSERT OR REPLACE INTO metadata VALUES(?,?)',('sqlite_vec_version',db.execute('SELECT vec_version()').fetchone()[0]))
    db.execute('INSERT OR REPLACE INTO metadata VALUES(?,?)',('license','Community wiki text: CC-BY-SA unless otherwise noted. Source and revision links retained.'))
    db.commit()
    return db

class Client:
    def __init__(self, db, delay=2.0):
        self.db, self.delay, self.last = db, max(delay, 1.0), 0.0
    def get(self, params, purpose):
        wait = self.delay-(time.monotonic()-self.last)
        if wait > 0:
            time.sleep(wait)
        params = {'format':'json','formatversion':2,'maxlag':5,**params}
        url = API+'?'+urlencode(params)
        response = Fetcher.get(API, params=params, impersonate=None,
                               stealthy_headers=False, headers={'User-Agent':USER_AGENT},
                               retries=0, timeout=35)
        self.last = time.monotonic()
        body = response.body
        sha = digest(body)
        raw = ROOT/'raw'/f'{sha}.json'
        raw.parent.mkdir(exist_ok=True)
        if not raw.exists(): raw.write_bytes(body)
        request_id = self.db.execute(
            'INSERT INTO requests(url,fetched_at,status,sha256,raw_path,purpose) VALUES(?,?,?,?,?,?)',
            (url,utc(),response.status,sha,str(raw.relative_to(ROOT)),purpose)).lastrowid
        self.db.commit()
        if response.status != 200:
            raise RuntimeError(f'HTTP {response.status}; stopped without bypass: {url}')
        data = response.json()
        if 'error' in data:
            raise RuntimeError(f'MediaWiki error; stopped: {data["error"]}')
        return data, request_id

def clean_wikitext(text):
    code = mwparserfromhell.parse(text)
    # Keep infobox/quest template values: strip_code alone silently loses them.
    fields=[]
    for template in code.filter_templates(recursive=True):
        if len(template.params):
            fields.append(str(template.name).strip()+': '+ '; '.join(
                str(p.name).strip()+': '+p.value.strip_code(normalize=True,collapse=True).strip()
                for p in template.params if str(p.value).strip()))
    plain = code.strip_code(normalize=True, collapse=True)
    if '<' in plain:
        plain = Selector('<div>'+plain+'</div>').get_all_text(separator=' ',strip=True)
    plain = re.sub(r'[ \t]+',' ',plain)
    plain = re.sub(r'\n{3,}','\n\n',plain).strip()
    return plain + ('\n\nTemplate fields:\n'+'\n'.join(fields) if fields else '')

def split_chunks(title, text, words=200, overlap=35):
    tokens = text.split()
    for start in range(0,len(tokens),words-overlap):
        yield title+'\n'+' '.join(tokens[start:start+words])
        if start+words >= len(tokens): break

def add_page(db, page, request_id):
    title = page['title']
    if page.get('missing') or not page.get('revisions'):
        db.execute('UPDATE titles SET status=? WHERE title=?',('missing',title))
        return False
    revision = page['revisions'][0]
    text = revision['slots']['main']['content']
    sha = digest(text.encode())
    if db.execute('SELECT 1 FROM pages WHERE title=? AND sha256=?',(title,sha)).fetchone():
        db.execute('UPDATE titles SET status=? WHERE title=?',('done',title)); return False
    # Existing content is immutable in this crawl. Preserve this snapshot before
    # implementing a future revision-aware update rather than overwriting it.
    if db.execute('SELECT 1 FROM pages WHERE title=?',(title,)).fetchone():
        raise RuntimeError(f'Revision changed for {title}; preserve old snapshot before refresh')
    raw = ROOT/'wikitext'/f'{page["pageid"]}-{revision["revid"]}-{sha[:12]}.wiki'
    raw.parent.mkdir(exist_ok=True)
    raw.write_bytes(text.encode('utf-8'))
    cleaned = clean_wikitext(text)
    redirect = re.match(r'\s*#redirect\s*\[\[([^\]]+)\]\]',text,re.I)
    page_id = db.execute('''INSERT INTO pages(title,source_url,page_id,namespace,revision_id,
        revision_timestamp,fetched_at,request_id,sha256,raw_wikitext_path,clean_text,redirect_target)
        VALUES(?,?,?,?,?,?,?,?,?,?,?,?)''',
        (title,page.get('fullurl') or 'https://freerealms.fandom.com/wiki/'+quote(title.replace(' ','_')),
         page['pageid'],page['ns'],revision['revid'],revision['timestamp'],utc(),request_id,
         sha,str(raw.relative_to(ROOT)),cleaned,redirect.group(1) if redirect else None)).lastrowid
    for index, chunk in enumerate(split_chunks(title,cleaned)):
        chunk_id = db.execute('INSERT INTO chunks(page_id,chunk_index,text,sha256) VALUES(?,?,?,?)',
                   (page_id,index,chunk,digest(chunk.encode()))).lastrowid
        db.execute('INSERT INTO chunks_fts(rowid,title,text) VALUES(?,?,?)',(chunk_id,title,chunk))
    db.execute('UPDATE titles SET status=?,page_id=?,namespace=? WHERE title=?',('done',page['pageid'],page['ns'],title))
    return True

def crawl(args):
    db=connect(); client=Client(db,args.delay)
    if not db.execute("SELECT 1 FROM metadata WHERE key='siteinfo'").fetchone():
        data,_=client.get({'action':'query','meta':'siteinfo','siprop':'general|rightsinfo|statistics'},'siteinfo')
        db.execute('INSERT OR REPLACE INTO metadata VALUES(?,?)',('siteinfo',json.dumps(data))); db.commit()
    for title in PRIORITY:
        db.execute('INSERT OR IGNORE INTO titles(title) VALUES(?)',(title,))
    db.commit()
    def fetch_titles(titles):
        data,rid=client.get({'action':'query','prop':'revisions|info','inprop':'url',
                           'rvprop':'ids|timestamp|content','rvslots':'main',
                           'titles':'|'.join(titles)},'page-revisions')
        count=sum(add_page(db,p,rid) for p in data['query']['pages'])
        db.commit()
        print(json.dumps({'new_pages':count,'total_pages':db.execute('SELECT count(*) FROM pages').fetchone()[0]}),flush=True)
    pending_priority=[r[0] for r in db.execute(
        'SELECT title FROM titles WHERE status=? ORDER BY rowid',('pending',)) if r[0] in PRIORITY]
    for i in range(0,len(pending_priority),20): fetch_titles(pending_priority[i:i+20])
    # API allpages is the public equivalent of Special:AllPages, with separate
    # continuation for each selected namespace. User/talk/forums are excluded.
    for namespace in args.namespaces:
        prefix='' if namespace==0 else f'namespace_{namespace}_'
        complete=db.execute('SELECT value FROM metadata WHERE key=?',(prefix+'enumeration_complete',)).fetchone()
        if complete:
            db.execute('DELETE FROM metadata WHERE key=?',(prefix+'enumeration_cursor',))
            db.commit()
            continue
        cursor=db.execute('SELECT value FROM metadata WHERE key=?',(prefix+'enumeration_cursor',)).fetchone()
        while True:
            params={'action':'query','list':'allpages','apnamespace':namespace,'aplimit':500}
            if cursor: params['apcontinue']=cursor[0]
            data,_=client.get(params,'allpages')
            db.executemany('''INSERT INTO titles(title,page_id,namespace) VALUES(?,?,?)
                ON CONFLICT(title) DO UPDATE SET page_id=excluded.page_id,namespace=excluded.namespace''',
                [(p['title'],p['pageid'],p['ns']) for p in data['query']['allpages']])
            continuation=data.get('continue',{}).get('apcontinue')
            if continuation:
                db.execute('INSERT OR REPLACE INTO metadata VALUES(?,?)',(prefix+'enumeration_cursor',continuation))
            else:
                db.execute('INSERT OR REPLACE INTO metadata VALUES(?,?)',(prefix+'enumeration_complete',utc()))
                db.execute('DELETE FROM metadata WHERE key=?',(prefix+'enumeration_cursor',))
            db.commit()
            print(json.dumps({'namespace':namespace,'enumerated':db.execute('SELECT count(*) FROM titles').fetchone()[0],'continuation':continuation}),flush=True)
            if not continuation: break
            cursor=(continuation,)
    pending=[r[0] for r in db.execute('SELECT title FROM titles WHERE status=? ORDER BY rowid',('pending',))]
    for i in range(0,min(len(pending),args.limit),20): fetch_titles(pending[i:min(i+20,args.limit)])
    report(db)

def model():
    from fastembed import TextEmbedding
    directory=ROOT/'models'/MODEL_REVISION
    if not (directory/'model_optimized.onnx').exists():
        raise RuntimeError('Pinned model missing; run download-model first')
    return TextEmbedding(model_name=MODEL,specific_model_path=str(directory),threads=2)

def download_model():
    from huggingface_hub import snapshot_download
    directory=ROOT/'models'/MODEL_REVISION
    snapshot_download(MODEL_REPOSITORY,revision=MODEL_REVISION,local_dir=str(directory),
                      allow_patterns=['*.json','*.onnx','*.txt','README.md'],max_workers=2)
    manifest={'model':MODEL,'repository':MODEL_REPOSITORY,'revision':MODEL_REVISION,
              'license':'MIT','fetched_at':utc(),'dimensions':DIMENSIONS,
              'files':[{ 'path':str(p.relative_to(directory)), 'bytes':p.stat().st_size,
                         'sha256':digest(p.read_bytes())} for p in directory.rglob('*')
                         if p.is_file() and '.cache' not in p.parts]}
    (ROOT/'model-manifest.json').write_text(json.dumps(manifest,indent=2),encoding='utf-8')
    print(json.dumps(manifest,indent=2))

def embed(args):
    db=connect(); embedding=model()
    db.execute('INSERT OR REPLACE INTO metadata VALUES(?,?)',('embedding_model',MODEL))
    db.execute('INSERT OR REPLACE INTO metadata VALUES(?,?)',('embedding_revision',MODEL_REVISION))
    db.execute('INSERT OR REPLACE INTO metadata VALUES(?,?)',('embedding_kind','local ONNX dense semantic; cosine; 384 dimensions'))
    rows=db.execute('SELECT c.id,c.text FROM chunks c LEFT JOIN chunk_vectors v ON v.chunk_id=c.id WHERE v.chunk_id IS NULL ORDER BY c.id').fetchall()
    for start in range(0,len(rows),64):
        batch=rows[start:start+64]
        vectors=embedding.passage_embed([r['text'] for r in batch])
        for row, vector in zip(batch,vectors):
            db.execute('INSERT INTO chunk_vectors(chunk_id,embedding) VALUES(?,?)',(row['id'],vector.astype('float32').tobytes()))
        db.commit()
        print(json.dumps({'embedded':start+len(batch),'pending_initial':len(rows)}),flush=True)
    report(db)

def report(db):
    result={
      'timestamp':utc(),'database':str(ROOT/'wiki.sqlite'),
      'integrity':db.execute('PRAGMA integrity_check').fetchone()[0],
      'foreign_key_errors':[list(r) for r in db.execute('PRAGMA foreign_key_check')],
      'sqlite_version':sqlite3.sqlite_version,
      'sqlite_vec_version':db.execute('SELECT vec_version()').fetchone()[0],
      'counts':{table:db.execute('SELECT count(*) FROM '+table).fetchone()[0]
                for table in ['pages','titles','chunks','chunks_fts','chunk_vectors','requests']},
      'pages_by_namespace':{str(r[0]):r[1] for r in db.execute('SELECT namespace,count(*) FROM pages GROUP BY namespace')},
      'redirect_pages':db.execute('SELECT count(*) FROM pages WHERE redirect_target IS NOT NULL').fetchone()[0],
      'empty_pages':db.execute("SELECT count(*) FROM pages WHERE clean_text='' ").fetchone()[0],
      'pending_vectors':db.execute('SELECT count(*) FROM chunks c LEFT JOIN chunk_vectors v ON v.chunk_id=c.id WHERE v.chunk_id IS NULL').fetchone()[0],
      'title_status':{r[0]:r[1] for r in db.execute('SELECT status,count(*) FROM titles GROUP BY status')},
      'metadata':{r[0]:r[1] for r in db.execute('SELECT key,value FROM metadata')},
      'packages':{name:importlib.metadata.version(name) for name in ['scrapling','sqlite-vec','fastembed','onnxruntime','mwparserfromhell']},
    }
    (ROOT/'coverage.json').write_text(json.dumps(result,indent=2),encoding='utf-8')
    print(json.dumps(result,indent=2))
    return result

def verify():
    db=connect(read_only=True)
    failures=[]
    for row in db.execute('SELECT sha256,raw_path FROM requests'):
        path=ROOT/row['raw_path']
        if not path.exists() or digest(path.read_bytes())!=row['sha256']:
            failures.append({'request_raw':row['raw_path']})
    for row in db.execute('SELECT title,sha256,raw_wikitext_path FROM pages'):
        path=ROOT/row['raw_wikitext_path']
        if not path.exists() or digest(path.read_bytes())!=row['sha256']:
            failures.append({'page_raw':row['title']})
    chunks=db.execute('SELECT count(*) FROM chunks').fetchone()[0]
    for table in ['chunks_fts','chunk_vectors']:
        count=db.execute('SELECT count(*) FROM '+table).fetchone()[0]
        if count!=chunks: failures.append({table:count,'expected':chunks})
    if db.execute('SELECT count(*) FROM chunk_vectors v LEFT JOIN chunks c ON c.id=v.chunk_id WHERE c.id IS NULL').fetchone()[0]:
        failures.append('orphan vector IDs')
    if db.execute('SELECT count(*) FROM chunks_fts f LEFT JOIN chunks c ON c.id=f.rowid WHERE c.id IS NULL').fetchone()[0]:
        failures.append('orphan FTS IDs')
    for row in db.execute('SELECT id,text,sha256 FROM chunks'):
        if digest(row['text'].encode())!=row['sha256']:
            failures.append({'chunk_hash':row['id']})
    if db.execute('PRAGMA integrity_check').fetchone()[0]!='ok': failures.append('sqlite integrity')
    if list(db.execute('PRAGMA foreign_key_check')): failures.append('foreign keys')
    manifest=json.loads((ROOT/'model-manifest.json').read_text(encoding='utf-8'))
    for record in manifest['files']:
        path=ROOT/'models'/manifest['revision']/record['path']
        if not path.exists() or digest(path.read_bytes())!=record['sha256']:
            failures.append({'model_file':record['path']})
    import numpy as np
    vector_rows=db.execute('SELECT chunk_id,embedding FROM chunk_vectors ORDER BY chunk_id').fetchall()
    norms=[]
    for row in vector_rows:
        vector=np.frombuffer(row['embedding'],dtype=np.float32)
        if len(vector)!=DIMENSIONS or not np.isfinite(vector).all():
            failures.append({'invalid_vector':row['chunk_id']})
        norms.append(float(np.linalg.norm(vector)))
    if norms and max(abs(value-1.0) for value in norms)>0.001:
        failures.append('unnormalized cosine vectors')
    sample_indices=sorted(set([0,len(vector_rows)//3,2*len(vector_rows)//3,len(vector_rows)-1])) if vector_rows else []
    samples=[vector_rows[i] for i in sample_indices]
    texts=[db.execute('SELECT text FROM chunks WHERE id=?',(row['chunk_id'],)).fetchone()[0] for row in samples]
    recomputed=list(model().passage_embed(texts)) if samples else []
    sample_errors=[]
    for row,vector in zip(samples,recomputed):
        error=float(np.max(np.abs(np.frombuffer(row['embedding'],dtype=np.float32)-vector)))
        sample_errors.append({'chunk_id':row['chunk_id'],'maximum_absolute_error':error})
        if error>1e-5: failures.append({'model_vector_mismatch':row['chunk_id'],'error':error})
    result={'timestamp':utc(),'status':'passed' if not failures else 'failed',
            'failures':failures,'vector_norm_min':min(norms) if norms else None,
            'vector_norm_max':max(norms) if norms else None,
            'recomputed_vector_samples':sample_errors,'coverage':report(db)}
    (ROOT/'verification.json').write_text(json.dumps(result,indent=2),encoding='utf-8')
    print(json.dumps({'status':result['status'],'failures':failures},indent=2))
    if failures: raise SystemExit(1)

def search(args):
    db=connect(read_only=True); results={}
    tokens=re.findall(r'\w+',args.query)
    fts=' OR '.join('"'+word+'"' for word in tokens)
    if args.mode in ('keyword','hybrid') and fts:
        results['keyword']=[dict(r) for r in db.execute('''
          SELECT c.id AS chunk_id,p.title,p.source_url,p.revision_id,p.revision_timestamp,p.fetched_at,p.sha256 AS source_sha256,
            c.chunk_index,c.text,bm25(chunks_fts,4.0,1.0) AS score
          FROM chunks_fts JOIN chunks c ON c.id=chunks_fts.rowid JOIN pages p ON p.id=c.page_id
          WHERE chunks_fts MATCH ? ORDER BY score LIMIT ?''',(fts,args.limit))]
    if args.mode in ('vector','hybrid'):
        embedding=next(model().query_embed(args.query)).astype('float32').tobytes()
        results['vector']=[dict(r) for r in db.execute('''
          SELECT c.id AS chunk_id,p.title,p.source_url,p.revision_id,p.revision_timestamp,p.fetched_at,p.sha256 AS source_sha256,
            c.chunk_index,c.text,v.distance AS score
          FROM chunk_vectors v JOIN chunks c ON c.id=v.chunk_id JOIN pages p ON p.id=c.page_id
          WHERE v.embedding MATCH ? AND k=? ORDER BY v.distance''',(embedding,args.limit))]
    for rows in results.values():
        for row in rows: row['revision_url']=row['source_url']+'?oldid='+str(row['revision_id'])
    if args.mode=='hybrid':
        combined={}
        for mode,rows in results.items():
            for rank,row in enumerate(rows,1):
                hit=combined.setdefault(row['chunk_id'],{**row,'score':0.0})
                hit['score']+=1.0/(60+rank)
                hit[mode+'_rank']=rank
        results={'rank_method':'reciprocal rank fusion, k=60; larger score is better',
                 'hybrid':sorted(combined.values(),key=lambda row:-row['score'])[:args.limit]}
    print(json.dumps(results,indent=2,ensure_ascii=False))

def main():
    if hasattr(sys.stdout,'reconfigure'): sys.stdout.reconfigure(encoding='utf-8')
    parser=argparse.ArgumentParser(description=__doc__)
    sub=parser.add_subparsers(dest='command',required=True)
    crawl_parser=sub.add_parser('crawl'); crawl_parser.add_argument('--limit',type=int,default=10000)
    crawl_parser.add_argument('--delay',type=float,default=2.0)
    crawl_parser.add_argument('--namespaces',nargs='+',type=int,default=[0,112])
    sub.add_parser('embed'); sub.add_parser('status'); sub.add_parser('download-model'); sub.add_parser('verify')
    search_parser=sub.add_parser('search'); search_parser.add_argument('query')
    search_parser.add_argument('--mode',choices=['keyword','vector','hybrid'],default='hybrid')
    search_parser.add_argument('--limit',type=int,default=5)
    args=parser.parse_args()
    if args.command=='crawl': crawl(args)
    elif args.command=='embed': embed(args)
    elif args.command=='search': search(args)
    elif args.command=='download-model': download_model()
    elif args.command=='verify': verify()
    else: report(connect(read_only=True))

if __name__=='__main__':
    main()
