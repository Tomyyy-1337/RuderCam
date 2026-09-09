<article>
    <h4>Fahrtenbuch löschen</h4>
    <p>Lösche alle Fahrten aus dem Fahrtenbuch.</p>
    <button style="width:100%;" onclick={deleteFahrtenbuch}>
        Alle Fahrten löschen
    </button>
    <div class="spacer"></div>
</article>

<script>
    let {fahrtenbuch = $bindable()} = $props();

    function deleteFahrtenbuch() {
        if (confirm('Möchten Sie wirklich alle Fahrten löschen? Diese Aktion kann nicht rückgängig gemacht werden.')) {
            fahrtenbuch.update(f => {
                f.clearHistory();
                return f;
            });
        }
    }
</script>