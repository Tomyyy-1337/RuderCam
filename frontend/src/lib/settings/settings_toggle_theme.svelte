<SettingsCard title="Color Theme" description="Wechseln zwischen hellem und dunklem Design.">
    <select id="theme-selector" name="theme-selector" bind:value={theme} onchange={handleThemeChange}>
        <option value="light">Helles Design</option>
        <option value="dark">Dunkles Design</option>
    </select>
</SettingsCard>
    
<script lang="ts">
    import { onMount } from "svelte";
    import { setTheme } from "../classes/themeStore.svelte";
    import SettingsCard from "./settings_card.svelte";
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
