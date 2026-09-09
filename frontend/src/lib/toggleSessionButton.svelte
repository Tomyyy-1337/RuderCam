{#if !activeSession.isActive}
    <button class="green {variant}" onclick={startSession}>
        Fahrt starten
    </button>
{:else}
    <button class="red {variant}" onclick={endSession}>
        Fahrt beenden
    </button>
{/if}

<script>
    import { Session } from "./fahrtenbuchStore";

    let { fahrtenbuch, activeSession = $bindable(), variant = "primary" } = $props();

    function startSession() {    
        activeSession.isActive = true;
        const currentTime = new Date().toISOString();
        activeSession.distance_traveled_km = 0;
        activeSession.average_speed_kmh = 0;
        activeSession.duration_secs = 0;

        fetch('/api/start_session', {
            method: 'POST',
            headers: {
                'Content-Type': 'application/json',
            },
            body: JSON.stringify({ client_time: currentTime })
        });
    }

    function endSession() {
        activeSession.isActive = false;
        activeSession.end_time = new Date();

        fetch('/api/stop_session', {
            method: 'POST',
            headers: {
                'Content-Type': 'application/json',
            }
        }).then(response => {
            if (response.ok) {
                return response.json();
            }
        }).then(sessionSummary => {
            if (sessionSummary) {
                console.log('Session summary received:', sessionSummary);
                let session = new Session(sessionSummary);
                fahrtenbuch.update(/** @param {import("./fahrtenbuchStore").FahrtenbuchStore} f */ (f) => {
                    f.addSession(session);
                    return f;
                });
            }
        });
    }
</script>

<style>
    button.primary {
        width: 100%;
        font-weight: bold;
        font-size: 1rem;
    }

    button.green {
        background-color: transparent;
        color: #4CAF50;
        border: 2px solid #4CAF50;
    }

    button.red {
        background-color: transparent;
        color: #f44336;
        border: 2px solid #f44336;
    }
    
    button.overlay {
        display: flex;
        align-items: center;
        justify-content: center;
        text-align: center;
        height: 2.8rem;
        width: 4.8rem;
        margin: 0 0 0 auto;
        font-weight: bold;
        font-size: 0.72rem;
        line-height: 1.1;
        padding: 0.15rem;
        justify-self: end;
        box-sizing: border-box;
        white-space: normal;
        word-break: break-word;
        border-radius: 0.6rem; 
        background-color: rgba(0, 0, 0, 0.45);
    }
</style>