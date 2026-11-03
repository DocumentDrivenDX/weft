// SPDX-License-Identifier: MIT OR Apache-2.0
// Bounded full-result replay through actual web-target WASM in real Chromium.
import {createServer} from 'node:http';
import {readFileSync} from 'node:fs';
import {resolve} from 'node:path';
import {createRequire} from 'node:module';
const require=createRequire(import.meta.url);
const {chromium}=require('playwright');
const [webDirectory,corpusPath]=process.argv.slice(2);
const bodies=new Map([
 ['/weft_wasm.js',['text/javascript',readFileSync(resolve(webDirectory,'weft_wasm.js'))]],
 ['/weft_wasm_bg.wasm',['application/wasm',readFileSync(resolve(webDirectory,'weft_wasm_bg.wasm'))]],
 ['/corpus.json',['application/json',readFileSync(corpusPath)]],
 ['/', ['text/html',Buffer.from('<!doctype html><title>Weft temporal compiler component</title>')]],
]);
const server=createServer((request,response)=>{
 const body=bodies.get(new URL(request.url,'http://localhost').pathname);
 if(!body){response.writeHead(404).end();return;}
 response.writeHead(200,{'content-type':body[0]});response.end(body[1]);
});
await new Promise(resolve=>server.listen(0,'127.0.0.1',resolve));
let browser;
try {
 browser=await chromium.launch({headless:true});
 const context=await browser.newContext({timezoneId:'America/New_York'});
 const page=await context.newPage();const external=[];
 page.on('request',request=>{if(!request.url().startsWith('http://127.0.0.1:'))external.push(request.url());});
 await page.goto(`http://127.0.0.1:${server.address().port}/`);
 const result=await page.evaluate(async()=>{
  const owner=await import('/weft_wasm.js');await owner.default();
  const cases=(await (await fetch('/corpus.json')).json()).cases;
  for (const [i,test] of cases.entries()) {
   const actual=JSON.parse(owner[test.name](...test.arguments));
   if(JSON.stringify(actual)!==JSON.stringify(test.expected))throw Error(`Whole browser result differs: ${i} ${test.name}`);
  }
  for(const bad of ['\ud800',42,null]) {
   let refused=false;try{owner.check_engine_temporal_read_shape_json(bad,'{}');}catch{refused=true;}
   if(!refused)throw Error('Browser boundary admitted a non-Unicode/non-string carrier');
  }
  return {cases:cases.length,carrierRefusals:3,passed:true};
 });
 if(external.length)throw Error('Unexpected external request');
 console.log(JSON.stringify({scope:'Real Chromium compiler/export component only',browser:browser.version(),externalRequests:external.length,...result}));
} finally {
 if(browser)await browser.close();
 await new Promise(resolve=>server.close(resolve));
}
