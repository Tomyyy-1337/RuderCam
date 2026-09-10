<script lang="ts">
    import { buildSmoothedRoutePath, projectGpsCoordinates } from "./path_map_data";
    import type { GpsPosition } from "./types";

    let { fahrt }: { fahrt?: { gps_positions?: GpsPosition[] } } = $props();

    let routePoints = $derived(projectGpsCoordinates(fahrt?.gps_positions ?? []));
    let routePath = $derived(buildSmoothedRoutePath(routePoints));
</script>

{#if routePoints.length > 0}
    <figure class="path-map">
        <svg
            viewBox="0 0 100 100"
            preserveAspectRatio="xMidYMid meet"
            role="img"
            aria-label="GPS route of the drive"
        >
            <path class="route" d={routePath} />

            {#each routePoints as point, index}
                <circle
                    class:start-point={index === 0}
                    class:end-point={index === routePoints.length - 1}
                    class="route-point"
                    cx={point.x}
                    cy={point.y}
                    r={index === 0 || index === routePoints.length - 1 ? 2.8 : 1.6}
                />
            {/each}
        </svg>
    </figure>
{:else}
    <div class="empty-state">Keine GPS-Daten für diese Fahrt vorhanden.</div>
{/if}

<style>
    .path-map {
        width: 100%;
        margin: 0.9rem 0 0.3rem;
        border-radius: 1rem;
        overflow: hidden;
        border: 1px solid rgba(255, 255, 255, 0.08);
        box-shadow: inset 0 0 0 1px rgba(255, 255, 255, 0.04);
        background: rgba(10, 15, 25, 0.78);
    }

    svg {
        display: block;
        width: 100%;
        height: 220px;
    }

    .route {
        fill: none;
        stroke: #62b6ff;
        stroke-width: 2.8;
        stroke-linecap: round;
        stroke-linejoin: round;
        filter: drop-shadow(0 0 5px rgba(98, 182, 255, 0.55));
    }

    .route-point {
        fill: rgba(98, 182, 255, 0.92);
        stroke: rgba(255, 255, 255, 0.72);
        stroke-width: 0.5;
    }

    .start-point {
        fill: #7ef0b5;
    }

    .end-point {
        fill: #ff8a6c;
    }

    .empty-state {
        margin-top: 0.5rem;
        padding: 0.8rem 0.9rem;
        border-radius: 0.85rem;
        background: rgba(255, 255, 255, 0.04);
        color: rgba(255, 255, 255, 0.72);
        font-size: 0.9rem;
        text-align: center;
    }
</style>

