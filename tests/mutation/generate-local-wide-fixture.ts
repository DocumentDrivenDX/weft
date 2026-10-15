// Test-only original owner oracle; no runtime sidecar.
if(!Bun.argv[2])throw Error('UMF source root required');
const owner=await import(Bun.argv[2]+'/src/index.ts');
const root=new URL('../../',import.meta.url).pathname;
const fixture=await Bun.file(root+'/tests/mutation/fixtures/local-products-input.json').json();
const model=JSON.parse(fixture.request.model.utf8),module=model.modules[0],record=module.elements.find((e:any)=>e.kind==='record');
for(let n=3;n<=64;n++){const id='extra-'+n;module.elements.push({id,name:id,kind:'field',scalarType:'string',cardinality:'one',nullability:'required',extensions:{}});record.members.push({module:'sales',element:id});}
const utf8=JSON.stringify(model),sha256=new Bun.CryptoHasher('sha256').update(utf8).digest('hex');
const artifact={mediaType:'application/json',utf8,sha256},pin={...fixture.request.modelPin,sha256},action={...fixture.request.action,documentSha256:sha256};
const preparation=owner.prepareActionPlanModel(pin,action,{registry:owner.createUmfRegistry(),models:[{pin,artifact}],actions:[{pin:action,artifact}],selections:[]});
const value={scope:'Synthetic64-field mapping regression, original UMF public preparation output; not adopter execution or issued authority',configuration:{actions:[{pin:action,snapshot:artifact,preparation}],selectionBackends:[]},request:{...fixture.request,model:artifact,modelPin:pin,action}};
await Bun.write(root+'/tests/mutation/fixtures/local-products-64-input.json',JSON.stringify(value,null,2)+'\n');
console.log(JSON.stringify({model:sha256,preparation:preparation.sha256,producer:JSON.parse(preparation.utf8).producer}));
