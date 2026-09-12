{#if !activeSession.isActive}
    <button class="green {variant}" onclick={startSession}>
        Fahrt starten
    </button>
{:else}
    <button class="red {variant}" onclick={endSession}>
        Fahrt beenden
    </button>
{/if}

<script lang="ts">
    import type { Writable } from "svelte/store";
    import { FahrtenbuchStore, Session } from "../fahrtenbuch/fahrtenbuchStore";
    import { isSessionJson } from "../types";
    import type { ActiveSession, SessionButtonVariant, SessionJson } from "../types";

    let {
        fahrtenbuch,
        activeSession = $bindable(),
        variant = "primary",
    }: {
        fahrtenbuch: Writable<FahrtenbuchStore>;
        activeSession: ActiveSession;
        variant?: SessionButtonVariant;
    } = $props();

    function startSession(): void {
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

    async function endSession(): Promise<void> {
        activeSession.isActive = false;
        activeSession.end_time = new Date();

        const response = await fetch('/api/stop_session', {
            method: 'POST',
            headers: {
                'Content-Type': 'application/json',
            }
        });

        if (!response.ok) {
            return;
        }

        const payload: unknown = await response.json();
        if (!isSessionJson(payload)) {
            return;
        }

        const sessionSummary: SessionJson = payload;
        const session = new Session(sessionSummary);
        fahrtenbuch.update((store) => {
            store.addSession(session);
            return store;
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
        color: #4caf50;
        background-color: rgba(0, 255, 0, 0.1);
        border: 2px solid #4CAF50;
    }

    button.red {
        color: #f44336;
        background-color: rgba(255, 0, 0, 0.04);
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
    }
</style>