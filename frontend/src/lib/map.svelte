<script lang="ts">
    import { onMount } from "svelte";
    import {
        FullscreenControl,
        type LayerSpecification,
        LngLatBounds,
        Map,
        Marker,
        type SourceSpecification,
        addProtocol,
        removeProtocol,
        setWorkerUrl
    } from "maplibre-gl";
    import workerUrl from "maplibre-gl/dist/maplibre-gl-worker.mjs?worker&url";
    import { PMTiles, Protocol } from "pmtiles";
    import "maplibre-gl/dist/maplibre-gl.css";
    import type { GpsPosition, Waypoint } from "./types";

    const DEFAULT_CENTER: Waypoint = [8.472401705884762, 49.363691016649035];
    const PMTILES_MAGIC_NUMBER = 19792;
    const pmtilesFiles = [
        "germany.pmtiles",
        "france.pmtiles",
        "belgium.pmtiles",
        "netherlands.pmtiles",
        "poland.pmtiles",
        "denmark.pmtiles",
        "czech_republic.pmtiles",
        "luxembourg.pmtiles"
    ];

    let { waypoints = [] }: { waypoints: GpsPosition[] } = $props();

    const normalizeWaypoints = (points: GpsPosition[]): Waypoint[] =>
        points
            .map((point) => [point.lon, point.lat] as Waypoint)
            .filter(([longitude, latitude]) => !isNaN(longitude) && !isNaN(latitude));

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

    const buildSources = (baseUrl: string, files: string[]): { [_: string]: SourceSpecification } =>
        Object.fromEntries(
            files.map((filename) => {
                const sourceName = filename.replace(/\.pmtiles$/, "");
                return [
                    sourceName,
                    { type: "vector", url: `pmtiles://${baseUrl}/${filename}` } as SourceSpecification
                ];
            })
        );

    const buildLayers = (files: string[]): LayerSpecification[] => [
        {
            id: "background",
            type: "background",
            paint: { "background-color": "#d8e8d0" }
        } as LayerSpecification,
        ...files.flatMap((filename) => {
            const sourceName = filename.replace(/\.pmtiles$/, "");

            return [
                {
                    id: `${sourceName}-landuse`,
                    type: "fill",
                    source: sourceName,
                    "source-layer": "landuse",
                    paint: { "fill-color": "#d8e8d0" }
                } as LayerSpecification,
                {
                    id: `${sourceName}-water`,
                    type: "fill",
                    source: sourceName,
                    "source-layer": "water",
                    paint: { "fill-color": "#8fc8e8" }
                } as LayerSpecification,
                {
                    id: `${sourceName}-roads`,
                    type: "line",
                    source: sourceName,
                    "source-layer": "transportation",
                    paint: {
                        "line-color": "#ffffff",
                        "line-width": ["interpolate", ["linear"], ["zoom"], 5, 0.5, 10, 1.5, 14, 3, 18, 8]
                    }
                } as LayerSpecification
            ];
        })
    ];

    const verifyPmtilesFile = async (url: string): Promise<boolean> => {
        try {
            const response = await fetch(url, { headers: { Range: "bytes=0-1" } });
            if (!response.ok) {
                return false;
            }

            const bytes = await response.arrayBuffer();
            if (bytes.byteLength < 2) {
                return false;
            }

            return new DataView(bytes).getUint16(0, true) === PMTILES_MAGIC_NUMBER;
        } catch {
            return false;
        }
    };

    const discoverPmtilesBaseUrl = async (): Promise<{ baseUrl: string; validFiles: string[] }> => {
        const origins = [...new Set([
            window.location.origin,
            `${window.location.protocol}//${window.location.hostname}:3000`
        ])];

        for (const origin of origins) {
            const baseUrl = `${origin}/maps`;
            const validFiles = (
                await Promise.all(
                    pmtilesFiles.map(async (filename) => {
                        const isValid = await verifyPmtilesFile(`${baseUrl}/${filename}`);
                        return isValid ? filename : null;
                    })
                )
            ).filter((filename): filename is string => filename !== null);

            if (validFiles.length > 0) {
                return { baseUrl, validFiles };
            }
        }

        return { baseUrl: `${origins[0]}/maps`, validFiles: [] };
    };

    const addWaypointGeometry = (map: Map, points: Waypoint[]): void => {
        if (points.length < 2) {
            return;
        }

        map.addSource("waypoint-path", {
            type: "geojson",
            data: {
                type: "Feature",
                geometry: { type: "LineString", coordinates: points },
                properties: {}
            }
        });

        map.addLayer({
            id: "waypoint-path",
            type: "line",
            source: "waypoint-path",
            paint: {
                "line-color": "#e53935",
                "line-width": 5,
                "line-opacity": 0.9
            }
        });
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
                padding: 50,
                maxZoom: 15,
                animate: false,
                duration: 0
            }
        };
    };

    const fitMapToWaypoints = (map: Map, points: Waypoint[], options: { maxZoom?: number; animate?: boolean; duration?: number } = {}): void => {
        if (points.length <= 1) {
            return;
        }

        const bounds = points.reduce(
            (currentBounds, coordinate) => currentBounds.extend(coordinate),
            new LngLatBounds(points[0], points[0])
        );

        map.fitBounds(bounds, {
            padding: 50,
            maxZoom: options.maxZoom ?? 15,
            animate: options.animate ?? false,
            duration: options.duration ?? 0
        });
    };

    let mapElement: HTMLDivElement | undefined;
    let mapShell: HTMLDivElement | undefined;
    let mapInstance: Map | undefined;

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
        if (!mapInstance || !mapElement) {
            return;
        }

        const map = mapInstance;

        const isFullscreen = !!document.fullscreenElement;
        setMapInteractionEnabled(isFullscreen);

        requestAnimationFrame(() => {
            map.resize();

            if (isFullscreen) {
                fitMapToWaypoints(map, normalizedWaypoints, {
                    maxZoom: 17,
                    animate: true,
                    duration: 350
                });
                return;
            }

            fitMapToWaypoints(map, normalizedWaypoints, {
                maxZoom: 15,
                animate: true,
                duration: 350
            });
        });
    };

    onMount(() => {
        if (!mapElement) {
            return undefined;
        }

        setWorkerUrl(workerUrl);
        let cancelled = false;

        const initializeMap = async () => {
            const { baseUrl, validFiles } = await discoverPmtilesBaseUrl();

            if (cancelled || !mapElement) {
                return;
            }

            const protocol = new Protocol();
            for (const filename of validFiles) {
                protocol.add(new PMTiles(`${baseUrl}/${filename}`));
            }
            addProtocol("pmtiles", protocol.tile);

            const map = new Map({
                container: mapElement,
                style: {
                    version: 8,
                    sources: buildSources(baseUrl, validFiles),
                    layers: buildLayers(validFiles)
                },
                ...buildInitialMapOptions(normalizedWaypoints)
            });

            mapInstance = map;
            map.addControl(new FullscreenControl(), "top-right");
            setMapInteractionEnabled(false);
            document.addEventListener("fullscreenchange", handleFullscreenChange);

            map.on("style.load", () => {
                if (normalizedWaypoints.length >= 2) {
                    addWaypointGeometry(map, normalizedWaypoints);
                    fitMapToWaypoints(map, normalizedWaypoints);
                }
            });

            map.on("load", () => {
                if (normalizedWaypoints.length >= 2) {
                    addWaypointMarkers(map, normalizedWaypoints);
                }
            });

            map.on("error", (event) => {
                console.error("MAP ERROR:", event);
            });
        };

        initializeMap();

        return () => {
            cancelled = true;
            document.removeEventListener("fullscreenchange", handleFullscreenChange);
            mapInstance?.remove();
            mapInstance = undefined;
            removeProtocol("pmtiles");
        };
    });
</script>

<div bind:this={mapShell} class="map-shell">
    <div bind:this={mapElement} class="map"></div>
</div>

<style>
    .map-shell {
        width: 100%;
        height: 100%;
        background: #d8e8d0;
    }

    .map {
        width: 100%;
        height: 100%;
        background: #d8e8d0;
    }
</style>