#!/usr/bin/env node
/**
 * Read-only, strict MongoDB -> PostgreSQL archive reconciliation.
 * Every source collection must have the same number of distinct IDs in
 * owner-scoped PostgreSQL archival storage before a destructive cutover.
 */
import {MongoClient} from 'mongodb';
import pg from 'pg';
const {Client}=pg;
const mongoUri=process.env.MONGO_URI,postgresUrl=process.env.POSTGRES_DATABASE_URL;
const username=process.env.TARGET_REDDIT_USERNAME;
if(!mongoUri||!postgresUrl||!username){
 console.error('Set MONGO_URI, POSTGRES_DATABASE_URL and TARGET_REDDIT_USERNAME');
 process.exit(2);
}
const mongo=new MongoClient(mongoUri,{serverSelectionTimeoutMS:10000});
const sql=new Client({connectionString:postgresUrl});
let failed=false;
try{
 await mongo.connect();await sql.connect();
 const user=await sql.query('SELECT id FROM users WHERE reddit_username=$1',[username]);
 if(user.rowCount!==1)throw Error('Target user is missing or ambiguous');
 const owner=user.rows[0].id;
 const src=mongo.db();
 const coll=(await src.listCollections({},{nameOnly:true}).toArray()).filter(c=>!c.name.startsWith('system.'));
 const rows=[];
 for(const {name} of coll){
   const [sourceCount,archive]=await Promise.all([
     src.collection(name).countDocuments({}),
     sql.query('SELECT count(*)::bigint AS count FROM legacy_records WHERE owner_id=$1 AND collection_name=$2',[owner,name])
   ]);
   const archived=Number(archive.rows[0].count);
   const matches=sourceCount===archived;
   if(!matches)failed=true;
   rows.push({collection:name,source:sourceCount,archive:archived,matches});
 }
 const extra=await sql.query('SELECT collection_name,count(*)::bigint AS count FROM legacy_records WHERE owner_id=$1 GROUP BY collection_name',[owner]);
 for(const r of extra.rows){
   if(!rows.some(x=>x.collection===r.collection_name)){rows.push({collection:r.collection_name,source:0,archive:Number(r.count),matches:false});failed=true;}
 }
 console.log(JSON.stringify({owner,collections:rows,archive_reconciled:!failed},null,2));
 if(failed)process.exitCode=1;
}catch(error){
 console.error('Reconciliation failed:',error instanceof Error?error.message:error);
 process.exitCode=1;
}finally{await Promise.allSettled([mongo.close(),sql.end()]);}
