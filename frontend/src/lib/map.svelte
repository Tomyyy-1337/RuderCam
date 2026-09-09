<script>
    import { onMount } from "svelte";
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

    const DEFAULT_CENTER = [8.472401705884762, 49.363691016649035];
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

    let { waypoints = [] } = $props();

    const normalizeWaypoints = (points) =>
        (Array.isArray(points) ? points : []).flatMap((point) => {
            if (
                Array.isArray(point) &&
                point.length >= 2 &&
                Number.isFinite(point[0]) &&
                Number.isFinite(point[1])
            ) {
                return [[Number(point[0]), Number(point[1])]];
            }

            if (point && Number.isFinite(point.lon) && Number.isFinite(point.lat)) {
                return [[Number(point.lon), Number(point.lat)]];
            }

            return [];
        });

    let normalizedWaypoints = $derived(normalizeWaypoints(waypoints));

    const getWaypointCenter = (points) => {
        if (!Array.isArray(points) || points.length === 0) {
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

        return [
            total.longitude / points.length,
            total.latitude / points.length
        ];
    };

    const buildSources = (baseUrl, files) =>
        Object.fromEntries(
            files.map((filename) => {
                const name = filename.replace(/\.pmtiles$/, "");
                const url = `${baseUrl}/${filename}`;

                return [
                    name,
                    {
                        type: "vector",
                        url: `pmtiles://${url}`
                    }
                ];
            })
        );

    const buildLayers = (files) => [
        {
            id: "background",
            type: "background",
            paint: {
                "background-color": "#d9d9d9"
            }
        },
        ...files.flatMap((filename) => {
            const source = filename.replace(/\.pmtiles$/, "");

            return [
                {
                    id: `${source}-landuse`,
                    type: "fill",
                    source,
                    "source-layer": "landuse",
                    paint: {
                        "fill-color": "#d8e8d0"
                    }
                },
                {
                    id: `${source}-water`,
                    type: "fill",
                    source,
                    "source-layer": "water",
                    paint: {
                        "fill-color": "#8fc8e8"
                    }
                },
                {
                    id: `${source}-roads`,
                    type: "line",
                    source,
                    "source-layer": "transportation",
                    paint: {
                        "line-color": "#ffffff",
                        "line-width": [
                            "interpolate",
                            ["linear"],
                            ["zoom"],
                            5, 0.5,
                            10, 1.5,
                            14, 3,
                            18, 8
                        ]
                    }
                }
            ];
        })
    ];

    const verifyPmtilesFile = async (url) => {
        try {
            const response = await fetch(url, {
                headers: {
                    Range: "bytes=0-1"
                }
            });

            if (!response.ok) {
                return false;
            }

            const bytes = await response.arrayBuffer();

            if (bytes.byteLength < 2) {
                return false;
            }

            const magicNumber = new DataView(bytes).getUint16(0, true);
            return magicNumber === PMTILES_MAGIC_NUMBER;
        } catch {
            return false;
        }
    };

    const discoverPmtilesBaseUrl = async () => {
        const origins = [...new Set([
            window.location.origin,
            `${window.location.protocol}//${window.location.hostname}:3000`
        ])];

        for (const origin of origins) {
            const baseUrl = `${origin}/maps`;
            const checks = await Promise.all(
                pmtilesFiles.map(async (filename) => {
                    const url = `${baseUrl}/${filename}`;
                    const isValid = await verifyPmtilesFile(url);

                    return isValid ? filename : null;
                })
            );
            const validFiles = checks.filter((filename) => filename !== null);

            if (validFiles.length > 0) {
                return {
                    baseUrl,
                    validFiles
                };
            }
        }

        return {
            baseUrl: `${origins[0]}/maps`,
            validFiles: []
        };
    };

    const addWaypointGeometry = (map, points) => {
        if (points.length < 2) {
            return;
        }

        map.addSource("waypoint-path", {
            type: "geojson",
            data: {
                type: "Feature",
                geometry: {
                    type: "LineString",
                    coordinates: points
                },
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

    const addWaypointMarkers = (map, points) => {
        points.forEach((coordinate, index) => {
            if (!coordinate || (index !== 0 && index !== points.length - 1)) {
                return;
            }

            new Marker({
                color: index === 0 ? "#00a000" : "#d00000"
            })
                .setLngLat(coordinate)
                .addTo(map);
        });
    };

    const fitMapToWaypoints = (map, points) => {
        if (points.length <= 1) {
            return false;
        }

        const container = map.getContainer();
        const width = container?.clientWidth ?? 0;
        const height = container?.clientHeight ?? 0;
        const padding = 50;

        // fitBounds throws when the canvas is smaller than requested padding.
        if (width <= (padding * 2) + 2 || height <= (padding * 2) + 2) {
            return false;
        }

        const firstCoordinate = points[0] ?? DEFAULT_CENTER;
        const bounds = points.reduce(
            (currentBounds, coordinate) => currentBounds.extend(coordinate),
            new LngLatBounds(firstCoordinate, firstCoordinate)
        );

        try {
            map.fitBounds(bounds, {
                padding,
                maxZoom: 15
            });
            return true;
        } catch {
            return false;
        }
    };

    const canMeasureMapElement = () => {
        if (!mapElement) {
            return false;
        }

        return mapElement.clientWidth > 0 && mapElement.clientHeight > 0;
    };

    /** @type {HTMLElement | undefined} */
    let mapElement;
    /** @type {HTMLElement | undefined} */
    let mapShell;
    /** @type {Map | undefined} */
    let mapInstance;
    /** @type {{ center: [number, number], zoom: number, bearing: number, pitch: number } | null} */
    let previousCamera = null;
    /** @type {{ x: number, y: number } | null} */
    let previousScroll = null;
    /** @type {ResizeObserver | null} */
    let mapResizeObserver = null;
    /** @type {number | null} */
    let pendingRefitTimeout = null;
    let pendingRefitRaf = 0;
    let refitRetryCount = 0;
    const MAX_REFIT_RETRIES = 8;

    const clearPendingRefit = () => {
        if (pendingRefitTimeout !== null) {
            window.clearTimeout(pendingRefitTimeout);
            pendingRefitTimeout = null;
        }

        if (pendingRefitRaf) {
            window.cancelAnimationFrame(pendingRefitRaf);
            pendingRefitRaf = 0;
        }
    };

    const resizeAndFitMap = () => {
        if (!mapInstance || !canMeasureMapElement()) {
            return;
        }

        mapInstance.resize();
        const activeMap = mapInstance;

        // Wait one frame so the map transform uses the resized canvas before fitting bounds.
        pendingRefitRaf = window.requestAnimationFrame(() => {
            pendingRefitRaf = 0;

            if (!mapInstance || mapInstance !== activeMap || !canMeasureMapElement()) {
                return;
            }

            if (fitMapToWaypoints(activeMap, normalizedWaypoints)) {
                refitRetryCount = 0;
                return;
            }

            if (refitRetryCount < MAX_REFIT_RETRIES) {
                refitRetryCount += 1;
                // Retry shortly after transitions or layout changes complete.
                scheduleMapRefit(120);
            }
        });
    };

    const scheduleMapRefit = (delay = 60) => {
        clearPendingRefit();

        pendingRefitTimeout = window.setTimeout(() => {
            pendingRefitTimeout = null;

            requestAnimationFrame(() => {
                resizeAndFitMap();
            });
        }, delay);
    };

    const handleViewportChange = () => {
        refitRetryCount = 0;
        scheduleMapRefit();
    };

    const captureScrollPosition = () => {
        previousScroll = {
            x: window.scrollX,
            y: window.scrollY
        };
    };

    /** @param {MouseEvent | PointerEvent} event */
    const rememberScrollBeforeFullscreen = (event) => {
        const target = event.target;

        if (!(target instanceof Element)) {
            return;
        }

        if (target.closest(".maplibregl-ctrl-fullscreen")) {
            captureScrollPosition();
        }
    };

    /** @param {boolean} enabled */
    const setMapInteractionEnabled = (enabled) => {
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

    const updateFullscreenState = () => {
        const isFullscreen = document.fullscreenElement === mapElement ||
            document.fullscreenElement === mapShell;

        if (isFullscreen && !previousCamera && mapInstance) {
            previousCamera = {
                center: mapInstance.getCenter().toArray(),
                zoom: mapInstance.getZoom(),
                bearing: mapInstance.getBearing(),
                pitch: mapInstance.getPitch()
            };
        }

        if (isFullscreen && !previousScroll) {
            captureScrollPosition();
        }

        setMapInteractionEnabled(isFullscreen);

        if (mapInstance) {
            const activeMap = mapInstance;

            requestAnimationFrame(() => {
                activeMap.resize();
                scheduleMapRefit(0);

                if (!isFullscreen && previousCamera) {
                    activeMap.jumpTo({
                        center: previousCamera.center,
                        zoom: previousCamera.zoom,
                        bearing: previousCamera.bearing,
                        pitch: previousCamera.pitch
                    });
                    previousCamera = null;
                }

                if (!isFullscreen && previousScroll) {
                    const scrollPosition = previousScroll;

                    requestAnimationFrame(() => {
                        window.scrollTo({
                            left: scrollPosition.x,
                            top: scrollPosition.y,
                            behavior: "auto"
                        });
                    });

                    previousScroll = null;
                }
            });
        }
    };

    onMount(() => {
        if (!mapElement || !mapShell) {
            return undefined;
        }

        setWorkerUrl(workerUrl);

        let cancelled = false;

        const initializeMap = async () => {
            const { baseUrl, validFiles } = await discoverPmtilesBaseUrl();

            if (cancelled || !mapElement || !mapShell) {
                return;
            }

            const protocol = new Protocol();

            for (const filename of validFiles) {
                const url = `${baseUrl}/${filename}`;
                protocol.add(new PMTiles(url));
            }

            addProtocol("pmtiles", protocol.tile);

            const map = new Map({
                container: mapElement,
                style: {
                    version: 8,
                    sources: buildSources(baseUrl, validFiles),
                    layers: buildLayers(validFiles)
                },
                center: getWaypointCenter(normalizedWaypoints),
                zoom: 12
            });

            mapInstance = map;
            map.addControl(new FullscreenControl(), "top-right");
            setMapInteractionEnabled(false);
            mapShell.addEventListener("pointerdown", rememberScrollBeforeFullscreen, true);
            mapShell.addEventListener("click", rememberScrollBeforeFullscreen, true);
            document.addEventListener("fullscreenchange", updateFullscreenState);
            window.addEventListener("resize", handleViewportChange);
            document.addEventListener("visibilitychange", handleViewportChange);

            if (typeof ResizeObserver !== "undefined") {
                mapResizeObserver = new ResizeObserver(() => {
                    scheduleMapRefit();
                });
                mapResizeObserver.observe(mapElement);
                mapResizeObserver.observe(mapShell);
            }

            updateFullscreenState();

            map.on("load", () => {
                addWaypointGeometry(map, normalizedWaypoints);
                addWaypointMarkers(map, normalizedWaypoints);
                fitMapToWaypoints(map, normalizedWaypoints);
                refitRetryCount = 0;
                scheduleMapRefit(0);
                scheduleMapRefit(180);
            });

            map.on("error", (event) => {
                console.error("MAP ERROR:", event);
            });
        };

        initializeMap();

        return () => {
            cancelled = true;
            clearPendingRefit();
            refitRetryCount = 0;
            mapShell.removeEventListener("pointerdown", rememberScrollBeforeFullscreen, true);
            mapShell.removeEventListener("click", rememberScrollBeforeFullscreen, true);
            document.removeEventListener("fullscreenchange", updateFullscreenState);
            window.removeEventListener("resize", handleViewportChange);
            document.removeEventListener("visibilitychange", handleViewportChange);
            mapResizeObserver?.disconnect();
            mapResizeObserver = null;
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
    :global(html),
    :global(body) {
        margin: 0;
        padding: 0;
        width: 100%;
        height: 100%;
    }

    :global(body) {
        overflow: auto;
    }

    .map-shell {
        position: relative;
        width: 100%;
        height: 100%;
    }

    .map {
        width: 100%;
        height: 100%;
    }

    .map-shell:fullscreen {
        background: #d9d9d9;
    }

    .map-shell:fullscreen .map {
        width: 100vw;
        height: 100vh;
    }
</style>