// Monotone cubic display interpolation. Nulls break a run; no curve overshoots
// a supplied reading. Derived metric values are calculated in the core.
export function smoothPath(pts:[number,number|null][]):string{
 const runs:[number,number][][]=[];let run:[number,number][]=[];
 for(const [x,y] of pts){if(y==null){if(run.length)runs.push(run);run=[];}else run.push([x,y]);}
 if(run.length)runs.push(run);
 return runs.map(monotone).join('');
}
function monotone(p:[number,number][]):string{
 const n=p.length,f=(v:number)=>v.toFixed(1);
 if(n<3)return p.map(([x,y],i)=>`${i?'L':'M'}${f(x)},${f(y)}`).join('');
 const d=p.slice(1).map(([x,y],i)=>(y-p[i][1])/(x-p[i][0]||1));
 const m=p.map((_,i)=>i===0?d[0]:i===n-1?d[n-2]:d[i-1]*d[i]<=0?0:(d[i-1]+d[i])/2);
 for(let i=0;i<n-1;i++){
  if(d[i]===0){m[i]=m[i+1]=0;continue;}
  const a=m[i]/d[i],b=m[i+1]/d[i],s=a*a+b*b;
  if(s>9){const t=3/Math.sqrt(s);m[i]=t*a*d[i];m[i+1]=t*b*d[i];}
 }
 let out=`M${f(p[0][0])},${f(p[0][1])}`;
 for(let i=0;i<n-1;i++){const h=(p[i+1][0]-p[i][0])/3;out+=`C${f(p[i][0]+h)},${f(p[i][1]+m[i]*h)} ${f(p[i+1][0]-h)},${f(p[i+1][1]-m[i+1]*h)} ${f(p[i+1][0])},${f(p[i+1][1])}`;}
 return out;
}
