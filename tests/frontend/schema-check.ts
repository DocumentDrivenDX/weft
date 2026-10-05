import Ajv2020 from 'ajv/dist/2020.js';
const schema=await Bun.file('docs/helix/02-design/contracts/logical-plan.schema.json').json();
const validate=new Ajv2020({strict:false,allErrors:true}).compile(schema);
const reports=await Bun.file('target/b002/reports.json').json();let count=0;
for(const {id,response} of reports){
 if(response.status==='resolved'){
  if(!validate(response.logicalPlan))throw new Error(id+': '+JSON.stringify(validate.errors));
  count++;
 }
}
console.log(`Validated ${count} typed frontend plans against the public IR schema.`);
