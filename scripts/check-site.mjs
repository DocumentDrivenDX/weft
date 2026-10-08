import {chromium} from 'playwright';
import assert from 'node:assert/strict';
import {tmpdir} from 'node:os';
const base=process.env.SITE_TEST_URL || 'http://127.0.0.1:48193/';
const browser=await chromium.launch();
try {
 for(const viewport of [{width:1440,height:1000},{width:390,height:844}]){
  const page=await browser.newPage({viewport});const errors=[];
  page.on('pageerror',e=>errors.push(e.message));
  await page.goto(base);await page.locator('h1').waitFor();
  assert.equal(await page.locator('h1').count(),1);
  assert(await page.evaluate(()=>document.documentElement.scrollWidth<=innerWidth),'Viewport overflow');
  await page.locator('#tab-ashlar').click();
  assert.equal(await page.locator('#tab-ashlar').getAttribute('aria-selected'),'true');
  assert.match(await page.locator('#mapping').textContent(),/publication-aware/);
  await page.locator('#tab-ashlar').press('ArrowLeft');
  assert.equal(await page.locator('#tab-truss').getAttribute('aria-selected'),'true');
  assert.equal(await page.evaluate(()=>document.activeElement.id),'tab-truss');
  await page.locator('nav a[href="#backends"]').click();
  await page.waitForFunction(()=>document.querySelector('nav a[href="#backends"]').getAttribute('aria-current')==='location');
  assert.equal(errors.length,0,JSON.stringify(errors));
  await page.screenshot({path:tmpdir()+'/weft-site-'+viewport.width+'.png',fullPage:true});
  await page.close();
 }
 const response=await fetch(new URL('content-manifest.json',base));assert(response.ok);
 const manifest=await response.json();
 for(const file of manifest.files){
  const bytes=await fetch(new URL(file.path,base));assert(bytes.ok,file.path);
  const claim=await fetch(new URL(file.attestation,base));assert(claim.ok,file.attestation);
 }
 const page=await browser.newPage({javaScriptEnabled:false});await page.goto(base);
 await page.locator('nav a[href="#contract"]').click();assert.match(page.url(),/#contract$/);await page.close();
 console.log('Site checks passed: desktop/mobile, keyboard tabs, section navigation, no-JS anchors, no page errors/overflow and all signed-content URLs. Chromium '+browser.version());
} finally {await browser.close();}
