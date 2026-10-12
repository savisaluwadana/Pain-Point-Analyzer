#!/usr/bin/env node
/**
 * Loss-preserving historical MongoDB -> PostgreSQL import.
 * Default mode is read-only dry run. --apply imports source records in the archive
 * and projects + evidence in the new owner-scoped tables.
 */
import {MongoClient} from 'mongodb';
import pg from 'pg';

const {Client}=pg;
const apply=process.argv.includes('--apply');
const mongoUri=process.env.MONGO_URI;
const postgresUrl=process.env.POSTGRES_DATABASE_URL;
const redditUsername=process.env.TARGET_REDDIT_USERNAME;
if(!mongoUri||!postgresUrl||!redditUsername){
 console.error('Set MONGO_URI, POSTGRES_DATABASE_URL and TARGET_REDDIT_USERNAME. See v2/README.md.');
 process.exit(2);
}
const mongo=new MongoClient(mongoUri,{serverSelectionTimeoutMS:10000});
const sql=new Client({connectionString:postgresUrl});
const totals=[];
const clean=(s,n=10000)=>String(s??'').slice(0,n);
try{
 await mongo.connect();await sql.connect();
 const target=await sql.query('SELECT id FROM users WHERE reddit_username=$1',[redditUsername]);
 if(target.rowCount!==1)throw Error('Connect the destination Reddit account via OAuth first; target must be unique');
 const owner=target.rows[0].id;
 const sourceDb=mongo.db();
 const collections=(await sourceDb.listCollections({}, {nameOnly:true}).toArray()).filter(c=>!c.name.startsWith('system.'));
 console.log(JSON.stringify({mode:apply?'apply':'dry-run',collections:collections.map(c=>c.name),owner}));
 for(const coll of collections){
  let count=0,projected=0;
  const cursor=sourceDb.collection(coll.name).find({}).batchSize(100);
  for await(const doc of cursor){
   count++;
   if(!apply)continue;
   const legacyId=clean(doc._id,120);
   const payload=JSON.parse(JSON.stringify(doc));
   await sql.query('INSERT INTO legacy_records(owner_id,collection_name,legacy_id,payload) VALUES($1,$2,$3,$4) ON CONFLICT(owner_id,collection_name,legacy_id) DO UPDATE SET payload=excluded.payload',
     [owner,coll.name,legacyId,JSON.stringify(payload)]);
   if(coll.name==='researchprojects'){
     const name=clean(doc.name||'Imported project',160);
     await sql.query("INSERT INTO projects(owner_id,name,description) SELECT $1,$2,$3 WHERE NOT EXISTS(SELECT 1 FROM projects WHERE owner_id=$1 AND name=$2)",
       [owner,name,clean(doc.description,2000)]);
     projected++;
   }
   if(coll.name==='redditposts'){
     const sourceId=clean(doc.redditId||legacyId,150);
     if(!sourceId)continue;
     await sql.query(`INSERT INTO evidence(owner_id,source_kind,source_id,source_url,title,body,metadata,published_at)
       VALUES($1,'legacy-reddit',$2,$3,$4,$5,$6,$7)
       ON CONFLICT(owner_id,source_kind,source_id) DO UPDATE SET title=excluded.title,body=excluded.body,metadata=excluded.metadata`,
       [owner,sourceId,clean(doc.permalink?'https://www.reddit.com'+doc.permalink:doc.url,2048),clean(doc.title,500),
        clean(doc.selfText,10000),JSON.stringify({subreddit:doc.subreddit,legacy_id:legacyId}),
        doc.createdUtc?new Date(Number(doc.createdUtc)*1000):null]);
     projected++;
   }
   if(coll.name==='evidenceitems'){
     const sourceId=clean(doc.externalId||doc.fingerprint||legacyId,150);
     await sql.query(`INSERT INTO evidence(owner_id,source_kind,source_id,source_url,title,body,metadata,published_at)
       VALUES($1,'legacy-import',$2,$3,$4,$5,$6,$7)
       ON CONFLICT(owner_id,source_kind,source_id) DO UPDATE SET title=excluded.title,body=excluded.body,metadata=excluded.metadata`,
       [owner,sourceId,clean(doc.sourceUrl,2048),clean(doc.title,500),clean(doc.text,10000),
        JSON.stringify({legacy_kind:doc.sourceKind,legacy_source:doc.sourceName,legacy_id:legacyId}),
        doc.publishedAt?new Date(doc.publishedAt):null]);
     projected++;
   }
  }
  totals.push({collection:coll.name,records:count,projected});
  console.log(`${coll.name}: ${count} records${apply?' archived / imported':''}`);
 }
 console.log(JSON.stringify({mode:apply?'applied':'dry-run',totals},null,2));
 if(!apply)console.log('Dry run only. Backup MongoDB and run with --apply when ready.');
}catch(e){
 console.error('Import failed:',e instanceof Error?e.message:e);
 process.exitCode=1;
}finally{await Promise.allSettled([mongo.close(),sql.end()]);}
