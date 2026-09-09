<nav>
    <button 
        onclick={() => activeTab = 'sessions'} 
        style="width:50%" 
        class:inactive={activeTab !== 'sessions'}
    > Fahrtenbuch </button>
    <button 
        onclick={() => activeTab = 'settings'} 
        style="width:50%"
        class:inactive={activeTab !== 'settings'}
    > Einstellungen </button>
</nav>

<script>
    let { activeTab = $bindable() } = $props();
</script>