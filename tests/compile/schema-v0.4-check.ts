import Ajv2020 from 'ajv/dist/2020.js';
// Envelope controls only. Model, query, target and capability admission are separate.
const ajv = new Ajv2020({strict:false, allErrors:true});
const validate = ajv.compile(await Bun.file('docs/helix/02-design/contracts/compile-request-v0.4.schema.json').json());
const original = (await Bun.file('docs/helix/03-test/fixtures/cases.json').json())[0].request;
const originalModules = JSON.stringify(original.modules);
const request = {...original, interfaceVersion:'weft-compile/0.4.0', dialect:'weft-sql/0.4.0'};
let controls = 0;
function check(id:string, value:unknown, expected:boolean) {
  controls++;
  if (Boolean(validate(value)) !== expected) throw new Error(id+': '+JSON.stringify(validate.errors));
}
check('closed04-envelope', request, true);
check('exact-numeric-parameters', {...request, parameters:{quantity:{family:'integer',value:'9007199254740993'},price:{family:'decimal',value:'12.5000'}}}, true);
for (const version of ['0.1.0','0.2.0','0.3.0']) {
  check('mixed-interface-'+version, {...request, interfaceVersion:'weft-compile/'+version}, false);
  check('mixed-dialect-'+version, {...request, dialect:'weft-sql/'+version}, false);
}
check('old-read-profile', {...request, readProfile:{version:'weft-application-read/0.2.0',subset:'entity-page'}}, false);
check('unknown-path-settings', {...request, pathSettings:{bound:20}}, false);
check('unknown-endpoint-setting', {...request, target:{...request.target, endpoint:'https://untrusted.invalid'}}, false);
check('unknown-registration-override', {...request, options:{allowCandidate:true, backendInterface:'weft-backend-interface/0.2.0'}}, false);
check('native-float-parameter', {...request, parameters:{quantity:{family:'integer',value:9007199254740993}}}, false);
check('unsupported-parameter-family', {...request, parameters:{quantity:{family:'float',value:'1.0'}}}, false);
check('missing-original-source-pin', {...request, modules:request.modules.map((m:any)=>({...m,pin:{...m.pin,sha256:'missing'}}))}, false);
if (JSON.stringify(request.modules) !== originalModules) throw new Error('Original model bytes/pins changed');
console.log(`Validated ${controls} 0.4 request-envelope controls; no compiler or engine execution.`);
