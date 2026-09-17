<div bind:this={mapShellElement} class:ios-fullscreen={iosFullscreen} class="map-shell">
    <div bind:this={mapElement} class="map"></div>
    {#if showFullscreenTapHint}
        <div class="fullscreen-tap-hint" role="status">
            Noch mal berühren, um Vollbild zu aktivieren
        </div>
    {/if}
    {#if usesIOSFullscreenFallback}
        <button
            type="button"
            class="map-fullscreen-button"
            onclick={toggleIOSFullscreen}
            aria-label={iosFullscreen ? "Karte verkleinern" : "Karte vergroessern"}
            title={iosFullscreen ? "Karte verkleinern" : "Karte vergroessern"}
        >{iosFullscreen ? "x" : "+"}</button>
    {/if}
</div>

<script lang="ts">
    import { onMount } from "svelte";
    import { get } from "svelte/store";
    import {
        FullscreenControl,
        LngLatBounds,
        Map,
        Marker,
        addProtocol,
        removeProtocol,
        setWorkerUrl
    } from "maplibre-gl";
    import workerUrl from "maplibre-gl/dist/maplibre-gl-worker.mjs?worker&url";
    import { PMTiles, Protocol } from "pmtiles";
    import "maplibre-gl/dist/maplibre-gl.css";
    import { currentTheme } from "../settings/themeStore";
    import { applyThemeToMap, buildLayers, buildSources, getMapPalette } from "./mapStyle";
    import { discoverPmtilesBaseUrl } from "./pmtilesDiscovery";
    import type { GpsPosition, Waypoint } from "../types";

    const DEFAULT_CENTER: Waypoint = [8.472401705884762, 49.363691016649035];
    const MARKER_FIT_PADDING = 35;
    const FULLSCREEN_TAP_WINDOW_MS = 1000;

    let { waypoints = [] }: { waypoints: GpsPosition[] } = $props();

    const normalizeWaypoints = (points: GpsPosition[]): Waypoint[] => points.map((point) => [point.lon, point.lat] as Waypoint)
    
    let normalizedWaypoints = $derived(normalizeWaypoints(waypoints));

    const getWaypointCenter = (points: Waypoint[]): Waypoint => {
        if (!points.length) {
            return DEFAULT_CENTER;
        }

        const total = points.reduce(
            (accumulator, [longitude, latitude]) => {
                accumulator.longitude += longitude;
                accumulator.latitude += latitude;
                return accumulator;
            },
            { longitude: 0, latitude: 0 }
        );

        return [total.longitude / points.length, total.latitude / points.length];
    };

    const addWaypointGeometry = (map: Map, points: Waypoint[]): void => {
        if (points.length < 2) {
            return;
        }

        const source = map.getSource("waypoint-path");
        if (source && "setData" in source) {
            (source as any).setData({
                type: "Feature",
                geometry: { type: "LineString", coordinates: points },
                properties: {}
            });
        }
    };

    const addWaypointMarkers = (map: Map, points: Waypoint[]): void => {
        points.forEach((coordinate, index) => {
            if (!coordinate || (index !== 0 && index !== points.length - 1)) {
                return;
            }

            new Marker({ color: index === 0 ? "#00a000" : "#d00000" })
                .setLngLat(coordinate)
                .addTo(map);
        });
    };

    const getMarkerAwarePadding = (padding: number) => ({
        top: Math.max(padding, MARKER_FIT_PADDING),
        right: padding,
        bottom: padding,
        left: padding
    });

    const buildInitialMapOptions = (points: Waypoint[]) => {
        if (points.length <= 1) {
            return {
                center: getWaypointCenter(points),
                zoom: 12
            };
        }

        const bounds = points.reduce(
            (currentBounds, coordinate) => currentBounds.extend(coordinate),
            new LngLatBounds(points[0], points[0])
        );

        return {
            bounds,
            fitBoundsOptions: {
                padding: getMarkerAwarePadding(50),
                maxZoom: 15,
                animate: false,
                duration: 0
            }
        };
    };

    const fitMapToWaypoints = (map: Map, points: Waypoint[], options: { maxZoom?: number; animate?: boolean; duration?: number; padding?: number } = {}): void => {
        if (points.length <= 1) {
            return;
        }

        const bounds = points.reduce(
            (currentBounds, coordinate) => currentBounds.extend(coordinate),
            new LngLatBounds(points[0], points[0])
        );

        const padding = options.padding ?? (isMapFullscreen ? 60 : 12);

        map.fitBounds(bounds, {
            padding: getMarkerAwarePadding(padding),
            maxZoom: options.maxZoom ?? 15,
            animate: options.animate ?? false,
            duration: options.duration ?? 0
        });
    };

    let mapShellElement: HTMLDivElement | undefined;
    let mapElement: HTMLDivElement | undefined;
    let mapInstance: Map | undefined;
    let resizeObserver: ResizeObserver | undefined;
    let isMapFullscreen = $state(false);
    let iosFullscreen = $state(false);
    let usesIOSFullscreenFallback = $state(false);
    let showFullscreenTapHint = $state(false);
    let fullscreenTapTimer: number | undefined;
    let activePmtilesFiles: string[] = [];

    const clearFullscreenTapHint = (): void => {
        showFullscreenTapHint = false;

        if (fullscreenTapTimer !== undefined) {
            clearTimeout(fullscreenTapTimer);
            fullscreenTapTimer = undefined;
        }
    };

    const activateMapFullscreen = async (): Promise<void> => {
        if (!mapShellElement) {
            return;
        }

        if (usesIOSFullscreenFallback) {
            window.history.pushState({ ...window.history.state, mapFullscreen: true }, "", window.location.href);
            setIOSFullscreen(true);
            return;
        }

        try {
            await mapShellElement.requestFullscreen();
        } catch (error) {
            console.warn("Could not open fullscreen map", error);
        }
    };

    const handleMapTap = (event: MouseEvent): void => {
        if (event.target instanceof Element && event.target.closest(".maplibregl-control-container")) {
            return;
        }

        if (isMapFullscreen) {
            return;
        }

        if (showFullscreenTapHint) {
            clearFullscreenTapHint();
            void activateMapFullscreen();
            return;
        }

        showFullscreenTapHint = true;
        fullscreenTapTimer = window.setTimeout(clearFullscreenTapHint, FULLSCREEN_TAP_WINDOW_MS);
    };

    const setMapInteractionEnabled = (enabled: boolean): void => {
        if (!mapInstance) {
            return;
        }

        const handlers = [
            mapInstance.scrollZoom,
            mapInstance.dragPan,
            mapInstance.dragRotate,
            mapInstance.doubleClickZoom,
            mapInstance.touchZoomRotate,
            mapInstance.boxZoom,
            mapInstance.keyboard
        ];

        for (const handler of handlers) {
            if (handler && typeof handler.enable === "function" && typeof handler.disable === "function") {
                if (enabled) {
                    handler.enable();
                } else {
                    handler.disable();
                }
            }
        }
    };

    const handleFullscreenChange = (): void => {
        if (!mapInstance || !mapShellElement) {
            return;
        }

        clearFullscreenTapHint();

        const fullscreenElement = document.fullscreenElement;
        const isNowMapFullscreen = !!fullscreenElement
            && (fullscreenElement === mapShellElement || mapShellElement.contains(fullscreenElement));

        if (isNowMapFullscreen === isMapFullscreen) {
            return;
        }
        isMapFullscreen = isNowMapFullscreen;
        setMapInteractionEnabled(isMapFullscreen);
        requestAnimationFrame(() => {
            const map = mapInstance;
            if (!map) {
                return;
            }

            map.resize();
            fitMapToWaypoints(map, normalizedWaypoints, { animate: true, duration: 300, padding: isMapFullscreen ? 80 : 12 });
        });
    };

    const setIOSFullscreen = (enabled: boolean): void => {
        clearFullscreenTapHint();
        iosFullscreen = enabled;
        isMapFullscreen = enabled;
        setMapInteractionEnabled(isMapFullscreen);

        requestAnimationFrame(() => {
            const map = mapInstance;
            if (!map) {
                return;
            }

            map.resize();
            fitMapToWaypoints(map, normalizedWaypoints, {
                animate: true,
                duration: 300,
                padding: isMapFullscreen ? 80 : 12
            });
        });
    };

    const toggleIOSFullscreen = (): void => {
        if (!iosFullscreen) {
            window.history.pushState({ ...window.history.state, mapFullscreen: true }, "", window.location.href);
            setIOSFullscreen(true);
            return;
        }

        setIOSFullscreen(false);
        if (window.history.state?.mapFullscreen === true) {
            window.history.back();
        }
    };

    const registerPmtilesProtocol = (baseUrl: string, files: string[]): void => {
        removeProtocol("pmtiles");
        const protocol = new Protocol();
        for (const filename of files) {
            protocol.add(new PMTiles(`${baseUrl}/${filename}`));
        }
        addProtocol("pmtiles", protocol.tile);
    };

    const sameFiles = (a: string[], b: string[]): boolean =>
        a.length === b.length && a.every((filename) => b.includes(filename));

    onMount(() => {
        const handlePopState = (): void => {
            if (iosFullscreen) {
                setIOSFullscreen(false);
            }
        };

        window.addEventListener("popstate", handlePopState);
        if (!mapElement || !mapShellElement) {
            return undefined;
        }

        const mapShell = mapShellElement;
        const mapTarget = mapElement;
        mapTarget.addEventListener("click", handleMapTap);
        setWorkerUrl(workerUrl);
        let cancelled = false;
        const unsubscribeTheme = currentTheme.subscribe((theme) => {
            const palette = getMapPalette(theme);
            mapElement!.style.backgroundColor = palette.background;
            if (mapInstance) {
                applyThemeToMap(mapInstance, activePmtilesFiles, theme);
            }
        });

        const reloadMapTiles = async (): Promise<void> => {
            const map = mapInstance;
            if (!map) {
                return;
            }

            const { baseUrl, validFiles } = await discoverPmtilesBaseUrl();
            if (cancelled || sameFiles(validFiles, activePmtilesFiles)) {
                return;
            }

            activePmtilesFiles = validFiles;
            registerPmtilesProtocol(baseUrl, validFiles);
            map.setStyle({
                version: 8,
                sources: buildSources(baseUrl, validFiles),
                layers: buildLayers(validFiles, get(currentTheme))
            });
        };
        const handleOnline = (): void => {
            reloadMapTiles();
        };

        const handleWindowResize = (): void => {
            const map = mapInstance;
            if (!map) {
                return;
            }

            requestAnimationFrame(() => {
                map.resize();
                if (normalizedWaypoints.length >= 2) {
                    fitMapToWaypoints(map, normalizedWaypoints, { animate: true, duration: 220, padding: isMapFullscreen ? 80 : 12 });
                }
            });
        };

        window.addEventListener("online", handleOnline);
        window.addEventListener("resize", handleWindowResize);

        const initializeMap = async () => {
            const { baseUrl, validFiles } = await discoverPmtilesBaseUrl();

            if (cancelled || !mapElement) {
                return;
            }

            activePmtilesFiles = validFiles;
            registerPmtilesProtocol(baseUrl, validFiles);

            const theme = get(currentTheme);
            const palette = getMapPalette(theme);
            mapElement.style.backgroundColor = palette.background;

            const map = new Map({
                container: mapElement,
                style: {
                    version: 8,
                    sources: buildSources(baseUrl, validFiles),
                    layers: buildLayers(validFiles, theme)
                },
                ...buildInitialMapOptions(normalizedWaypoints)
            });

            mapInstance = map;
            usesIOSFullscreenFallback = typeof mapShell.requestFullscreen !== "function";
            if (!usesIOSFullscreenFallback) {
                map.addControl(new FullscreenControl({ container: mapShell }), "top-right");
            }
            setMapInteractionEnabled(false);
            document.addEventListener("fullscreenchange", handleFullscreenChange);
            resizeObserver = new ResizeObserver(() => {
                requestAnimationFrame(() => map.resize());
            });
            resizeObserver.observe(mapShell);

            map.on("style.load", () => {
                applyThemeToMap(map, activePmtilesFiles, get(currentTheme));

                if (normalizedWaypoints.length >= 2) {
                    addWaypointGeometry(map, normalizedWaypoints);
                    addWaypointMarkers(map, normalizedWaypoints);
                    fitMapToWaypoints(map, normalizedWaypoints);
                }
            });

            map.on("load", () => {
                if (normalizedWaypoints.length >= 2) {
                    addWaypointGeometry(map, normalizedWaypoints);
                }
            });

            map.on("error", (event) => {
                console.error("MAP ERROR:", event);
            });
        };

        initializeMap();

        return () => {
            cancelled = true;
            unsubscribeTheme();
            window.removeEventListener("popstate", handlePopState);
            window.removeEventListener("online", handleOnline);
            window.removeEventListener("resize", handleWindowResize);
            document.removeEventListener("fullscreenchange", handleFullscreenChange);
            mapTarget.removeEventListener("click", handleMapTap);
            clearFullscreenTapHint();
            resizeObserver?.disconnect();
            resizeObserver = undefined;
            mapInstance?.remove();
            mapInstance = undefined;
            removeProtocol("pmtiles");
        };
    });
</script>

<style>
    .map-shell {
        width: 100%;
        height: 100%;
        position: relative;
        overflow: hidden;
    }

    .map-shell.ios-fullscreen {
        position: fixed;
        inset: 0;
        z-index: 1000;
        width: 100vw;
        height: 100dvh;
    }

    .map {
        width: 100%;
        height: 100%;
        touch-action: manipulation;
    }

    .fullscreen-tap-hint {
        position: absolute;
        inset: 0;
        z-index: 2;
        display: grid;
        place-items: center;
        padding: 1rem;
        box-sizing: border-box;
        color: #f2f5f8;
        background: rgba(18, 20, 22, 0.72);
        font-size: 1rem;
        font-weight: 600;
        line-height: 1.3;
        text-align: center;
        pointer-events: none;
        backdrop-filter: blur(6px);
        -webkit-backdrop-filter: blur(6px);
    }

    .map-fullscreen-button {
        position: absolute;
        top: 0.625rem;
        right: 0.625rem;
        z-index: 2;
        display: grid;
        place-items: center;
        width: 2.25rem;
        height: 2.25rem;
        padding: 0;
        color: #202124;
        background: #fff;
        border: 0;
        border-radius: 2px;
        box-shadow: 0 1px 4px rgb(0 0 0 / 30%);
        font-size: 1.5rem;
        line-height: 1;
        cursor: pointer;
    }

    .map-fullscreen-button:focus-visible {
        outline: 2px solid #2563eb;
        outline-offset: 2px;
    }

    .map-shell.ios-fullscreen .map-fullscreen-button {
        position: fixed;
        top: calc(0.625rem + env(safe-area-inset-top));
        right: calc(0.625rem + env(safe-area-inset-right));
        z-index: 1001;
    }
</style>