<SettingsCard title="Fahrtenbuch löschen" description="Lösche alle Fahrten aus dem Fahrtenbuch.">
    <button style="width:100%;" onclick={deleteFahrtenbuch}>
        Alle Fahrten löschen
    </button>
    <Dialog
        bind:open={confirmOpen}
        title="Fahrtenbuch löschen?"
        message="Möchten Sie wirklich alle Fahrten löschen? Diese Aktion kann nicht rückgängig gemacht werden."
        confirmLabel="Löschen"
        cancelLabel="Abbrechen"
        tone="danger"
        onConfirm={deleteConfirmed}
    />
</SettingsCard>

<script lang="ts">
    import Dialog from "../components/dialog.svelte";
    import SettingsCard from "./settings_card.svelte";
    import { fahrtenbuch, FahrtenbuchStore } from "../classes/fahrtenbuchStore";

    let confirmOpen = $state(false);

    function deleteFahrtenbuch(): void {
        confirmOpen = true;
    }

    function deleteConfirmed(): void {
        fahrtenbuch.update((store: FahrtenbuchStore) => {
            store.clearHistory();
            return store;
        });
    }
</script>