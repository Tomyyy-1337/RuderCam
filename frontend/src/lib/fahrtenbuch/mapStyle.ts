import type { LayerSpecification, Map, SourceSpecification } from "maplibre-gl";
import type { Theme } from "../types";

export interface MapPalette {
    background: string;
    water: string;
    waterOutline: string;
    roadMotorway: string;
    roadTrunk: string;
    roadPrimary: string;
    roadSecondary: string;
    roadOther: string;
    textCapital: string;
    textTown: string;
    textVillage: string;
    textHalo: string;
    landuse: {
        residential: string;
        suburb: string;
        commercial: string;
        industrial: string;
        hospital: string;
        military: string;
        quarry: string;
        themePark: string;
        cemetery: string;
        track: string;
        fallback: string;
    };
}

export const getMapPalette = (theme: Theme): MapPalette => {
    if (theme === "dark") {
        return {
            background: "#121518",
            water: "#2f4f63",
            waterOutline: "#4f748a",
            roadMotorway: "#8b9aa6",
            roadTrunk: "#7f8d97",
            roadPrimary: "#73808b",
            roadSecondary: "#6a757f",
            roadOther: "#5f6973",
            textCapital: "#f0f3f6",
            textTown: "#dbe1e7",
            textVillage: "#c8d0d8",
            textHalo: "#0d1013",
            landuse: {
                residential: "#22262b",
                suburb: "#1f2429",
                commercial: "#2a2f35",
                industrial: "#30343a",
                hospital: "#3a3030",
                military: "#2e3238",
                quarry: "#3a3f45",
                themePark: "#294036",
                cemetery: "#314238",
                track: "#3a332b",
                fallback: "#242a30"
            }
        };
    }

    return {
        background: "#e0e0e0",
        water: "#5da8d4",
        waterOutline: "#3a7fa0",
        roadMotorway: "#888888",
        roadTrunk: "#999999",
        roadPrimary: "#a0a0a0",
        roadSecondary: "#ffffff",
        roadOther: "#ffffff",
        textCapital: "#000000",
        textTown: "#0a0a0a",
        textVillage: "#1a1a1a",
        textHalo: "#e8e8e8",
        landuse: {
            residential: "#d9d6cf",
            suburb: "#d3d0c9",
            commercial: "#ccc9c2",
            industrial: "#b8b2ab",
            hospital: "#e8c8b8",
            military: "#a8a8a8",
            quarry: "#959595",
            themePark: "#6ba86b",
            cemetery: "#a8c8a8",
            track: "#c8b89c",
            fallback: "#c8c8c8"
        }
    };
};

const buildLanduseColorExpression = (palette: MapPalette): unknown[] => [
    "match",
    ["get", "class"],
    "residential", palette.landuse.residential,
    "suburb", palette.landuse.suburb,
    "neighbourhood", palette.landuse.suburb,
    "commercial", palette.landuse.commercial,
    "retail", palette.landuse.commercial,
    "industrial", palette.landuse.industrial,
    "hospital", palette.landuse.hospital,
    "military", palette.landuse.military,
    "quarry", palette.landuse.quarry,
    "theme_park", palette.landuse.themePark,
    "cemetery", palette.landuse.cemetery,
    "track", palette.landuse.track,
    palette.landuse.fallback
];

const pmtilesSourceName = (filename: string): string => filename.replace(/\.pmtiles$/, "");

export const buildSources = (baseUrl: string, files: string[]): { [_: string]: SourceSpecification } => {
    const pmtilesSources = Object.fromEntries(
        files.map((filename) => [
            pmtilesSourceName(filename),
            { type: "vector", url: `pmtiles://${baseUrl}/${filename}` } as SourceSpecification
        ])
    );

    // Add GeoJSON source for waypoint path (always present, even without map data)
    return {
        ...pmtilesSources,
        "waypoint-path": {
            type: "geojson",
            data: {
                type: "Feature",
                geometry: { type: "LineString", coordinates: [] },
                properties: {}
            }
        } as SourceSpecification
    };
};

const buildSourceLayers = (sourceName: string, palette: MapPalette): LayerSpecification[] => [
    {
        id: `${sourceName}-landuse`,
        type: "fill",
        source: sourceName,
        "source-layer": "landuse",
        paint: {
            "fill-color": buildLanduseColorExpression(palette),
            "fill-opacity": 1
        }
    } as LayerSpecification,
    {
        id: `${sourceName}-water`,
        type: "fill",
        source: sourceName,
        "source-layer": "water",
        paint: {
            "fill-color": palette.water,
            "fill-opacity": 0.95
        }
    } as LayerSpecification,
    {
        id: `${sourceName}-water-outline`,
        type: "line",
        source: sourceName,
        "source-layer": "water",
        paint: {
            "line-color": palette.waterOutline,
            "line-width": ["interpolate", ["linear"], ["zoom"], 4, 0.5, 8, 0.5, 14, 1.5]
        }
    } as LayerSpecification,
    {
        id: `${sourceName}-motorway`,
        type: "line",
        source: sourceName,
        "source-layer": "transportation",
        filter: ["==", ["get", "class"], "motorway"],
        layout: { "line-join": "round", "line-cap": "round" },
        paint: {
            "line-color": palette.roadMotorway,
            "line-width": ["interpolate", ["linear"], ["zoom"], 5, 0.5, 10, 2, 14, 4, 18, 10]
        }
    } as LayerSpecification,
    {
        id: `${sourceName}-trunk`,
        type: "line",
        source: sourceName,
        "source-layer": "transportation",
        filter: ["==", ["get", "class"], "trunk"],
        layout: { "line-join": "round", "line-cap": "round" },
        paint: {
            "line-color": palette.roadTrunk,
            "line-width": ["interpolate", ["linear"], ["zoom"], 5, 0.4, 10, 1.5, 14, 3, 18, 8]
        }
    } as LayerSpecification,
    {
        id: `${sourceName}-primary`,
        type: "line",
        source: sourceName,
        "source-layer": "transportation",
        filter: ["==", ["get", "class"], "primary"],
        layout: { "line-join": "round", "line-cap": "round" },
        paint: {
            "line-color": palette.roadPrimary,
            "line-width": ["interpolate", ["linear"], ["zoom"], 8, 0.5, 12, 1.5, 16, 3]
        }
    } as LayerSpecification,
    {
        id: `${sourceName}-secondary`,
        type: "line",
        source: sourceName,
        "source-layer": "transportation",
        filter: ["==", ["get", "class"], "secondary"],
        layout: { "line-join": "round", "line-cap": "round" },
        paint: {
            "line-color": palette.roadSecondary,
            "line-width": ["interpolate", ["linear"], ["zoom"], 10, 0.5, 14, 1.5, 18, 4]
        }
    } as LayerSpecification,
    {
        id: `${sourceName}-roads-other`,
        type: "line",
        source: sourceName,
        "source-layer": "transportation",
        filter: ["all", ["!=", ["get", "class"], "motorway"], ["!=", ["get", "class"], "trunk"], ["!=", ["get", "class"], "primary"], ["!=", ["get", "class"], "secondary"]],
        paint: {
            "line-color": palette.roadOther,
            "line-width": ["interpolate", ["linear"], ["zoom"], 12, 0.3, 16, 0.8, 18, 2],
            "line-opacity": ["interpolate", ["linear"], ["zoom"], 10, 0.3, 14, 0.7]
        }
    } as LayerSpecification,
    {
        id: `${sourceName}-places-capital`,
        type: "symbol",
        source: sourceName,
        "source-layer": "place",
        filter: ["==", ["get", "class"], "city"],
        layout: {
            "text-field": ["get", "name"],
            "text-font": ["Open Sans Bold", "Arial Unicode MS Bold"],
            "text-size": ["interpolate", ["linear"], ["zoom"], 5, 12, 10, 16, 15, 22],
            "text-offset": [0, 0.5],
            "text-anchor": "center",
            "text-max-width": 10
        },
        paint: {
            "text-color": palette.textCapital,
            "text-halo-color": palette.textHalo,
            "text-halo-width": 2.5
        }
    } as LayerSpecification,
    {
        id: `${sourceName}-places-town`,
        type: "symbol",
        source: sourceName,
        "source-layer": "place",
        filter: ["==", ["get", "class"], "town"],
        layout: {
            "text-field": ["get", "name"],
            "text-font": ["Open Sans SemiBold", "Arial Unicode MS Bold"],
            "text-size": ["interpolate", ["linear"], ["zoom"], 8, 10, 12, 14, 16, 18],
            "text-offset": [0, 0.3],
            "text-anchor": "center",
            "text-max-width": 8
        },
        paint: {
            "text-color": palette.textTown,
            "text-halo-color": palette.textHalo,
            "text-halo-width": 2
        }
    } as LayerSpecification,
    {
        id: `${sourceName}-places-village`,
        type: "symbol",
        source: sourceName,
        "source-layer": "place",
        filter: ["==", ["get", "class"], "village"],
        layout: {
            "text-field": ["get", "name"],
            "text-font": ["Open Sans Regular", "Arial Unicode MS Regular"],
            "text-size": ["interpolate", ["linear"], ["zoom"], 10, 9, 14, 12, 18, 14],
            "text-offset": [0, 0.2],
            "text-anchor": "center",
            "text-max-width": 7
        },
        paint: {
            "text-color": palette.textVillage,
            "text-halo-color": palette.textHalo,
            "text-halo-width": 1.5
        }
    } as LayerSpecification
];

export const buildLayers = (files: string[], theme: Theme): LayerSpecification[] => {
    const palette = getMapPalette(theme);

    return [
        {
            id: "background",
            type: "background",
            paint: { "background-color": palette.background }
        } as LayerSpecification,
        ...files.flatMap((filename) => buildSourceLayers(pmtilesSourceName(filename), palette)),
        // Waypoint path layer (rendered on top of all other layers)
        {
            id: "waypoint-path",
            type: "line",
            source: "waypoint-path",
            paint: {
                "line-color": "#e53935",
                "line-width": 5,
                "line-opacity": 0.9
            }
        } as LayerSpecification
    ];
};

export const applyThemeToMap = (map: Map, files: string[], theme: Theme): void => {
    const palette = getMapPalette(theme);
    const landuseColorExpression = buildLanduseColorExpression(palette);

    const setPaint = (layerId: string, property: string, value: unknown) => {
        if (map.getLayer(layerId)) {
            map.setPaintProperty(layerId, property as any, value);
        }
    };

    setPaint("background", "background-color", palette.background);

    for (const filename of files) {
        const sourceName = pmtilesSourceName(filename);

        setPaint(`${sourceName}-landuse`, "fill-color", landuseColorExpression);
        setPaint(`${sourceName}-water`, "fill-color", palette.water);
        setPaint(`${sourceName}-water-outline`, "line-color", palette.waterOutline);
        setPaint(`${sourceName}-motorway`, "line-color", palette.roadMotorway);
        setPaint(`${sourceName}-trunk`, "line-color", palette.roadTrunk);
        setPaint(`${sourceName}-primary`, "line-color", palette.roadPrimary);
        setPaint(`${sourceName}-secondary`, "line-color", palette.roadSecondary);
        setPaint(`${sourceName}-roads-other`, "line-color", palette.roadOther);
        setPaint(`${sourceName}-places-capital`, "text-color", palette.textCapital);
        setPaint(`${sourceName}-places-capital`, "text-halo-color", palette.textHalo);
        setPaint(`${sourceName}-places-town`, "text-color", palette.textTown);
        setPaint(`${sourceName}-places-town`, "text-halo-color", palette.textHalo);
        setPaint(`${sourceName}-places-village`, "text-color", palette.textVillage);
        setPaint(`${sourceName}-places-village`, "text-halo-color", palette.textHalo);
    }
};
