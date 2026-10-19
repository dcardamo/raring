<script lang="ts">
 import Icon from './Icon.svelte';
 import type {PlanBoard} from '../generated/PlanBoard';
 import type {PlanItem} from '../generated/PlanItem';
 import type {LocalAction} from '../generated/LocalAction';
 import {SPORT_NAMES, STATE_WORD} from '../goals';
 import {sheetOpened} from '../sheet';
 import {clock, dateLabel, duration} from '../format';
 // One session's choices. Moving adds no catch-up; only its person marks a
 // passed session missed; a match Raring made can be undone.
 let {board, item, close, act, navigate}: {board: PlanBoard, item: PlanItem, close: () => void, act: (a: LocalAction) => void, navigate: (route: string) => void} = $props();
 let moving = $state(false);
 let i = $derived(item.intention);
 let passed = $derived(i.date < board.today);
 let open = $derived(i.status === 'open');
 let days = $derived(Array.from({length: 7}, (_, n) => { const d = new Date(board.today + 'T12:00:00Z'); d.setUTCDate(d.getUTCDate() + n); return d.toISOString().slice(0, 10); }).filter(d => d !== i.date));
 let candidates = $derived(item.candidates.map(id => board.recordings.find(r => r.id === id)).filter(r => !!r));
 let recording = $derived(board.recordings.find(r => r.id === item.recording));
 $effect(() => sheetOpened(close));
 const run = (a: LocalAction) => { act(a); close(); };
 const history = (h: PlanItem['intention']['history'][number]) => h.change === 'moved' ? `Moved from ${dateLabel(h.from ?? '')} to ${dateLabel(h.to ?? '')}` : h.change === 'added' ? `Added for ${dateLabel(h.to ?? i.date)}` : h.change === 'linkCleared' ? 'Link removed' : h.change === 'letGo' ? 'Let go' : h.change === 'missed' ? 'Marked missed' : h.change === 'reopened' ? 'Reopened' : 'Edited';
</script>
<svelte:window onkeydown={e => { if (e.key === 'Escape') close(); }}/>
<button class="scrim" aria-label="Close" onclick={close}></button>
<div class="sheet" role="dialog" aria-modal="true" aria-labelledby="item-title">
 <span class="grab"></span>
 <div class="sheet-h"><h2 id="item-title">{i.name ?? SPORT_NAMES[i.sport]}</h2><button class="close" aria-label="Close" onclick={close}><Icon name="close" size={20}/></button></div>
 <p class="small">{dateLabel(i.date)}{i.time ? ` · ${i.time}` : ''} · {STATE_WORD[item.state]}{item.matchedBy === 'automatic' ? ' · matched by Raring (same day and sport)' : item.matchedBy === 'linked' ? ' · linked by you' : ''}</p>
 {#if recording}<button class="opt" onclick={() => { close(); navigate('workout/' + recording.id); }}><Icon name={recording.icon} size={18}/><span><b>{recording.name}</b><small>{clock(recording.startLocal)} · {duration(recording.durationMin)}</small></span></button>
  <button class="opt" onclick={() => run({action: 'unlink', planId: i.id, activityId: recording.id})}><Icon name="close" size={18}/><span><b>Not this session</b><small>Keep them apart; the recording still counts</small></span></button>{/if}
 {#if item.state === 'review'}{#each candidates as r}<button class="opt" onclick={() => run({action: 'link', planId: i.id, activityId: r!.id})}><Icon name="check" size={18}/><span><b>This was my session</b><small>{r!.name} · {clock(r!.startLocal)}</small></span></button>{/each}{/if}
 {#if open && item.state !== 'done'}
  {#if moving}<div class="chips2" role="group" aria-label="Move to">{#each days as d}<button onclick={() => run({action: 'moveIntention', id: i.id, date: d})}>{d === board.today ? 'Today' : dateLabel(d)}</button>{/each}</div>
  {:else}<button class="opt" onclick={() => moving = true}><Icon name="arrow-right" size={18}/><span><b>{passed ? 'Reschedule' : 'Move'}</b><small>To today or a later day · nothing to catch up</small></span></button>{/if}
  <button class="opt" onclick={() => run({action: 'setIntentionStatus', id: i.id, status: 'letGo'})}><Icon name="close" size={18}/><span><b>Let it go</b><small>Not planned any more</small></span></button>
  {#if passed}<button class="opt" onclick={() => run({action: 'setIntentionStatus', id: i.id, status: 'missed'})}><Icon name="alert" size={18}/><span><b>Mark as missed</b><small>Only if you want it recorded that way</small></span></button>{/if}
  {#if !passed}<button class="opt" onclick={() => run({action: 'removeIntention', id: i.id})}><Icon name="close" size={18}/><span><b>Remove</b><small>Delete it from your plan</small></span></button>{/if}
 {:else if !open}<button class="opt" onclick={() => run({action: 'setIntentionStatus', id: i.id, status: 'open'})}><Icon name="arrow-left" size={18}/><span><b>Reopen</b><small>Back in your plan</small></span></button>{/if}
 <button class="opt" disabled aria-describedby="garmin-na"><Icon name="arrow-right" size={18}/><span><b>Send to Garmin</b><small id="garmin-na">Not yet available: Raring doesn’t write to Garmin</small></span></button>
 {#if i.history.length}<ol class="hist">{#each i.history as h}<li>{history(h)}</li>{/each}</ol>{/if}
</div>
<style>
 .scrim{position:fixed;inset:0;border:0;background:rgba(5,8,14,.62);z-index:20;padding:0}
 .sheet{position:fixed;left:0;right:0;bottom:0;z-index:21;max-width:560px;margin:0 auto;background:#151B28;border:1px solid var(--line);border-bottom:0;border-radius:26px 26px 0 0;padding:12px 20px 28px;max-height:90vh;overflow:auto}
 .grab{display:block;width:40px;height:4px;border-radius:2px;background:rgba(255,255,255,.18);margin:0 auto 16px}
 .sheet-h{display:flex;justify-content:space-between;align-items:center;gap:12px}
 .close{width:44px;height:44px;border-radius:50%;border:1px solid var(--line);background:none;display:grid;place-items:center;color:var(--text2)}
 .opt{display:grid;grid-template-columns:24px 1fr;gap:12px;align-items:center;width:100%;text-align:left;margin-top:8px;padding:12px 14px;border:1px solid var(--line);border-radius:16px;background:rgba(255,255,255,.04);color:var(--text);min-height:52px}
 .opt:disabled{color:var(--text2);border-style:dashed;cursor:default}.opt b{display:block;font:600 15px Inter}.opt small{display:block;font-size:12px;color:var(--text2);margin-top:2px}
 .chips2{display:flex;flex-wrap:wrap;gap:8px;margin:12px 0 0}.chips2 button{min-height:40px;padding:8px 12px;border-radius:999px;border:1px solid var(--line);background:none;color:var(--text);font:600 13px Inter}
 .hist{margin:14px 4px 0;padding-left:18px;font-size:12px;color:var(--text2);display:grid;gap:3px}
</style>
