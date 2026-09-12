<article>
    <h4>Color Theme</h4>
    <p>Wechseln zwischen hellem und dunklem Design.</p>
    <select id="theme-selector" name="theme-selector" bind:value={theme} onchange={handleThemeChange}>
        <option value="light">Helles Design</option>
        <option value="dark">Dunkles Design</option>
    </select>
    <div class="spacer"></div>
</article>
    
<script lang="ts">
    import { onMount } from "svelte";
    import { setTheme } from "./setTheme";
    import type { Theme } from "../types";

    let theme = $state<Theme>('light');

    onMount(() => {
        theme = localStorage.getItem('theme') === 'dark' ? 'dark' : 'light';
    });

    function handleThemeChange(event: Event): void {
        theme = (event.currentTarget as HTMLSelectElement).value as Theme;
        setTheme(theme);
    }
</script>
