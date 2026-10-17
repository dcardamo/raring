<script lang="ts">
 import type {Reconciliation} from '../generated/Reconciliation';
 import {toneColor} from '../charts';
 import Icon from './Icon.svelte';
 // Self-contained so Today's layout can move it without touching its logic.
 let {reconcile}:{reconcile:Reconciliation}=$props();
 let open=$state(false);
 const id=`reconcile-${Math.random().toString(36).slice(2)}`;
</script>

<div class="reconcile">
 <button class:open aria-expanded={open} aria-controls={id} onclick={()=>open=!open}>
  <span class="dots" aria-hidden="true">{#each reconcile.tones as tone}<i style:background={toneColor(tone)}></i>{/each}</span>
  <span>{reconcile.label}</span><Icon name="chevron-down" size={15}/>
 </button>
 <p {id} hidden={!open}>{reconcile.line}</p>
</div>

<style>
 .reconcile{margin-top:12px}
 button{display:inline-flex;align-items:center;gap:8px;min-height:36px;padding:7px 12px;border-radius:999px;background:none;border:1px dashed rgba(255,255,255,.24);font:500 13px Inter;color:var(--text);text-align:left}
 button :global(svg){flex:none;color:var(--text2)}
 button.open :global(svg){transform:rotate(180deg)}
 .dots{display:inline-flex;gap:3px;flex:none}
 .dots i{width:7px;height:7px;border-radius:50%}
 p{max-width:340px;margin:10px auto 0;font-size:14px;line-height:1.45;color:var(--text2)}
 @media(min-width:660px){p{margin-left:0}}
</style>
