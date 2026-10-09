/** Lossless JSON token custody only; actual public UMF owns scalar validity. */
import {resolve,join} from 'node:path';import {pathToFileURL} from 'node:url';
const [repoArg,inputArg,outputArg]=process.argv.slice(2);if(!repoArg||!inputArg||!outputArg)throw Error('Explicit UMF source, request and fresh receipt required');
const repo=resolve(repoArg),pin='c7c95e1c4ea5b72541f47fa0350ca467ff02f395';
for(const [args,expected]of [[['rev-parse','HEAD'],pin],[['status','--porcelain'],'']]as const){const r=Bun.spawnSync(['git','-C',repo,...args]);if(r.exitCode||new TextDecoder().decode(r.stdout).trim()!==expected)throw Error('Clean pinned public UMF required');}
const {parseDocument,isMap,isScalar}=await import(pathToFileURL(join(repo,'node_modules/yaml/dist/index.js')).href);
const {readDocument}=await import(pathToFileURL(join(repo,'src/model/document.ts')).href);const {validateCoreFieldValue}=await import(pathToFileURL(join(repo,'src/model/schema-properties.ts')).href);
const text=await Bun.file(inputArg).text(),request=JSON.parse(text),source=readDocument(request.sourceText,'json');
if(!Array.isArray(request.modelPins)||request.modelPins.length!==1)throw Error('One original selected model pin required');
const modelPin=request.modelPins[0],digest=new Bun.CryptoHasher('sha256').update(request.sourceText).digest('hex');
if(Object.keys(modelPin).sort().join(',')!=='documentId,revision,sha256,umfVersion'||modelPin.documentId!==source.id||modelPin.umfVersion!==source.umf||modelPin.sha256!==digest||typeof modelPin.revision!=='string'||!modelPin.revision)throw Error('Exact original source model pin differs');
for(const entry of request.checks)for(const identity of [entry.check.field,entry.check.record])if(!identity||identity.documentId!==modelPin.documentId||identity.revision!==modelPin.revision||typeof identity.module!=='string'||typeof identity.element!=='string')throw Error('Source field/record identity differs from original pin');
const observations=request.checks.map((entry:any)=>({check:entry.check,rows:entry.rows.map((row:any)=>{
 let token:any=null,span:any=null,result:any=null,error:any=null,failureClass:any="source-integrity";
 try{JSON.parse(row.props_json);const ast=parseDocument(row.props_json,{schema:'json',uniqueKeys:true,strict:true,keepSourceTokens:true,intAsBigInt:true});if(ast.errors.length||ast.warnings.length||!isMap(ast.contents))throw Error('Strict unique JSON object required');
 const pairs=ast.contents.items.filter((pair:any)=>isScalar(pair.key)&&pair.key.value===entry.check.propertyId);if(pairs.length!==1)throw Error('Exactly one supplied property required');const node=pairs[0].value;if(!isScalar(node)||node.tag||!node.range)throw Error('Original untagged scalar token required');span=node.range.slice(0,2);token=row.props_json.slice(span[0],span[1]);
 // Syntax determines carrier kind; AST numeric value is never used.
 if(!token||!'-0123456789'.includes(token[0]))throw Error('JSON numeric carrier required');
 result=validateCoreFieldValue(source,{module:entry.check.field.module,element:entry.check.field.element},{integerToken:token});
 if(result.valid!==true||result.complete!==true)throw Error('Public original integer source validation refused');if(row.extracted_token!==token){failureClass='backend-capability';throw Error('Native token extraction differs from original lexical source');}
 }catch(e){error=String(e);}
 return {original:row,token,span,publicResult:result,error,failureClass:error===null?null:failureClass,admitted:error===null};})}));
const output=resolve(outputArg);if(await Bun.file(output).exists())throw Error('Fresh receipt required');await Bun.write(output,JSON.stringify({umfRevision:pin,originalRequestText:text,observations,admitted:observations.every((o:any)=>o.rows.every((r:any)=>r.admitted)),qualification:'Tokenization is structural custody only; actual public UMF validates source values. Native snapshot and artifact admission remain host obligations.'})+'\n');
