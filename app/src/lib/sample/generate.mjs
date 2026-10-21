// Synthetic values transcribed from design/mockup/index.html; regenerate with node generate.mjs.
import { readFileSync, writeFileSync } from 'node:fs';
// Views whose per-day and per-activity detail is already stored.
const noDetail={missing:[],unavailable:[]};
const activity=(id,name,icon,start,duration,distance,load,zones,te,benefit,avgHr=null,intensityMin=null,vigorousMin=null)=>({id,icon,name,startLocal:`2025-10-${start}`,durationS:duration,distanceKm:distance,load,zoneSeconds:zones,aerobicTe:te,aerobicLabel:te?'Improving':null,anaerobicTe:te?1.2:null,anaerobicLabel:te?'Minor':null,benefit,avgHr,intensityMin,vigorousMin,distanceUnit:'km'});
const run=activity('9000000000001','Morning Run','run','16T06:12:00',3134,10.2,186,[190,585,1100,1050,209],3.8,'Tempo',152,62,24);
const lowerA=activity('9000000000002','Lower A','strength','15T17:12:00',2882,null,74,[840,1320,540,180,2],null,null,118,24,4);
const bike=activity('9000000000003','Evening Ride','bike','15T18:12:00',5050,38.4,246,[480,1800,1920,720,120],2.9,'Base',138,96,28);
const swim={...activity('9000000000007','Pool Swim','swim','09T07:05:00',1920,1.5,58,[300,900,600,120,0],2.4,'Base',132,30,5),distanceUnit:'m'};
const walk=activity('9000000000008','Lunch Walk','walk','07T12:10:00',2880,4.1,22,[1500,1200,180,0,0],1.1,'Recovery',104,null);
// Body Battery in three-minute slots to 13:30: charging overnight to 87 at 06:39, the run, then a slow drain to 78.
const batteryTimeline=Array.from({length:270},(_,i)=>{const m=i*3;return m<399?Math.round(25+62*m/399):m<445?null:Math.round(84-6*(m-445)/365)});
const vitals={readings:[['restingHr',58,'bpm',[48,68],[52,62],[57,56,58,55,56,58,57,58]],['hrv',58,'ms',[38,72],[48,62],[52,55,54,57,56,55,57,58]],['respiration',14.1,'brpm',[12,16.5],[13.2,15],[14.2,14,14.3,14.1,14,14.2,14.1,14.1]],['spo2',96,'%',[90,100],[94,98],[96,95,96,97,96,96,95,96]],['skinTemp',0.1,'°C',[-1,1],[-.4,.4],[0,-.1,.1,.2,0,-.1,0,.1]]].map(([kind,value,unit,scale,normal,trend])=>({kind,value,unit,scale,normal,trend:Array.from({length:30},(_,i)=>i===9||i===17?null:i===29?value:trend[i%trend.length]),tone:'good',status:'IN_RANGE',rangeSource:'garmin'})),offCount:0,baselineReady:true};
const age={recordedOn:'2025-10-12',fitnessAge:39,chronologicalAge:46,trend:[42,41.6,41.4,41,40.6,40.4,40,39.8,39.5,39.3,39.1,39],change12m:-3};
// Fitness Age's vigorous window to Thu 16 Oct: six seven-day blocks, the last holding the run's 24.
const vigorousWindow={end:'2025-10-16',weeks:[14,31,6,27,20,30],today:24,average:128/6,beforeToday:104/6,target:75,ageAtTarget:37.8};
const sleepPlan={needReadAt:'2025-10-16T12:58:00Z',night:'2025-10-17',bedtime:'23:06',bedtimeStart:'22:30',bedtimeSource:'garmin',inBed:'22:51',inBedAt:'2025-10-16T22:51:00',fallAsleepMin:15,inBedMin:485,wakeAt:'2025-10-17T06:56:00',windDown:'22:00',windDownLeadMin:30,windDownAt:'2025-10-16T22:00:00',bedtimeStartAt:'2025-10-16T22:30:00',bedtimeAt:'2025-10-16T23:06:00',wakeTime:'06:36',needMin:470,source:'tonight',baselineMin:450,adjustments:[{factor:'Training',detail:"today's activity and load"}],reason:"Garmin added 20 min for today's activity and load.",tip:"Your sleep lines up with Garmin's optimal window. Keep waking near 06:36.",wakeNights:14,stale:false};
// Garmin's optimal window is a dashed target, distinct from the shaded usual windows; one later night reads as Behind.
const optimal=[-75,375];
const alignmentHistory=[[10,22.7,6.5],[11,23,6.7],[12,24,7.1],[13,22.6,6.5],[14,22.9,6.8],[15,22.7,6.6],[16,22.52,6.65]].map(([d,bed,wake])=>({date:`2025-10-${d}`,status:d===12?'Behind':'Aligned',tone:d===12?'fair':'good',window:optimal,sleep:[Math.round((bed-24)*60),Math.round(wake*60)]}));
const alignment={state:'current',latest:alignmentHistory[6],detail:"Last night lined up with Garmin's optimal window.",history:alignmentHistory};
const timing={nights:[[13,22.6,6.5],[14,22.9,6.8],[15,22.7,6.6],[16,22.52,6.65],[17,23.4,7.2],[18,0.5,8.4],[19,23.1,7]].map(([d,bedHour,wakeHour])=>({date:`2025-10-${d}`,bedHour,wakeHour})),bedRange:['22:31','00:30'],wakeRange:['06:30','08:24'],wakeSpreadMin:114,previousWakeSpreadMin:52,aligned:5,ahead:0,behind:2,focus:'Garmin rated most nights aligned. Keep waking near 06:48.'};
// Three-minute readings to 13:30: calm sleep, a run Garmin cannot rate, an off-wrist gap and a busier late morning.
const stressTimeline=Array.from({length:270},(_,i)=>{const m=i*3;return m<372?Math.round(13+Math.sin(i*.35)*4):m<445?null:m<600?Math.round(32+Math.sin(i*.2)*7):Math.round(50+Math.sin(i*.13)*16+Math.sin(i*.9)*4)});
const stress={average:31,max:88,level:'low',qualifier:'Balanced',tone:'good',timeline:stressTimeline,active:[[372,426]],sleep:[0,399],workouts:[[372,424]],time:[['rest',25920],['low',10800],['medium',5400],['high',1260]].map(([level,seconds])=>({level,seconds})),activeS:3240,trend:Array.from({length:30},(_,i)=>i===11?null:i===29?31:27+Math.round(Math.sin(i*.45)*4))};
// Weekly plan: Mon 13 – Sun 19 Oct, viewed Thu 16; rest Mon and Fri, so two of five training days have passed.
const item=(kind,value,target,pace,state,note,unavailable=null)=>({kind,value,target,completion:Math.min(value/target,1),pace,state,note,unavailable});
const planWeek=(start,end,daysLeft,items,headline,detail,paused=false)=>{const reached=items.filter(i=>i.state==='done').length;return {start,end,paused,daysLeft,items,total:items.length,onTrack:items.filter(i=>i.state!=='behind').length,reached,headline,detail,completion:items.reduce((a,i)=>a+i.completion,0)/items.length}};
const thisWeek=planWeek('2025-10-13','2025-10-19',3,[item('strength',2,3,.4,'onTrack','Tue, Wed'),item('intensity',124,150,.4,'onTrack',"Garmin's weekly goal · 26 min to go"),item('zone2',30,90,.4,'behind','60 min to go'),item('asleepOnTime',3,5,4/7,'onTrack','Mon, Tue, Thu')],'On track','3 days left · Zone 2 is behind');
const lastWeek=planWeek('2025-10-06','2025-10-12',0,[item('strength',3,3,1,'done','Tue, Thu, Sat'),item('intensity',162,150,1,'done',"Garmin's weekly goal · Reached"),item('zone2',72,90,1,'behind','18 min to go'),item('asleepOnTime',4,5,1,'behind','Mon, Tue, Wed, Sat')],'Partly reached','Zone 2 and Asleep on time came up short');
const preset=(preset,strengthSessions,zone2Min,asleepNights)=>({preset,strengthSessions,intensityMin:null,zone2Min,asleepNights,restDays:[]});
const plan={configured:true,targets:{...preset('strength',3,90,5),restDays:[0,4]},garminIntensityGoal:150,weekStartsOn:0,week:thisWeek,lastWeek,history:[['08-25',.7],['09-01',.91],['09-08',1],['09-15',null],['09-22',.76],['09-29',.99],['10-06',.9],['10-13',thisWeek.completion]].map(([d,completion],i)=>({start:`2025-${d}`,completion,partial:false,paused:completion===null,current:i===7})),presets:[preset('strength',3,90,5),preset('endurance',1,150,5),preset('sleep',2,60,6)]};
const planCheckin={week:{...lastWeek,start:'2025-10-13',end:'2025-10-19'},previous:[3,150,85,5],next:'Same targets · rest Mon and Fri'};
const why=(key,name,value,rating,tone,detail)=>({key,name,value,rating,tone,detail});
// What the core's explain rules produce for this synthetic day.
const readinessWhy={title:'Why readiness is High',summary:'Recovery time and sleep are helping readiness most.',feedback:'Well recovered',source:"From Garmin's readiness factors, worst first. Raring adds no score.",inputs:[
 why('acuteLoad','Acute load','612','Moderate','fair',"Your last 7 days of training load against what you're used to."),
 why('stressHistory','Stress history',null,'Good','good','Your stress over the last three days.'),
 why('hrv','HRV','53 ms','Good','good','Your 7-day HRV average against your baseline.'),
 why('sleepHistory','Sleep history',null,'Good','good','Your sleep over the last three nights against your need.'),
 why('sleep','Sleep','84','Good','good',"Last night's sleep score."),
 why('recovery','Recovery time','18 h','Very good','good','Hours until Garmin expects you to be recovered from recent workouts.')]};
const bodyBatteryWhy={title:'Why Body Battery is 78',summary:'Sleep charged it by 62, to 87 at wake. Since then: Morning Run and a balanced day.',feedback:'Balanced and active so far, with exercise',source:"From Garmin's Body Battery events and feedback.",inputs:[
 why('overnight','Overnight','+62',null,'good','Charge while you slept, 22:31–06:39; 87 at wake.'),
 why('stress','Stress','31','Balanced','good',"Today's average. Stress drains Body Battery while you're awake and still."),
 why('activity','Morning Run','−9','Improving tempo','neutral','06:12 · 52 min')]};
const statusWhy={title:'Why Productive',summary:'Your load is building fitness. Load focus shows an anaerobic shortage.',feedback:null,source:"From Garmin's training status inputs.",inputs:[
 why('loadFocus','Load focus',null,'Anaerobic shortage','fair','How your last 4 weeks split across low aerobic, high aerobic and anaerobic.'),
 why('acuteLoad','Acute load','612 · 1.1× your 4-week load','Optimal','good','Your last 7 days of load against your 4-week load.'),
 why('hrvStatus','HRV status','53 ms','Balanced','good','Your 7-day HRV average against your baseline.'),
 why('vo2max','VO₂ max','52',null,'neutral',"Garmin's fitness estimate. +1 in 30 days.")]};
const nightVitals={sleepHr:{value:58,unit:'bpm',usual:[52,62],rangeSource:'garmin',state:'within',delta:null,tone:'good'},hrv:{value:58,unit:'ms',usual:[48,62],rangeSource:'garmin',state:'within',delta:null,tone:'good'}};
const today={date:'2025-10-16',watchLastUpload:null,watchSleepUploadStale:false,missingMetricReasons:{},sleepPlan,readiness:{value:82,max:100,label:'High',tone:'good',band:null},readinessTrend:[67,72,74,69,64,71,76,79,82,78,73,68,70,76,80,84,77,74,79,83,85,81,78,75,80,86,83,79,84,82],sleep:{value:84,max:100,label:'Good',tone:'good',band:null},load:{value:612,max:1000,label:'Optimal',tone:'good',band:[420,760]},headline:'Ready to push',factors:[['sleep','Sleep',86,'good'],['recovery','Recovery time',92,'good'],['hrv','HRV',78,'good'],['acuteLoad','Acute load',58,'fair'],['sleepHistory','Sleep history',81,'good'],['stressHistory','Stress history',73,'good']].map(([key,name,percent,tone])=>({key,name,percent,tone,detail:`${name} is ${tone==='fair'?'the main limit on':'supporting'} your readiness today.`})),readinessWhy,bodyBatteryWhy,reconcile:null,call:'push',callNote:'Morning run done · 186 load',bodyBattery:{now:78,charged:62,drained:18,timeline:batteryTimeline},stress,hrv:{lastNight:58,status:'Balanced',tone:'good',baseline:[48,62],week:[51,49,53,55,52,56,58]},vitals,activities:[run],fitnessAge:age,checkinReady:true,plan:thisWeek,planned:{state:'available',session:{name:'Easy Run',date:'2025-10-17'},coach:'Garmin Coach 10K',coverageStart:'2025-10-16',coverageEnd:'2025-10-30'},lead:null,adapt:null,nightVitals,
 // The run's vigorous minutes moved the six-week average: the card follows This week.
 fitness:{window:vigorousWindow,age:39,droppedFrom:null,card:'afterWeek',leverLine:false},
 // Shortly after waking: last night leads, then the limiting factor and the effort; tonight's plan is one line.
 focus:{phase:'wake',cards:['focus','day','sleepPlan'],until:'2025-10-16T09:39:00',effort:true,planOpen:false,sleepMissing:false,lastNight:{asleepMin:468,needMin:450,woke:'06:39'},note:null,recoveryHours:18,schedule:null}};
const stages=[['light',12],['deep',38],['light',22],['rem',14],['light',30],['deep',32],['light',18],['awake',4],['rem',22],['light',34],['deep',18],['light',26],['rem',28],['awake',6],['light',40],['deep',8],['light',24],['rem',30],['light',28],['awake',5],['rem',26],['light',18],['awake',5]];
let offset=0;const segments=stages.map(([stage,minutes])=>{let x={stage,startMin:offset,minutes};offset+=minutes;return x});
const sleep={detail:noDetail,date:'2025-10-16',startLocal:'2025-10-15T22:31:00',endLocal:'2025-10-16T06:39:00',score:today.sleep,asleepMin:468,needMin:450,headline:'A good night',stages:segments,totals:[['deep',96,12],['light',252,-4],['rem',120,9],['awake',20,-6]].map(([stage,minutes,usualDeltaMin])=>({stage,minutes,usualDeltaMin,comparisonMin:minutes-usualDeltaMin,comparisonNights:stage==='awake'?26:28})),overnight:[['hrv',58,'ms',[49,54,57,61,60,58,56,59]],['heartRate',46,'bpm',[52,50,48,47,46,46,47,46,48,47,49,51]],['respiration',14.1,'brpm',[14.4,14.2,14,13.9,14.1,14,14.2,14.1]],['spo2',96,'%',[97,96,95,96,96,95,96,96]]].map(([kind,value,unit,series])=>({kind,value,unit,series})),consistency:[[22.6,6.5],[23.1,6.7],[22.9,6.6],[23.6,7],[22.5,6.4],[23.3,6.9],[22.8,6.6],[22.7,6.5],[23,6.7],[24,7.1],[22.6,6.5],[22.9,6.8],[22.7,6.6],[22.52,6.65]].map(([bedHour,wakeHour],i)=>({date:`2025-10-${String(i+3).padStart(2,'0')}`,bedHour,wakeHour})),consistencySpreadMin:18,bedWindow:[22.5,23],wakeWindow:[6.5,7],garminAverages:{bedtime:'22:55',wakeTime:'06:42',nights:7,through:'2025-10-16'},alignment,sleepPlan};
// Seven nights against Garmin's need, the 12th unrecorded; Sleep opens Last night in the morning.
sleep.recent=[[10,452,450],[11,468,455],[12,null,null],[13,431,460],[14,476,450],[15,440,465],[16,468,450]].map(([d,asleepMin,needMin])=>({date:`2025-10-${d}`,asleepMin,needMin}));sleep.tonightFirst=false;sleep.woundDown=false;sleep.windDownOpen=false;
sleep.napsState='available';sleep.naps=[{startLocal:'2025-10-15T23:45:00',endLocal:'2025-10-16T00:15:00',durationMin:30,feedback:null},{startLocal:'2025-10-16T13:10:00',endLocal:'2025-10-16T13:35:00',durationMin:25,feedback:null},{startLocal:null,endLocal:'2025-10-16T16:30:00',durationMin:15,feedback:null}];
const training={acute:today.load,status:'Productive',statusTone:'good',statusWhy,headline:'In your optimal range',chips:[{icon:'flame',value:'1.1',text:'Load ratio',tone:'fair'},{icon:'clock',value:'18h',text:'Recovery',tone:'neutral'}],week:[0,142,210,0,95,320,186].map((load,i)=>({date:`2025-10-${String(i+10).padStart(2,'0')}`,load})),loadFocus:[['lowAerobic','Low aerobic',450,[300,600],false],['highAerobic','High aerobic',610,[400,800],false],['anaerobic','Anaerobic',60,[150,350],true]].map(([key,name,value,target,shortage])=>({key,name,value,target,shortage})),vo2max:52,vo2maxChange30d:1,vo2maxTrend:[49,49,50,50,50,51,51,52],races:[['5K',1300],['10K',2710],['Half marathon',5990],['Marathon',12665]].map(([label,seconds])=>({label,seconds})),enduranceScore:6800,hillScore:74,runningToleranceKm:63,thresholds:[{kind:'runningHr',value:164,unit:'bpm',recordedOn:'2025-10-13',stale:false},{kind:'runningPace',value:264,unit:'s/km',recordedOn:'2025-10-13',stale:false},{kind:'runningPower',value:280,unit:'W',recordedOn:'2025-10-12',stale:false},{kind:'cyclingFtp',value:230,unit:'W',recordedOn:'2025-10-10',stale:false},{kind:'heat',value:0,unit:'%',recordedOn:'2025-10-15',stale:false},{kind:'altitude',value:850,unit:'m',recordedOn:'2025-10-14',stale:false}],workouts:[run,bike,lowerA,swim,walk],planned:today.planned,race:{name:'City 10K',date:'2025-10-28',daysUntil:12},raceState:'available'};
training.gear={unit:'km',fetchedAt:'2025-10-16T10:30:00Z',items:[
 {id:'sample-shoes',name:'Road shoes',kind:'Shoes',status:'Active',distance:420,threshold:650},
 {id:'sample-bike',name:'City bike',kind:'Bike',status:'Retired',distance:1240,threshold:null},
 {id:'sample-spare',name:'Spare shoes',kind:'Shoes',status:'Active',distance:0,threshold:null}]};
const workout={detail:noDetail,summary:run,recoveryHours:18,stats:[['distance','Distance',10.2,'km'],['duration','Time',3134,'s'],['pace','Pace',307,'s/km'],['avgHr','Avg HR',152,'bpm'],['maxHr','Max HR',171,'bpm'],['calories','Calories',712,'kcal']].map(([key,label,value,unit])=>({key,label,value,unit})),route:[[45.421,-75.697],[45.423,-75.699],[45.426,-75.697],[45.428,-75.693],[45.429,-75.688],[45.427,-75.683],[45.424,-75.681],[45.421,-75.683],[45.42,-75.688],[45.422,-75.692],[45.421,-75.697]],heartRate:Array.from({length:210},(_,i)=>({tS:i*15,bpm:Math.round((i<32?112+i*1.3:i>184?160-(i-184)*1.1:Math.floor((i-32)/25)%2?149:164)+Math.sin(i*1.9)*2)})),zoneFloors:[120,138,154,167],splits:[321,308,302,298,305,295,301,297,310,318].map((seconds,i)=>({index:i+1,distanceKm:1,seconds,avgHr:147+i*2}))};
// Stored streams on the HR's 15 s axis (a sensor gap at 20–21 min), summary fields this run has, and Garmin's effects.
workout.along={tS:workout.heartRate.map(s=>s.tS),series:[
 {kind:'speed',unit:'km/h',values:workout.heartRate.map((s,i)=>i===80||i===81?null:Math.round((11.2+Math.sin(i/9)*.8+(s.bpm-150)/40)*10)/10)},
 {kind:'elevation',unit:'m',values:workout.heartRate.map((_,i)=>Math.round(70+Math.sin(i/30)*9))},
 {kind:'cadence',unit:'spm',values:workout.heartRate.map((_,i)=>172+Math.round(Math.sin(i/7)*3))},
 // Running form follows the pace; ground contact drops out with the speed sensor gap.
 {kind:'strideLength',unit:'cm',values:workout.heartRate.map((s,i)=>Math.round((108+Math.sin(i/9)*5+(s.bpm-150)/4)*10)/10)},
 {kind:'groundContactTime',unit:'ms',values:workout.heartRate.map((s,i)=>i===80||i===81?null:Math.round(240-Math.sin(i/9)*8-(s.bpm-150)/2))},
 {kind:'verticalOscillation',unit:'cm',values:workout.heartRate.map((_,i)=>Math.round((8.4+Math.sin(i/11)*.3)*10)/10)},
 {kind:'verticalRatio',unit:'%',values:workout.heartRate.map((_,i)=>Math.round((7.6+Math.sin(i/11)*.3-Math.sin(i/9)*.2)*10)/10)}]};
const stat=([key,label,value,unit])=>({key,label,value,unit});
workout.more=[['moderateIntensityMinutes','Moderate intensity',14,'min'],['vigorousIntensityMinutes','Vigorous intensity',24,'min'],['differenceBodyBattery','Body Battery',-9,'points'],
 ['beginPotentialStamina','Stamina at start',100,'%'],['endPotentialStamina','Stamina at end',68,'%'],['minAvailableStamina','Lowest available stamina',66,'%']].map(stat);
// Garmin's other fields in the core's groups: pace in s/km, stride and bounce in cm.
workout.recorded=[['time',[['movingDuration','Moving time',3101,'s']]],
 ['pace',[['averageMovingSpeed','Moving pace',304,'s/km'],['maxSpeed','Best pace',251,'s/km'],['avgGradeAdjustedSpeed','Grade-adjusted pace',302,'s/km']]],
 ['form',[['steps','Steps',8920,''],['maxRunCadence','Max cadence',184,'spm'],['strideLength','Stride length',113.4,'cm'],['groundContactTime','Ground contact time',238,'ms'],['verticalOscillation','Vertical oscillation',8.4,'cm'],['verticalRatio','Vertical ratio',7.4,'%']]],
 ['elevation',[['elevationLoss','Descent',41,'m'],['minElevation','Lowest',61,'m'],['maxElevation','Highest',79,'m']]],
 ['heartRate',[['recoveryHeartRate','Recovery heart rate',31,'bpm']]],
 ['energy',[['bmrCalories','Resting calories',96,'kcal'],['waterEstimated','Estimated sweat loss',820,'ml']]]].map(([group,fields])=>({group,fields:fields.map(stat)}));
workout.selfEvaluation={feel:'strong',effort:6};
workout.effects=[{kind:'aerobic',value:3.8,band:'Improving',line:'builds your aerobic base',garminLabel:'Improving'},{kind:'anaerobic',value:1.2,band:'Minor benefit',line:'little sprint or hard work',garminLabel:'Minor'}];
workout.vo2maxNew=null;
// The run's street map, drawn by the core from OpenFreeMap tiles: `raring route-map --route` with this route.
// Map data © OpenStreetMap contributors (ODbL) and © OpenMapTiles; the card shows the credit.
workout.map=JSON.parse(readFileSync(new URL('workoutMap.json',import.meta.url),'utf8'));
// A pool swim (25 m lengths, 100 m splits, swim fields as Garmin's summary names them) and a walk without streams.
const swimHr=Array.from({length:128},(_,i)=>({tS:i*15,bpm:Math.round(118+Math.min(i,40)*.5+Math.sin(i/3)*4)}));
const otherWorkouts={
 [swim.id]:{detail:noDetail,summary:swim,recoveryHours:8,stats:[['distance','Distance',1.5,'km'],['duration','Time',1920,'s'],['pace','Pace',1280,'s/km'],['averageHR','Avg HR',132,'bpm'],['maxHR','Max HR',151,'bpm'],['calories','Calories',310,'kcal']].map(([key,label,value,unit])=>({key,label,value,unit})),
  route:[],heartRate:swimHr,zoneFloors:[120,138,154,167],splits:[124,126,125,128,127,129,126,130,128,131,129,132,130,127,124].map((seconds,i)=>({index:i+1,distanceKm:.1,seconds,avgHr:124+i})),
  along:{tS:swimHr.map(s=>s.tS),series:[{kind:'speed',unit:'km/h',values:swimHr.map((_,i)=>i%25===24?null:Math.round((2.85+Math.sin(i/5)*.12)*100)/100)}]},
  more:[{key:'moderateIntensityMinutes',label:'Moderate intensity',value:20,unit:'min'},{key:'vigorousIntensityMinutes',label:'Vigorous intensity',value:5,unit:'min'}],
  recorded:[{group:'strokes',fields:[{key:'poolLength',label:'Pool length',value:25,unit:'m'},{key:'activeLengths',label:'Lengths',value:60,unit:''},{key:'averageSwolf',label:'Avg SWOLF',value:38,unit:''},{key:'averageStrokes',label:'Strokes per length',value:16,unit:''},
   {key:'averageStrokeCadence',label:'Stroke rate',value:28,unit:'strokes/min'},{key:'averageStrokeDistance',label:'Distance per stroke',value:1.6,unit:'m'}]}],selfEvaluation:null,
  effects:[{kind:'aerobic',value:2.4,band:'Maintaining',line:'keeps your aerobic fitness',garminLabel:'Improving'},{kind:'anaerobic',value:1.2,band:'Minor benefit',line:'little sprint or hard work',garminLabel:'Minor'}],
  vo2maxNew:null,counted:{week:'2025-10-06',goals:true,lines:[],planItem:null,bests:[],fitness:null},strength:null},
 [walk.id]:{detail:noDetail,summary:walk,recoveryHours:null,stats:[['distance','Distance',4.1,'km'],['duration','Time',2880,'s'],['pace','Pace',702,'s/km'],['averageHR','Avg HR',104,'bpm']].map(([key,label,value,unit])=>({key,label,value,unit})),
  route:[],heartRate:Array.from({length:192},(_,i)=>({tS:i*15,bpm:Math.round(98+Math.sin(i/11)*6)})),zoneFloors:[120,138,154,167],splits:[[1,690],[1,701],[1,708],[1,712],[.1,69]].map(([distanceKm,seconds],i)=>({index:i+1,distanceKm,seconds,avgHr:102+i})),
  along:null,more:[],recorded:[{group:'time',fields:[{key:'movingDuration',label:'Moving time',value:2830,unit:'s'}]}],selfEvaluation:null,effects:[{kind:'aerobic',value:1.1,band:'Minor benefit',line:'easy movement',garminLabel:'Improving'}],vo2maxNew:null,counted:{week:'2025-10-06',goals:true,lines:[],planItem:null,bests:[],fitness:null},strength:null}};
for(const w of Object.values(otherWorkouts))w.map=null;
writeFileSync(new URL('otherWorkouts.json',import.meta.url),JSON.stringify(otherWorkouts,null,2)+'\n');
workout.counted={week:'2025-10-13',goals:true,lines:[{kind:'zone2',amount:18,position:null,recorded:100,target:150}],planItem:null,bests:['Longest run'],fitness:{minutes:24,week:30,thisWeek:true,target:75,until:'2025-11-26'}};
const health={detail:noDetail,nightVitals,date:today.date,bodyWeight:{value:72.4,unit:'kg',measuredOn:'2025-10-14',bmi:23.8,bodyFat:null,trend:[71.8,null,72.1,null,72.6,72.3,null,72.5,72.4,null,72.2,72.4]},fitnessAge:age,lever:{key:'vigorousMinutes',name:'Vigorous minutes',current:17.3,target:75,unit:'min/wk',ageIfMet:37.8},vigorous:vigorousWindow,stress,intensity:{start:'2025-10-13',end:'2025-10-19',through:'2025-10-16',total:124,moderate:64,vigorous:30,goal:150},drivers:[['vo2max','VO₂ max',52,'',55,true,'good'],['restingHr','Resting HR',46,'bpm',54,true,'good'],['bmi','BMI',23.8,'',25,true,'good'],['vigorousMinutes','Vigorous',17.3,'min/wk',75,false,'fair']].map(([key,name,value,unit,target,met,tone])=>({key,name,value,unit,target,met,tone})),vitals};
health.journalEnabled=true;health.restingHrRange=[55,58];health.journal=[{date:'2025-10-16',state:'available',entries:[{name:'Stretching',value:'Yes'},{name:'Morning caffeine',value:'2 coffee'},{name:'Other reading',value:null}]},{date:'2025-10-15',state:'empty',entries:[]},{date:'2025-10-14',state:'unavailable',entries:[]}];
const checkin={weekStart:'2025-10-13',timing,cards:[['readiness','Readiness',82,'',76,'good',[70,74,82,78,80,84,82]],['sleep','Sleep Score',84,'',79,'good',[76,81,82,79,84,83,84]],['training','Acute load',612,'load',520,'neutral',[580,596,610,604,615,630,612]],['health','Fitness Age',39,'years',40,'neutral',[40,40,40,40,39,39,39]]].map(([area,headline,value,unit,previous,tone,series])=>({area,headline,value,unit,previous,tone,series})),vo2maxChange:1,fitnessAgeChange:-1,win:'Readiness up 6 on the week before, averaging 82',plan:planCheckin,intensity:{start:'2025-10-13',end:'2025-10-19',through:'2025-10-19',total:162,moderate:62,vigorous:50,goal:150},focus:"Garmin's load focus: anaerobic was furthest below its range"};
const status={signedIn:true,syncing:false,lastSync:'2025-10-16T06:58:00',historyDays:90,historyFrom:'2019-03-02',historyComplete:true,lastError:null};
// Strength: the mockup's Lower A session and twelve weeks of history (kilograms, Monday weeks).
const MUSCLES=['chest','back','shoulders','arms','core','hips','quads','hamstrings','glutes','calves'];
const muscleSets=o=>MUSCLES.map(muscle=>({muscle,sets:o[muscle]??0}));
const musclesThisWeek={chest:3,back:6,shoulders:3,arms:2,core:0,hips:1,quads:6,hamstrings:6,glutes:9,calves:3};
const avg4={chest:8,back:11,shoulders:7,arms:5,core:4,hips:2,quads:8,hamstrings:7,glutes:10,calves:4};
training.strength={figure:'female',weekStart:'2025-10-13',sets:29,sessionsWithSets:2,strengthActivities:3,records:1,muscles:muscleSets(musclesThisWeek)};
workout.strength=null;
const lift=(weight,reps)=>({weight,reps});
const sset=(reps,weight,flags=[],record=false)=>({reps,weight,durationS:reps*3+12,flags,record});
const exercises=[
 {key:'SQUAT/BARBELL_BACK_SQUAT',name:'Barbell Back Squat',muscles:['quads','glutes'],sets:[sset(8,60),sset(5,100,[],true),sset(5,100),sset(5,100)],lastDate:'2025-10-08',lastSets:[lift(60,8),lift(97.5,5),lift(97.5,5),lift(97.5,5)],record:lift(100,5)},
 {key:'DEADLIFT/ROMANIAN_DEADLIFT',name:'Romanian Deadlift',muscles:['hamstrings','glutes'],sets:[sset(8,80),sset(8,80),sset(0,80,['zeroReps']),sset(8,80)],lastDate:'2025-10-08',lastSets:[lift(80,8),lift(80,8),lift(80,7)],record:null},
 {key:'LUNGE/DUMBBELL_WALKING_LUNGE',name:'Dumbbell Walking Lunge',muscles:['quads','glutes'],sets:[sset(12,16),sset(24,16,['doubled'])],lastDate:'2025-10-08',lastSets:[lift(16,12),lift(16,12),lift(16,12)],record:null},
 {key:null,name:null,muscles:[],sets:[sset(12,40,['unnamed']),sset(12,40,['unnamed']),sset(10,40,['unnamed'])],lastDate:null,lastSets:[],record:null},
 {key:'CALF_RAISE/STANDING_CALF_RAISE',name:'Standing Calf Raise',muscles:['calves'],sets:[sset(15,40),sset(15,40),sset(14,40)],lastDate:'2025-10-08',lastSets:[lift(40,15),lift(40,15),lift(40,15)],record:null},
];
const strengthSummary=lowerA;
const strengthWorkout={detail:noDetail,counted:{week:'2025-10-13',goals:true,lines:[{kind:'strength',amount:1,position:2,recorded:3,target:4}],planItem:'Strength',bests:[],fitness:{minutes:4,week:30,thisWeek:true,target:75,until:'2025-11-25'}},along:null,more:[],recorded:[],selfEvaluation:{feel:null,effort:7},effects:[],vo2maxNew:null,summary:strengthSummary,recoveryHours:14,stats:[['duration','Time',2882,'s'],['avgHr','Avg HR',118,'bpm'],['maxHr','Max HR',151,'bpm'],['calories','Calories',312,'kcal']].map(([key,label,value,unit])=>({key,label,value,unit})),route:[],heartRate:Array.from({length:192},(_,i)=>({tS:i*15,bpm:Math.round(104+Math.abs(Math.sin(i/9))*34+Math.sin(i*1.7)*3)})),zoneFloors:[120,138,154,167],splits:[],strength:{unit:'kg',figure:'female',checkedAt:'2025-10-15T19:30:00Z',sets:16,reps:161,unknownRepSets:0,restS:1520,records:1,setsRecorded:true,zeroRepSets:1,missingRepSets:0,doubledSets:1,unnamedSets:3,muscles:muscleSets({quads:6,glutes:9,hamstrings:3,calves:3}),exercises}};
strengthWorkout.map=null;
const mondays=Array.from({length:12},(_,i)=>{const d=new Date(Date.UTC(2025,6,28+i*7));return d.toISOString().slice(0,10)});
const weekly=mondays.map((weekStart,i)=>({weekStart,muscles:i===11?muscleSets(musclesThisWeek):muscleSets(Object.fromEntries(MUSCLES.map((m,j)=>[m,i===5?Math.round(avg4[m]*.3):Math.max(0,Math.round(avg4[m]+Math.sin(i*1.3+j)*2.5))])))}));
const last4=Object.fromEntries(MUSCLES.map((m,j)=>[m,weekly.slice(8).reduce((a,w)=>a+w.muscles[j].sets,0)]));
const squatTop=[85,87.5,87.5,90,90,90,92.5,95,95,97.5,97.5,100],squatReps=[5,5,5,5,5,4,5,5,5,5,5,5];
const sessionsOf=(tops,reps,repsPerSet=3)=>tops.map((w,i)=>({activityId:String(9000000000100+i),date:new Date(Date.UTC(2025,6,30+i*7)).toISOString().slice(0,10),top:lift(w,reps[i]),reps:Array(repsPerSet).fill(reps[i]),record:i===tops.length-1})).reverse();
const bests=(rows)=>[1,3,5,8,10].map((reps,i)=>({reps,weight:rows[i]?.[0]??null,date:rows[i]?.[1]??null}));
const hist=(key,name,muscles,tops,reps,b)=>({key,name,muscles,sessions:sessionsOf(tops,reps),bests:bests(b),repsBest:null});
// Bodyweight work has no weight; its best is the most reps in one set.
const bodyweight=(key,name,muscles,reps,repsBest)=>({key,name,muscles,repsBest,
 sessions:reps.map((r,i)=>({activityId:String(9000000000100+i),date:new Date(Date.UTC(2025,6,30+i*7)).toISOString().slice(0,10),top:null,reps:Array(3).fill(r),record:false})).reverse(),
 bests:[1,3,5,8,10].map(n=>({reps:n,weight:null,date:null}))});
// Eleven strength workouts: two this week with sets, one timed without lap presses, and weekly sessions back to August.
const strengthWorkouts=[
 {activity:strengthSummary,setsRecorded:true},
 {activity:activity('9000000000004','Upper B','strength','14T07:10:00',3120,null,68,[700,1500,600,200,0],null,null),setsRecorded:true},
 {activity:activity('9000000000005','Strength','strength','13T12:30:00',2400,null,31,[1500,800,100,0,0],null,null),setsRecorded:false},
 ...Array.from({length:8},(_,i)=>{const d=new Date(Date.UTC(2025,9,8-i*7));const id=String(9000000000110-i);return {activity:{...activity(id,i%2?'Upper A':'Lower A','strength','01T18:05:00',2700+i*30,null,60+i,[800,1300,500,150,0],null,null),startLocal:`${d.toISOString().slice(0,10)}T18:05:00`},setsRecorded:true}}),
];
const strength={unit:'kg',figure:'female',date:'2025-10-16',weekStart:'2025-10-13',thisWeek:muscleSets(musclesThisWeek),average4w:muscleSets(avg4),last4w:muscleSets(last4),weeks:weekly,sessionsWithSetsThisWeek:2,exercises:[
 hist('SQUAT/BARBELL_BACK_SQUAT','Barbell Back Squat',['quads','glutes'],squatTop,squatReps,[null,[105,'2025-08-20'],[100,'2025-10-15'],[90,'2025-09-10'],[85,'2025-08-06']]),
 hist('BENCH_PRESS/BARBELL_BENCH_PRESS','Barbell Bench Press',['chest'],[62.5,62.5,65,65,65,67.5,67.5,67.5,70,70,70,70],Array(12).fill(12),[[90,'2025-09-02'],[85,'2025-08-12'],[80,'2025-09-23'],[75,'2025-09-30'],[72.5,'2025-10-07']]),
 hist('DEADLIFT/ROMANIAN_DEADLIFT','Romanian Deadlift',['hamstrings','glutes'],[70,70,72.5,72.5,75,75,75,77.5,77.5,80,80,80],Array(12).fill(8),[null,null,[90,'2025-09-17'],[80,'2025-10-08'],[75,'2025-08-27']]),
 hist('ROW/ONE_ARM_DUMBBELL_ROW','One Arm Dumbbell Row',['back','arms'],[26,26,26,28,28,28,28,30,30,30,30,30],Array(12).fill(12),[null,null,[34,'2025-09-09'],[32,'2025-09-30'],[30,'2025-10-14']]),
 hist('CALF_RAISE/STANDING_CALF_RAISE','Standing Calf Raise',['calves'],[30,30,32.5,32.5,35,35,35,37.5,37.5,40,40,40],Array(12).fill(15),[null,null,null,[45,'2025-09-24'],[42.5,'2025-10-01']]),
 bodyweight('CORE/BICYCLE_CRUNCH','Bicycle Crunch',['core'],[18,18,20,20,20,22,22,22,24,24,24,26],{reps:26,date:'2025-09-15'}),
],workouts:strengthWorkouts};

// ID-matched workout views for every strength history link. Build their sets
// from the same session rows as the history, not from the canonical Lower A.
const strengthWorkoutHistory={};
const historyIds=new Set([...strength.workouts.map(w=>w.activity.id),...strength.exercises.flatMap(e=>e.sessions.map(s=>s.activityId))]);
for(const id of historyIds){
 if(id===strengthWorkout.summary.id)continue;
 const listed=strength.workouts.find(w=>w.activity.id===id);
 const matches=strength.exercises.flatMap(e=>{const index=e.sessions.findIndex(s=>s.activityId===id);return index<0?[]:[{e,index,s:e.sessions[index]}]});
 const summary=listed?.activity??{...activity(id,'Strength session','strength','01T18:05:00',2700,null,60,[],null,null),startLocal:`${matches[0].s.date}T18:05:00`};
 const recorded=listed?.setsRecorded??true;
 // The standalone Upper B list entry has no chart row; give it its own sets.
 const rows=matches.length?matches:recorded?[{e:strength.exercises[1],index:0,s:{reps:[12,12,12],top:lift(70,12),record:false}}]:[];
 const exs=rows.map(({e,index,s})=>{const prev=matches.length?e.sessions[index+1]:null;return {
  key:e.key,name:e.name,muscles:e.muscles,sets:s.reps.map((r,i)=>sset(r,s.top?.weight??0,[],s.record&&i===0)),
  lastDate:prev?.date??null,lastSets:prev?prev.reps.map(r=>lift(prev.top?.weight??0,r)):[],record:s.record?s.top:null,
 }});
 const counts={};for(const e of exs)for(const m of e.muscles)counts[m]=(counts[m]??0)+e.sets.length;
 strengthWorkoutHistory[id]={detail:noDetail,counted:null,along:null,more:[],recorded:[],selfEvaluation:null,effects:[],vo2maxNew:null,summary,recoveryHours:null,stats:[{key:'duration',label:'Time',value:summary.durationS,unit:'s'}],route:[],heartRate:[],zoneFloors:[],splits:[],strength:{
  unit:strength.unit,figure:strength.figure,checkedAt:'2025-10-15T19:30:00Z',sets:exs.reduce((n,e)=>n+e.sets.length,0),
  reps:recorded?exs.reduce((n,e)=>n+e.sets.reduce((n,s)=>n+s.reps,0),0):null,
  unknownRepSets:0,restS:0,records:exs.filter(e=>e.record).length,setsRecorded:recorded,
  zeroRepSets:0,missingRepSets:0,doubledSets:0,unnamedSets:0,muscles:muscleSets(counts),exercises:exs,
 }};
}
for(const w of Object.values(strengthWorkoutHistory))w.map=null;
writeFileSync(new URL('strengthWorkoutHistory.json',import.meta.url),JSON.stringify(strengthWorkoutHistory,null,2)+'\n');

// Synthetic daily HR with sleep, an activity and two missing intervals.
const heartTimeline=Array.from({length:720},(_,i)=>{
 const minute=i*2;
 if((minute>=450 && minute<510)||(minute>=800 && minute<844))return null;
 return Math.round(minute<370?52+Math.sin(i/13)*3:minute>=510 && minute<565?122+Math.sin(i/5)*10:66+Math.sin(i/17)*8);
});
const recordedHeart=heartTimeline.filter(value=>value!==null);
const heartRateDay={date:today.date,resting:54,minimum:Math.min(...recordedHeart),maximum:Math.max(...recordedHeart),durationMin:1440,sleep:[[0,370]],activities:[[510,565]],timeline:heartTimeline};
today.heartRateDay=heartRateDay;
// Your day: the same readings on one midnight axis, uploaded at 13:30 and viewed at 14:05.
// The 13:10 nap ends after the last Body Battery reading, so its change is unknown.
today.day={date:today.date,nowMin:845,uploadMin:810,sleep:segments.map(s=>({stage:s.stage,startMin:Math.max(0,s.startMin-89),endMin:s.startMin+s.minutes-89})).filter(s=>s.endMin>0),asleepMin:468,bodyBattery:batteryTimeline,bodyBatteryNow:78,stress:stressTimeline,stressAverage:31,heartRate:heartTimeline.slice(0,406),restingHr:54,clockChange:false,
 events:[{kind:'activity',id:run.id,name:run.name,icon:'run',startMin:372,endMin:424,durationMin:52,bodyBatteryChange:null},{kind:'nap',id:null,name:'Nap',icon:'moon',startMin:790,endMin:815,durationMin:25,bodyBatteryChange:null}]};
health.heartRateDay=heartRateDay;

// Training's 31 days list every recording, strength sessions included, newest first.
training.workouts=[...new Map([...training.workouts,...strength.workouts.map(w=>w.activity)].map(a=>[a.id,a])).values()].filter(a=>a.startLocal>='2025-09-16').sort((a,b)=>b.startLocal.localeCompare(a.startLocal));
// Plan board: Mon 13 – Sun 19 Oct viewed Thu 16. Strength 4, Zone 2 150 min, intensity on Garmin's goal.
const rec=(a,sport)=>{const z2=Math.floor(a.zoneSeconds[1]/60),hi=Math.floor((a.zoneSeconds[3]+a.zoneSeconds[4])/60);return {id:a.id,date:a.startLocal.slice(0,10),startLocal:a.startLocal,name:a.name,icon:a.icon,sport,typeKey:sport==='strength'?'strength_training':sport,durationMin:a.durationS/60,zone2Min:z2,highIntensityMin:hi,intensityMin:a.intensityMin}};
const boardRecs=[rec(training.workouts.find(w=>w.id==='9000000000005')??{id:'9000000000005',name:'Strength',icon:'strength',startLocal:'2025-10-13T12:30:00',durationS:2400,zoneSeconds:[1500,800,100,0,0],intensityMin:null},'strength'),
 rec({id:'9000000000004',name:'Upper B',icon:'strength',startLocal:'2025-10-14T07:10:00',durationS:3120,zoneSeconds:[700,1500,600,200,0],intensityMin:null},'strength'),
 rec(lowerA,'strength'),rec(bike,'cycling'),rec(run,'running')];
const zone2s={'2025-10-13':800,'2025-10-14':1500,'2025-10-15':1320+1800,'2025-10-16':585};
const intention=(id,date,sport,allocation,extra={})=>({id,date,time:extra.time??null,sport,typeKey:null,name:extra.name??null,durationMin:extra.durationMin??null,allocation:{strengthSessions:0,zone2Min:0,highIntensityMin:0,intensityMin:0,sleepNights:0,...allocation},kind:'workout',garminId:null,status:'open',history:[{at:'2025-10-12T19:00:00',change:'added',from:null,to:date}]});
const boardItems=[
 {intention:intention('i1760380000000','2025-10-13','strength',{strengthSessions:1},{time:'12:30'}),state:'done',recording:'9000000000005',matchedBy:'automatic',candidates:[],coach:null},
 {intention:intention('i1760380000001','2025-10-17','cycling',{zone2Min:40},{durationMin:60,name:'Easy ride'}),state:'planned',recording:null,matchedBy:null,candidates:[],coach:null},
 {intention:intention('i1760380000002','2025-10-18','strength',{strengthSessions:1}),state:'planned',recording:null,matchedBy:null,candidates:[],coach:null}];
const goalTargets={strengthSessions:4,zone2Min:150,highIntensityMin:0,intensityMin:0,intensityGarminGoal:true,sleepNights:0,sleepTime:null,sleepKind:'asleepBy',sleepScore:0,restDays:[]};
const boardDays=Array.from({length:7},(_,i)=>{const date=`2025-10-${13+i}`,future=date>'2025-10-16',recs=boardRecs.filter(r=>r.date===date),items=boardItems.filter(p=>p.intention.date===date);
 const plannedOf=k=>items.reduce((n,p)=>n+p.intention.allocation[k],0);
 const ring=(kind,recorded,planned)=>({kind,recorded,planned,fill:recorded>0?recorded/Math.max(planned,recorded):0});
 return {date,today:date==='2025-10-16',future,mark:null,rings:[ring('strength',future?0:recs.filter(r=>r.sport==='strength').length,plannedOf('strengthSessions')),ring('zone2',future?0:Math.floor((zone2s[date]??0)/60),plannedOf('zone2Min')),ring('intensity',future?0:({'2025-10-13':20,'2025-10-14':28,'2025-10-15':40,'2025-10-16':36})[date]??0,0)],
  done:items.length>0&&items.every(p=>p.state==='done'),notRecorded:items.some(p=>p.state==='notRecorded'),planned:items.filter(p=>p.state==='planned').length,recordings:recs.map(r=>r.id),items:items.map(p=>p.intention.id),garmin:date==='2025-10-17'?1:0,sleepScore:future?null:({'2025-10-13':81,'2025-10-14':76,'2025-10-15':88,'2025-10-16':84})[date]}});
const ledger=(kind,target,recorded,planned)=>({kind,target,original:null,revisedOn:null,recorded,planned,remaining:Math.max(0,target-recorded-planned),reached:recorded>=target,review:0,unavailable:null,observation:false});
const boardLedger=[ledger('strength',4,3,1),ledger('zone2',150,100,40),ledger('intensity',150,124,0)];
const board={today:'2025-10-16',start:'2025-10-13',end:'2025-10-19',goals:{from:'2025-10-06',targets:goalTargets,source:'setup',savedAt:'2025-10-06T08:00:00'},versions:[{from:'2025-10-06',targets:goalTargets,source:'setup',savedAt:'2025-10-06T08:00:00'}],garminIntensityGoal:150,
 ledger:boardLedger,weekRings:boardLedger.map(l=>({kind:l.kind,recorded:l.recorded,planned:l.planned,fill:Math.min(1,l.recorded/l.target)})),sleepScore:null,days:boardDays,items:boardItems,recordings:boardRecs,
 garmin:[{date:'2025-10-17',name:'Easy Run'}],garminState:'available',backlog:[{kind:'zone2',amount:10,sport:'cycling'},{kind:'intensity',amount:26,sport:'cycling'}],backlogPaused:false,pause:null,coverageComplete:true,explainObservation:false,readOnly:false,prompt:null,events:[{id:'e1760000000000',name:'Harbour 10K',date:'2025-11-09',daysUntil:24,sport:'running',source:'local',partial:false}]};
// The review of Mon 6 – Sun 12 Oct with next week's proposals.
const review={start:'2025-10-06',end:'2025-10-12',counted:true,reasons:['1 rest day'],illDays:0,pausedDays:0,restDays:1,notRecorded:1,goals:[{kind:'strength',target:4,recorded:4,state:'met'},{kind:'zone2',target:150,recorded:168,state:'met'},{kind:'intensity',target:150,recorded:162,state:'met'}],proposals:[{kind:'strength',current:4,proposed:4,reason:'Not enough complete weeks to suggest a change'},{kind:'zone2',current:150,proposed:165,reason:'Met in your last two complete weeks, 168 of 150 min last week'}],nextStart:'2025-10-13',appliesFrom:'2025-10-16',decidable:true,decision:null};
const reviews=[{start:'2025-10-06',counted:true,met:3,goals:3,reasons:['1 rest day']},{start:'2025-09-29',counted:false,met:0,goals:3,reasons:['2 ill days']}];

const progressTotals={activities:5,activeMin:Math.round(boardRecs.reduce((n,a)=>n+a.durationMin,0)),distanceKm:48.6,intensityMin:124,intensityDays:4,activeDays:4,coveredDays:16,days:16};
const progress={milestonesTotal:0,ringDays:[],today:'2025-10-16',period:'month',from:'2025-10-01',to:'2025-10-31',end:'2025-10-16',current:true,historyStart:'2025-09-01',totals:progressTotals,previous:null,previousFrom:null,previousTo:null,partial:false,
 buckets:Array.from({length:31},(_,i)=>{const date=`2025-10-${String(i+1).padStart(2,'0')}`,acts=boardRecs.filter(r=>r.date===date);return {from:date,to:date,minutes:acts.map(a=>({sport:a.sport,minutes:a.durationMin})),total:acts.reduce((n,a)=>n+a.durationMin,0),known:i<16,future:i>=16}}),
 calendar:Array.from({length:31},(_,i)=>{const date=`2025-10-${String(i+1).padStart(2,'0')}`,n=boardRecs.filter(r=>r.date===date).length;return {date,state:i>=16?'future':n?'active':'none',activities:n,covered:i<16}}),
 goalWeeks:[
  {start:'2025-09-29',rings:board.weekRings.map(r=>({...r,fill:r.fill/2})),current:false,mark:'ill',counted:false,met:0,goals:3,reasons:['2 ill days']},
  {start:'2025-10-06',rings:board.weekRings.map(r=>({...r,fill:1})),current:false,mark:null,counted:true,met:3,goals:3,reasons:['1 rest day']},
  {start:board.start,rings:board.weekRings,current:true,mark:null,counted:false,met:0,goals:3,reasons:['Week still in progress']}],
 milestones:[{id:'longest:running',kind:'longest',title:'Longest run',value:'10.2 km',date:'2025-10-16',route:'workout/9000000000001',new:false,corrected:false}],sleep:{nights:14,belowHour:2}};
// Training's lanes: Zone 2 and strength goals on, Thursday; eight counted weeks behind the band.
training.lanes={start:'2025-10-13',today:'2025-10-16',lanes:[
 {kind:'zone2',goal:150,cumulative:[0,30,82,100,null,null,null],usual:{low:[0,15,30,45,60,80,100],median:[0,25,45,65,85,110,130],high:[10,40,65,90,115,140,160]},weeks:8},
 {kind:'strength',goal:4,cumulative:[1,1,2,2,null,null,null],usual:{low:[0,1,1,1,2,2,3],median:[1,1,2,2,2,3,3],high:[1,2,2,3,3,3,4]},weeks:8}],
 bySport:[{sport:'cycling',minutes:84},{sport:'strength',minutes:96},{sport:'running',minutes:52}].sort((a,b)=>b.minutes-a.minutes)};
// Progress: the October ring calendar (whole weeks 29 Sep – 2 Nov) and the domain views.
{
 const day=n=>{const d=new Date(Date.UTC(2025,8,29+n));return d.toISOString().slice(0,10)};
 const fill=(r,p)=>r>0?r/Math.max(p,r):0;
 progress.milestonesTotal=progress.milestones.length;progress.ringDays=Array.from({length:35},(_,i)=>{const date=day(i),inWeek=boardDays.find(d=>d.date===date);if(inWeek)return inWeek;
  const future=date>'2025-10-16',lift=!future&&[1,3].includes(i%7)?1:0,z2=!future&&i%7===5?45:0,im=future?0:[0,30,25,10,40,0,55][i%7];
  return {date,today:false,future,mark:date==='2025-10-08'?'ill':date==='2025-10-09'?'ill':null,rings:[{kind:'strength',recorded:lift,planned:0,fill:fill(lift,0)},{kind:'zone2',recorded:z2,planned:0,fill:fill(z2,0)},{kind:'intensity',recorded:im,planned:0,fill:fill(im,0)}],
   done:false,notRecorded:false,planned:0,recordings:[],items:[],garmin:0,sleepScore:null}});
 const weekStart=n=>{const d=new Date(Date.UTC(2025,6,28+7*n));return d.toISOString().slice(0,10)};
 const zones=[90,120,140,0,155,130,160,150,40,135,165,100],ims=[140,160,150,null,175,150,180,165,60,155,170,124],str=[3,4,3,2,4,4,3,4,1,4,4,3];
 const months=['2024-11','2024-12','2025-01','2025-02','2025-03','2025-04','2025-05','2025-06','2025-07','2025-08','2025-09','2025-10'];
 var progressDomains={today:'2025-10-16',goals:goalTargets,weeksFrom:weekStart(0),weeksTo:'2025-10-19',monthsFrom:'2024-11-01',monthsTo:'2025-10-31',
  weeks:zones.map((z,i)=>({start:weekStart(i),current:i===11,mark:i===8?'ill':i===3?'unknown':null,zone2Min:i===3?null:z,highIntensityMin:null,intensityMin:ims[i],strength:i===3?null:str[i],vo2max:i%3===2?50+Math.floor(i/4):null,
   // No goals for the first four weeks, then 120 min and three sessions, then today's targets.
   zone2Goal:i<4?null:i<8?120:150,highIntensityGoal:null,intensityGoal:i<4?null:150,strengthGoal:i<4?null:i<8?3:4})),
  months:months.map((m,i)=>({start:m+'-01',minutes:i<1?null:1200+i*90+(i%3)*120,restingHr:i<1?null:Math.round((60-i*0.3)*10)/10,hrv:i<1?null:Math.round((50+i*0.6)*10)/10,vo2max:i<1?null:48+Math.floor(i/3),fitnessAge:i<1?null:43-Math.floor(i/3)}))};
 globalThis.progressDomains=progressDomains;
}
// Harbour 10K is a run; no run is planned this week, so it only counts down.
board.eventWeek=null;board.coach=null;board.fitness={vigorous:30,target:75};
// Profile: Garmin's facts as stored, the gear Training lists, and your limits.
const profile={age:46,units:'metric',weight:72.4,weightUnit:'kg',weightFreshness:{recordedOn:'2025-10-14',ageDays:2,stale:false,thresholdDays:7},watchLastUpload:'2025-10-16T12:30:00.000Z',gear:training.gear,limits:{excluded:[],note:''}};
for(const [name,obj] of Object.entries({progressDomains:globalThis.progressDomains,profile,today,sleep,training,workout,strengthWorkout,strength,health,checkin,status,plan,board,review,reviews,progress})){writeFileSync(new URL(`${name}.json`,import.meta.url),JSON.stringify(obj,null,2)+'\n')}
// `satisfies` checks the exact fixtures against the generated Rust contract at build time.
const types={progressDomains:'ProgressDomains',profile:'ProfileView',today:'TodayView',sleep:'SleepView',training:'TrainingView',workout:'WorkoutView',strengthWorkout:'WorkoutView',strength:'StrengthView',health:'HealthView',checkin:'CheckinView',status:'AppStatus',plan:'PlanView',board:'PlanBoard',review:'WeekReview',reviews:'ReviewRow[]',progress:'ProgressView'};
writeFileSync(new URL('data.ts',import.meta.url),[...new Set(Object.values(types).map(t=>t.replace('[]','')))].map(t=>`import type { ${t} } from '../generated/${t}';`).join('\n')+'\n'+Object.entries({today,sleep,training,workout,strengthWorkout,strength,health,checkin,status,plan,board,review,reviews,progress}).map(([n,v])=>`export const ${n} = ${JSON.stringify(v)} satisfies ${types[n]};`).join('\n')+'\n'+`export const strengthWorkoutHistory = ${JSON.stringify(strengthWorkoutHistory)} satisfies Record<string,WorkoutView>;\n`);
