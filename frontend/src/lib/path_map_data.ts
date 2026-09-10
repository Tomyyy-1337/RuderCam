import type { GpsPosition, ProjectedGpsPosition } from './types'

export function projectGpsCoordinates(
    points: GpsPosition[],
    padding = 8,
): ProjectedGpsPosition[] {
    const validPoints = Array.isArray(points)
        ? points.filter(
            (point): point is GpsPosition =>
                !!point && Number.isFinite(point.lat) && Number.isFinite(point.lon),
        )
        : [];

    if (validPoints.length === 0) {
        return [];
    }

    let minLat = Number.POSITIVE_INFINITY;
    let maxLat = Number.NEGATIVE_INFINITY;
    let minLon = Number.POSITIVE_INFINITY;
    let maxLon = Number.NEGATIVE_INFINITY;

    for (const point of validPoints) {
        minLat = Math.min(minLat, Number(point.lat));
        maxLat = Math.max(maxLat, Number(point.lat));
        minLon = Math.min(minLon, Number(point.lon));
        maxLon = Math.max(maxLon, Number(point.lon));
    }

    const latSpan = maxLat - minLat;
    const lonSpan = maxLon - minLon;
    const inset = Math.max(0, Math.min(49, Number(padding) || 0));
    const usableSize = 100 - inset * 2;

    return validPoints.map((point) => {
        const lat = Number(point.lat);
        const lon = Number(point.lon);
        const lonRatio = lonSpan === 0 ? 0.5 : (lon - minLon) / lonSpan;
        const latRatio = latSpan === 0 ? 0.5 : (lat - minLat) / latSpan;
        const x = inset + lonRatio * usableSize;
        const y = 100 - (inset + latRatio * usableSize);

        return {
            lat,
            lon,
            x: Number(x.toFixed(4)),
            y: Number(y.toFixed(4)),
            speed_kmh: point.speed_kmh,
        };
    });
}

export function buildSmoothedRoutePath(points: ProjectedGpsPosition[]): string {
    if (!Array.isArray(points) || points.length === 0) {
        return '';
    }

    if (points.length === 1) {
        const [{ x, y }] = points;
        return `M ${x} ${y}`;
    }

    const toPathNumber = (value: number) => Number(Number(value).toFixed(4));
    const firstPoint = points[0];
    let path = `M ${toPathNumber(firstPoint.x)} ${toPathNumber(firstPoint.y)}`;

    for (let index = 0; index < points.length - 1; index += 1) {
        const previousPoint = points[index - 1] ?? points[index];
        const currentPoint = points[index];
        const nextPoint = points[index + 1];
        const afterNextPoint = points[index + 2] ?? nextPoint;

        const controlPoint1X = toPathNumber(currentPoint.x + (nextPoint.x - previousPoint.x) / 6);
        const controlPoint1Y = toPathNumber(currentPoint.y + (nextPoint.y - previousPoint.y) / 6);
        const controlPoint2X = toPathNumber(nextPoint.x - (afterNextPoint.x - currentPoint.x) / 6);
        const controlPoint2Y = toPathNumber(nextPoint.y - (afterNextPoint.y - currentPoint.y) / 6);

        path += ` C ${controlPoint1X} ${controlPoint1Y}, ${controlPoint2X} ${controlPoint2Y}, ${toPathNumber(nextPoint.x)} ${toPathNumber(nextPoint.y)}`;
    }

    return path;
}