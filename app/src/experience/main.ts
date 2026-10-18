import {mount} from 'svelte';
import '../app.css';
import './experience.css';
import Experience from './Experience.svelte';

export default mount(Experience, {target: document.getElementById('app')!});
