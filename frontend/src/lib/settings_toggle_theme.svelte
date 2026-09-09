<article>
    <h4>Color Theme</h4>
    <p>Wechseln zwischen hellem und dunklem Design.</p>
    <button onclick={toggleTheme} style="width:100%;">
        {#if theme === 'dark'}
            Helles Design aktivieren
        {:else}
            Dunkles Design aktivieren
        {/if}
    </button>
    <div class="spacer"></div>
</article>
    
<script>
    import { onMount } from "svelte";
    import { setTheme } from "./setTheme";

    let theme = $state('light');

    onMount(() => {
        theme = localStorage.getItem('theme') || 'light';
    });

    function toggleTheme() {
        theme = theme === 'dark' ? 'light' : 'dark';
        setTheme(theme);
    }
</script>

<style>
    button {
        margin-left: auto;
    }
</style>