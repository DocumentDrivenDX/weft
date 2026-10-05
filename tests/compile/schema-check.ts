import Ajv2020 from 'ajv/dist/2020.js';
const ajv = new Ajv2020({strict:false,allErrors:true});
const dir='docs/helix/02-design/contracts/';
for(const name of ['logical-plan.schema.json','logical-plan-v0.2.schema.json','backend-manifest-v0.2.schema.json']) ajv.addSchema(await Bun.file(dir+name).json());
const validators=new Map();
for(const [version,name] of [['weft-compile/0.1.0','compile-response.schema.json'],['weft-compile/0.2.0','compile-response-v0.2.schema.json']]) validators.set(version,ajv.compile(await Bun.file(dir+name).json()));
const reports=await Bun.file('target/b004/reports.json').json();
for(const {id,response} of reports){const validate=validators.get(response.interfaceVersion);if(!validate?.(response))throw new Error(id+': '+JSON.stringify(validate?.errors));}
console.log(`Validated ${reports.length} public responses against their versioned schemas.`);
