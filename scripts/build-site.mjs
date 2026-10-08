import {mkdir,copyFile,rm} from 'node:fs/promises';
await rm('dist/site',{recursive:true,force:true});
await mkdir('dist/site/brand',{recursive:true});
for (const file of ['index.html','style.css','site.js','mark.svg']) await copyFile('website/'+file,'dist/site/'+file);
for (const file of ['brand-voice.md','design.md']) await copyFile('docs/helix/02-design/'+file,'dist/site/brand/'+file);
await copyFile('.innsigle/colo.json','dist/site/colophon.json');
console.log('Built static Weft microsite, including brand and design sources.');
