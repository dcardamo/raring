// Offline native refresh cost, not a model of phone battery drain or Garmin radio cost.
// Seed synthetic demo data and enable status in the debug app first. No usable tokens.
import assert from 'node:assert/strict';
import {execFileSync} from 'node:child_process';
import {mkdirSync,writeFileSync} from 'node:fs';
const adb=(...args)=>execFileSync('adb',args,{encoding:'utf8'}).trim();
assert.match(process.env.ANDROID_SERIAL??'',/^emulator-(?!5560$)\d+$/,'select an isolated emulator');
const pkg='app.raring.debug',out='.data/status-widgets';
mkdirSync(out,{recursive:true});
assert.doesNotMatch(adb('shell','run-as',pkg,'ls','no_backup'),/tokens\.enc/,'never benchmark with Garmin credentials');
const snapshot=()=>JSON.parse(adb('shell','run-as',pkg,'cat','files/raring.status.json'));
assert.equal(snapshot().enabled,true,'enable status on synthetic demo data first');
adb('shell','am','force-stop',pkg);
adb('shell','dumpsys','battery','unplug');
try {
 const results=[];
 for(const count of [1,4,20]){
  adb('shell','dumpsys','batterystats','--reset');
  const times=[];
  for(let i=0;i<count;i++){
   const before=snapshot().generatedAt,start=performance.now();
   adb('shell','am','broadcast','-n',`${pkg}/app.raring.DebugSyncReceiver`,'-a','app.raring.DEBUG_STATUS');
   while(snapshot().generatedAt===before){
    assert.ok(performance.now()-start<10000,'native refresh completes');
    await new Promise(r=>setTimeout(r,50));
   }
   times.push(Math.round(performance.now()-start));
  }
  adb('shell','dumpsys','batterystats','--write');
  writeFileSync(`${out}/battery-${count}.txt`,adb('shell','dumpsys','batterystats',pkg));
  const pid=adb('shell','pidof',pkg);
  assert.ok(!adb('shell','cat','/proc/net/unix').includes(`webview_devtools_remote_${pid}`),'refresh never starts a WebView');
  results.push({refreshes:count,wallMs:times});
 }
 writeFileSync(`${out}/cadence.json`,JSON.stringify(results,null,2)+'\n');
 console.log(JSON.stringify(results));
} finally {adb('shell','dumpsys','battery','reset')}
