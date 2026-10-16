<script lang="ts">
 import type {Chip as ChipType} from '../generated/Chip';
 import {toneColor} from '../charts';
 import Icon from './Icon.svelte';
 let {chip,onclick,label}:{chip:ChipType,onclick?:()=>void,label?:string}=$props();
</script>
{#snippet content()}<Icon name={chip.icon} size={15}/>{#if chip.text && chip.text!=='ms'}<span class="muted">{chip.text}</span>{/if}{#if chip.value}<b class="n">{chip.value}</b>{/if}{#if chip.text==='ms'}<span class="muted">ms</span>{/if}{/snippet}
{#if onclick}<button class="chip tap" style:color={toneColor(chip.tone)} {onclick} aria-label={label}>{@render content()}</button>{:else}<span class="chip" style:color={toneColor(chip.tone)}>{@render content()}</span>{/if}
