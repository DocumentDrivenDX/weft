import {readFile,writeFile,mkdir,readdir} from 'node:fs/promises';
import {execFileSync} from 'node:child_process';
import {createHash} from 'node:crypto';
import path from 'node:path';
const root='dist/site';
const cli=process.env.INNSIGLE_CLI;
const key=process.env.INNSIGLE_KEY_FILE;
if (!cli || !key) throw new Error('INNSIGLE_CLI and INNSIGLE_KEY_FILE are required; unsigned deployment is refused.');
const config=JSON.parse(await readFile('.innsigle/config.json','utf8'));
async function files(dir){const found=[];for(const entry of await readdir(dir,{withFileTypes:true})){const p=path.join(dir,entry.name);if(entry.isDirectory())found.push(...await files(p));else found.push(p);}return found.sort();}
const contents=await files(root);
const records=[];
for(const file of contents){const relative=path.relative(root,file).split(path.sep).join('/');records.push({path:relative,sha256:createHash('sha256').update(await readFile(file)).digest('hex'),attestation:'.well-known/innsigle/claims/'+relative.replaceAll('/','--')+'.attestation.json'});}
await writeFile(root+'/content-manifest.json',JSON.stringify({issuer:'weft',composition:'model-primary',files:records},null,2)+'\n');
const all=[...records,{path:'content-manifest.json',attestation:'.well-known/innsigle/claims/content-manifest.json.attestation.json'}];
await mkdir(root+'/.well-known/innsigle/claims',{recursive:true});
await writeFile(root+'/.well-known/innsigle/keys.json',await readFile('.innsigle/public/keys.json'));
for(const record of all){
 const content=root+'/'+record.path;const claim=root+'/.claim.json';const attestation=root+'/'+record.attestation;
 execFileSync(process.execPath,[cli,'claim','build','--content',content,'--uri','https://documentdrivendx.github.io/weft/'+record.path,'--colo','.innsigle/colo.json','--issuer-id','weft','--issuer-name','Weft','--key-id',config.issuer.key_id,'--key-url',config.issuer.key_url,'--out',claim],{stdio:'inherit'});
 execFileSync(process.execPath,[cli,'sign','--claim',claim,'--key',key,'--out',attestation],{stdio:'inherit'});
 execFileSync(process.execPath,[cli,'verify','--attestation',attestation,'--content',content,'--keys',root+'/.well-known/innsigle/keys.json'],{stdio:'inherit'});
}
const {unlink}=await import('node:fs/promises');await unlink(root+'/.claim.json');
console.log('Signed and verified '+all.length+' site files. Claims and public key are verification metadata.');
