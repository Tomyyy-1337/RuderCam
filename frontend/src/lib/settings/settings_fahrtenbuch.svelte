<article>
    <h4>Fahrtenbuch löschen</h4>
    <p>Lösche alle Fahrten aus dem Fahrtenbuch.</p>
    <button style="width:100%;" onclick={deleteFahrtenbuch}>
        Alle Fahrten löschen
    </button>
    <div class="spacer"></div>
</article>

<script lang="ts">
    import type { Writable } from "svelte/store";
    import type { FahrtenbuchStore } from "../fahrtenbuch/fahrtenbuchStore";

    let { fahrtenbuch = $bindable() }: { fahrtenbuch: Writable<FahrtenbuchStore> } = $props();

    function deleteFahrtenbuch(): void {
        if (confirm('Möchten Sie wirklich alle Fahrten löschen? Diese Aktion kann nicht rückgängig gemacht werden.')) {
            fahrtenbuch.update((store) => {
                store.clearHistory();
                return store;
            });
        }
    }
</script>