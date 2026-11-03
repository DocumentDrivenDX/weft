// SPDX-License-Identifier: MIT OR Apache-2.0
// Replay whole Rust results through the actual generated Node/WASM exports.
import {readFileSync} from 'node:fs';
import {createRequire} from 'node:module';
import {isDeepStrictEqual} from 'node:util';
const require=createRequire(import.meta.url);
const owner=require(process.argv[2]);
const cases=JSON.parse(readFileSync(process.argv[3],'utf8')).cases;
for (const [i,test] of cases.entries()) {
 const actual=JSON.parse(owner[test.name](...test.arguments));
 if (!isDeepStrictEqual(actual,test.expected)) throw Error(`Whole boundary result differs: ${i} ${test.name}`);
}
for (const bad of ['\ud800',42,null]) {
 let refused=false;
 try {owner.check_engine_temporal_read_shape_json(bad,'{}');} catch {refused=true;}
 if(!refused) throw Error('WASM boundary admitted a non-Unicode/non-string carrier');
}

console.log(JSON.stringify({scope:'Actual Node/WASM exported boundary only',cases:cases.length,passed:true}));
