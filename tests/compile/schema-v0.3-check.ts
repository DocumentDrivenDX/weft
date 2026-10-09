import Ajv2020 from 'ajv/dist/2020.js';
const ajv = new Ajv2020({strict:false,allErrors:true});
const validate=ajv.compile(await Bun.file('docs/helix/02-design/contracts/compile-request-v0.3.schema.json').json());
const baseline=(await Bun.file('docs/helix/03-test/fixtures/cases.json').json())[0].request;
const request={...baseline,interfaceVersion:'weft-compile/0.3.0',dialect:'weft-sql/0.3.0'};
const check=(id:string,r:unknown,expected:boolean)=>{if(Boolean(validate(r))!==expected)throw new Error(id+': '+JSON.stringify(validate.errors));};
check('explicit-new-profile',request,true);
check('exact-numeric-parameters',{...request,parameters:{quantity:{family:'integer',value:'9007199254740993'},price:{family:'decimal',value:'12.5000'}}},true);
for(const version of ['0.1.0','0.2.0']) {
 check('old-interface-'+version,{...request,interfaceVersion:'weft-compile/'+version},false);
 check('old-dialect-'+version,{...request,dialect:'weft-sql/'+version},false);
}
check('old-read-subset',{...request,readProfile:{version:'weft-application-read/0.2.0',subset:'entity-page'}},false);
check('unknown-member',{...request,unknown:true},false);
check('numeric-float-parameter',{...request,parameters:{quantity:{family:'integer',value:9007199254740993}}},false);
check('unknown-parameter-meaning',{...request,parameters:{quantity:{family:'float',value:'1.0'}}},false);
check('parameter-extension-member',{...request,parameters:{quantity:{family:'integer',value:'1',bits:64}}},false);
check('invalid-source-pin',{...request,modules:request.modules.map((m:any)=>({...m,pin:{...m.pin,sha256:'missing'}}))},false);
console.log('Validated two positive and ten negative arithmetic request envelope controls.');
