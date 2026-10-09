import Ajv2020 from 'ajv/dist/2020.js';
import {readdirSync,readFileSync,writeFileSync} from 'node:fs';
import {join} from 'node:path';
const [artifacts,output]=process.argv.slice(2);if(!artifacts||!output)throw new Error('Explicit artifacts/output required');
const ajv=new Ajv2020({strict:false,allErrors:true});
const schemas='docs/helix/02-design/contracts';for(const file of readdirSync(schemas).filter(f=>f.endsWith('.schema.json'))) {const schema=JSON.parse(readFileSync(join(schemas,file),'utf8'));ajv.addSchema(schema);}
const responseSchema=JSON.parse(readFileSync(join(schemas,'compile-response-v0.3.schema.json'),'utf8'));const response=ajv.getSchema(responseSchema.$id)!;
const requestSchema=JSON.parse(readFileSync(join(schemas,'compile-request-v0.3.schema.json'),'utf8'));const request=ajv.getSchema(requestSchema.$id)!;
const cases=[];for(const file of readdirSync(artifacts).filter(f=>f.endsWith('-artifact.json'))){const value=JSON.parse(readFileSync(join(artifacts,file),'utf8'));if(!response(value))throw new Error(file+': '+JSON.stringify(response.errors));const req=JSON.parse(readFileSync(join(artifacts,file.replace('-artifact.json','-request.json')),'utf8'));if(!request(req))throw new Error(file+': '+JSON.stringify(request.errors));cases.push({file,status:value.status,officialRequest:true,officialResponse:true});}
writeFileSync(output,JSON.stringify({format:'weft-null03-schema-check/0.1',cases},null,2)+'\n',{flag:'wx'});
