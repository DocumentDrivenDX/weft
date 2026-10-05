import Ajv2020 from 'ajv/dist/2020.js';
const schema=await Bun.file('docs/helix/02-design/contracts/backend-manifest-v0.2.schema.json').json();
const validate=new Ajv2020({strict:false,allErrors:true}).compile(schema);
const reports=await Bun.file('target/b003/reports.json').json();let count=0;
for(const {id,response} of reports){if(response.status==='emitted'){if(!validate(response.manifest))throw new Error(id+': '+JSON.stringify(validate.errors));count++;}}
console.log(`Validated ${count} emitted third-backend manifest snapshots against the versioned schema.`);
