// SPDX-License-Identifier: MIT OR Apache-2.0
import {mkdir,copyFile,rm,readFile,writeFile} from 'node:fs/promises';
import {execFileSync} from 'node:child_process';

// The site shows the latest released version: the newest `v*` tag, or the
// manifest version when the checkout has no tags (for example a shallow clone).
function releasedVersion() {
 try {
  const tag=execFileSync('git',['describe','--tags','--abbrev=0','--match','v*'],{encoding:'utf8',stdio:['ignore','pipe','ignore']}).trim();
  if (/^v\d+\.\d+\.\d+$/.test(tag)) return tag.slice(1);
 } catch {}
 return null;
}
async function manifestVersion() {
 const text=await readFile('crates/weft-python/pyproject.toml','utf8');
 const match=text.match(/^version\s*=\s*"([^"]+)"/m);
 if (!match) throw new Error('no version in crates/weft-python/pyproject.toml');
 return match[1];
}
const version=releasedVersion() ?? await manifestVersion();

await rm('dist/site',{recursive:true,force:true});
await mkdir('dist/site/brand',{recursive:true});
for (const file of ['style.css','site.js','mark.svg','social-preview.png']) await copyFile('website/'+file,'dist/site/'+file);
const html=(await readFile('website/index.html','utf8')).replaceAll('{{WEFT_VERSION}}',version);
if (html.includes('{{')) throw new Error('unreplaced template token in index.html');
await writeFile('dist/site/index.html',html);
for (const file of ['brand-voice.md','design.md']) await copyFile('docs/helix/02-design/'+file,'dist/site/brand/'+file);
await copyFile('.innsigle/colo.json','dist/site/colophon.json');
console.log('Built static Weft microsite '+version+', including brand and design sources.');
