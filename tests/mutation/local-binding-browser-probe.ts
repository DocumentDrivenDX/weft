import {PGlite} from '@electric-sql/pglite';
import {prepareDocument,registerCatalog} from 'truss-browser-catalog';
(globalThis as any).runBinding=async(fixture:any,layout:string,scalars:string)=>{
 const wasmPath='/weft_wasm.js';const wasm=await import(wasmPath);await wasm.default();
 const compiler=new wasm.MutationCompiler(JSON.stringify(fixture.configuration));const db=new PGlite();
 try{
  const version=(await db.query("SELECT current_setting('server_version_num') AS n,version() AS version")).rows[0] as any;
  if(version.n!=='180006'||!version.version.includes('PGlite 0.5.10'))throw Error('Pinned PGlite/PostgreSQL required');
  await db.exec(layout);await db.exec(scalars);
  const request=fixture.request,pin=request.modelPin;
  const registration=await registerCatalog(db,await prepareDocument({documentId:pin.documentId,documentRevision:pin.revision,sourceUtf8:request.model.utf8}));
  if(registration.outcome!=='registered')throw Error('Original catalog registration required');
  const catalog=registration.catalog,type=catalog.types[0],key=catalog.keys[0];
  const model=JSON.parse(request.model.utf8),record=model.modules.find((m:any)=>m.id===type.moduleId).elements.find((e:any)=>e.id===type.recordId);
  const identity=(module:string,element:string)=>({documentId:pin.documentId,revision:pin.revision,module,element});
  const binding={interfaceVersion:'truss-browser-binding/0.1',pgliteVersion:'0.5.10',postgresqlVersion:'18.6',
   store:{profile:'truss-browser-layout/0.1',origin:location.origin,namespace:'truss-browser:compiler-check',storeId:crypto.randomUUID(),incarnation:crypto.randomUUID(),layoutSha256:'c203d09418fb4eddda829d791dda11c20341227c2d1d82e86d4c24afda5e7294'},
   catalogRevision:catalog.catalogRevision,document:pin,maximumScanRows:1000,
   record:{identity:identity(type.moduleId,type.recordId),typeId:type.typeId,fields:record.members.map((m:any)=>({identity:identity(m.module,m.element),propertyId:catalog.properties.find((p:any)=>p.moduleId===m.module&&p.fieldId===m.element).propertyId})),key:{keyId:key.keyId,propertyIds:key.propertyIds}}};
  const utf8=JSON.stringify(binding),sha256=Array.from(new Uint8Array(await crypto.subtle.digest('SHA-256',new TextEncoder().encode(utf8))),b=>b.toString(16).padStart(2,'0')).join('');
  const artifact={mediaType:'application/json',utf8,sha256};
  const handle=compiler.register_browser_selection(JSON.stringify({model:request.model,modelPin:pin,binding:artifact}));
  const response=JSON.parse(compiler.compile_json(JSON.stringify(request)));
  if(response.status!=='compiled')throw Error(JSON.stringify(response));
  const plan=JSON.parse(response.plan.utf8);
  const lowered=JSON.parse(compiler.lower_selection_json(handle,JSON.stringify({selection:plan.selection.artifact,model:request.model,binding:artifact,maxItems:2})));
  if(lowered.status!=='lowered'||lowered.preflight.length!==4)throw Error(JSON.stringify(lowered));
  const props=(id:string,name:string)=>Object.fromEntries(binding.record.fields.map((f:any,i:number)=>[f.propertyId,{kind:'string',text:i===0?id:name}]));
  // Administrative fixture seeding, not a public mutation or issued capability.
  for(const [id,name] of [['bar','Sample'],['spare','Untouched']])await db.query('INSERT INTO truss.object(type_id,props,rev) VALUES($1,$2::jsonb,1)',[type.typeId,JSON.stringify(props(id!,name!))]);
  const values=(parameters:any[])=>parameters.map(p=>{if(typeof p.value.string!=='string')throw Error('String parameter required');return p.value.string;});
  const violations=[];
  for(const preflight of lowered.preflight){const rows=(await db.query(preflight.sql,values(preflight.parameters))).rows as any[];if(rows.length!==1||rows[0].violations!=='0')throw Error(JSON.stringify(rows));violations.push(rows[0].violations);}
  const rows=(await db.query(lowered.sql,values(lowered.parameters))).rows as any[];
  if(rows.length!==1||rows[0].key0!=='bar'||rows[0].version!=='1')throw Error(JSON.stringify(rows));
  return {catalogRevision:catalog.catalogRevision,bindingSha256:sha256,selection:rows,completeRecordPreflights:violations.length,physicalLowering:true,issuedOwner:false,selectionReadGrant:false,publicMutation:false,effects:false};
 }finally{compiler.free();await db.close();}
};
