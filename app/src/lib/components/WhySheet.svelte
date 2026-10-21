<script lang="ts">
 import {tick} from 'svelte';
 import type {Why} from '../generated/Why';
 import {toneColor} from '../charts';
 import {sheetOpened} from '../sheet';
 import Icon from './Icon.svelte';
 let {why,label='Why?'}:{why:Why,label?:string}=$props();
 let open=$state(false),trigger:HTMLButtonElement,closer=$state<HTMLButtonElement>();
 const id=`why-${Math.random().toString(36).slice(2)}`;
 async function show(){open=true;await tick();closer?.focus()}
 function hide(){open=false;trigger?.focus()}
 $effect(()=>{if(open)return sheetOpened(()=>{open=false;trigger?.focus()})});
</script>

<button class="why-button" bind:this={trigger} aria-haspopup="dialog" onclick={show}><Icon name="info" size={15}/>{label}</button>
{#if open}
 <div class="why-scrim" role="presentation" onclick={hide}></div>
 <div class="why-sheet" role="dialog" aria-modal="true" aria-labelledby={id} tabindex="-1" onkeydown={(e)=>{if(e.key==='Escape'){e.stopPropagation();hide()}else if(e.key==='Tab'){e.preventDefault();closer?.focus()}}}>
  <span class="grab" aria-hidden="true"></span>
  <div class="why-head"><h2 id={id}>{why.title}</h2><button class="why-close" bind:this={closer} aria-label="Close" onclick={hide}><Icon name="close" size={19}/></button></div>
  {#if why.feedback}<p class="why-feedback">Garmin · {why.feedback}</p>{/if}
  <p class="why-summary">{why.summary}</p>
  <div class="why-list">
   {#each why.inputs as input}
    <div class="why-row">
     <div class="why-top"><b>{input.name}</b>{#if input.value}<span class="n">{input.value}</span>{/if}{#if input.rating}<span class="pill" style:color={toneColor(input.tone)}>{input.rating}</span>{/if}</div>
     <p>{input.detail}</p>
    </div>
   {/each}
  </div>
  <p class="why-source">{why.source}</p>
 </div>
{/if}

<style>
 .why-button{display:inline-flex;align-items:center;gap:6px;max-width:100%;min-width:0;overflow-wrap:anywhere;min-height:36px;padding:7px 12px;border-radius:999px;background:rgba(255,255,255,.06);border:1px solid var(--line);font:500 13px Inter;color:var(--text)}
 .why-button :global(svg){flex:none;color:var(--text2)}
 .why-scrim{position:fixed;inset:0;z-index:20;background:rgba(4,6,11,.62)}
 .why-sheet{position:fixed;left:0;right:0;bottom:0;z-index:21;max-width:560px;margin-inline:auto;max-height:85dvh;overflow-y:auto;overscroll-behavior:contain;background:var(--card);border-top:1px solid var(--line);border-radius:26px 26px 0 0;padding:10px 20px calc(24px + env(safe-area-inset-bottom));text-align:left}
 .why-sheet:focus{outline:none}
 .grab{display:block;width:40px;height:4px;border-radius:2px;background:rgba(255,255,255,.18);margin:0 auto 14px}
 .why-head{display:flex;align-items:center;justify-content:space-between;gap:12px;margin-bottom:8px}
 .why-head h2{min-width:0;font-size:22px;overflow-wrap:anywhere}
 .why-close{flex:none;width:44px;height:44px;border-radius:50%;display:grid;place-items:center;background:rgba(255,255,255,.05);border:1px solid var(--line);color:var(--text2)}
 .why-feedback{font:500 13px Inter;color:var(--text3);margin-top:-2px}
 .why-summary{font-size:16px;line-height:1.45;color:var(--text);margin:10px 0 6px}
 .why-row{padding:12px 0;border-top:1px solid var(--line)}
 .why-row:first-child{border-top:0}
 .why-top{display:flex;flex-wrap:wrap;align-items:center;gap:10px}
 .why-top b{flex:1 1 10rem;min-width:0;overflow-wrap:anywhere;font:600 15px Inter;color:var(--text)}
 .why-top .n{font-size:17px;color:var(--text);overflow-wrap:anywhere}.why-top .pill{white-space:normal;overflow-wrap:anywhere}
 .why-row p{font-size:12px;line-height:1.45;color:var(--text2);margin-top:4px}
 .why-source{font-size:11px;color:var(--text3);margin-top:8px}
 @media(min-width:1000px){
  .why-sheet{left:50%;right:auto;bottom:auto;top:50%;transform:translate(-50%,-50%);width:min(520px,100%);max-height:85dvh;border:1px solid var(--line);border-radius:26px;padding-bottom:24px}
  .grab{display:none}
 }
</style>
