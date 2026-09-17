// Draws the livestream stat overlay directly onto a canvas 2D context.
// This mirrors the styling of the former HTML overlay (overlay.svelte / overlay_card.svelte)
// so that the whole stream + overlay can be captured as a single canvas element,
// which sidesteps iOS Safari's forced-native-fullscreen behaviour for <video> elements.

export interface OverlayCardData {
    id: string;
    label: string;
    value: string;
    unit?: string;
}

export interface OverlayDrawState {
    position: "top" | "bottom";
    cards: OverlayCardData[];
    battery_percentage: number;
    satellite_count: number;
    paused: boolean;
    sessionActive: boolean;
}

const WHITE = "#f2f5f8";
const LIGHT_GRAY = "lightgray";

function batteryTone(value: number): string {
    if (value <= 15) return "#e53935";
    if (value <= 30) return "#fb8c00";
    if (value <= 50) return "#fdd835";
    return "#43a047";
}

function satelliteTone(value: number): string {
    if (value === 0) return "#e53935";
    if (value <= 3) return "#fb8c00";
    if (value <= 6) return "#fdd835";
    return "#43a047";
}

function pauseTone(sessionActive: boolean, paused: boolean): string {
    if (!sessionActive) return "#ae1e1e";
    return paused ? "#fdd835" : "#4caf50";
}

function roundRect(ctx: CanvasRenderingContext2D, x: number, y: number, w: number, h: number, r: number): void {
    const radius = Math.min(r, w / 2, h / 2);
    ctx.beginPath();
    ctx.moveTo(x + radius, y);
    ctx.arcTo(x + w, y, x + w, y + h, radius);
    ctx.arcTo(x + w, y + h, x, y + h, radius);
    ctx.arcTo(x, y + h, x, y, radius);
    ctx.arcTo(x, y, x + w, y, radius);
    ctx.closePath();
}

function drawCardRow(
    ctx: CanvasRenderingContext2D,
    cards: OverlayCardData[],
    x: number,
    y: number,
    w: number,
    h: number,
    scale: number,
): void {
    if (cards.length === 0) return;

    ctx.save();
    ctx.fillStyle = "rgba(0, 0, 0, 0.5)";
    ctx.fillRect(x, y, w, h);

    const padding = 12 * scale;
    const innerW = w - padding * 2;
    const cardW = innerW / cards.length;
    const centerY = y + h / 2;

    ctx.textAlign = "center";
    ctx.textBaseline = "middle";

    cards.forEach((card, index) => {
        const cardCenterX = x + padding + cardW * (index + 0.5);
        const labelSize = Math.max(11, 14 * scale);
        const valueSize = Math.max(20, 34 * scale);
        const unitSize = Math.max(9, 12 * scale);

        ctx.fillStyle = WHITE;
        ctx.font = `bold ${labelSize}px sans-serif`;
        ctx.fillText(card.label, cardCenterX, centerY - valueSize * 0.62, cardW);

        ctx.font = `${valueSize}px sans-serif`;
        ctx.fillText(card.value, cardCenterX, centerY + labelSize * 0.1, cardW);

        if (card.unit) {
            ctx.fillStyle = LIGHT_GRAY;
            ctx.font = `${unitSize}px sans-serif`;
            ctx.fillText(card.unit, cardCenterX, centerY + valueSize * 0.62, cardW);
        }
    });

    ctx.restore();
}

function drawBatteryIcon(ctx: CanvasRenderingContext2D, x: number, y: number, size: number, color: string, percentage: number): void {
    ctx.save();
    ctx.strokeStyle = color;
    ctx.fillStyle = color;
    ctx.lineWidth = Math.max(1, size * 0.09);

    const bodyW = size;
    const bodyH = size * 0.62;
    const bodyY = y - bodyH / 2;

    roundRect(ctx, x, bodyY, bodyW, bodyH, size * 0.12);
    ctx.stroke();

    const tipW = size * 0.1;
    roundRect(ctx, x + bodyW, bodyY + bodyH * 0.22, tipW, bodyH * 0.56, tipW * 0.4);
    ctx.fill();

    const fillPad = size * 0.12;
    const fillW = Math.max(0, Math.min(bodyW - fillPad * 2, ((bodyW - fillPad * 2) * percentage) / 100));
    roundRect(ctx, x + fillPad, bodyY + fillPad, fillW, bodyH - fillPad * 2, size * 0.06);
    ctx.fill();
    ctx.restore();
}

function drawSatelliteIcon(ctx: CanvasRenderingContext2D, x: number, y: number, size: number, color: string): void {
    ctx.save();
    ctx.translate(x + size / 2, y);
    ctx.fillStyle = color;
    ctx.strokeStyle = color;

    ctx.save();
    ctx.rotate(Math.PI / 4);
    const bodySize = size * 0.34;
    ctx.fillRect(-bodySize / 2, -bodySize / 2, bodySize, bodySize);
    ctx.restore();

    const panelW = size * 0.24;
    const panelH = size * 0.7;
    ctx.fillRect(-size * 0.5, -panelH / 2, panelW, panelH);
    ctx.fillRect(size * 0.5 - panelW, -panelH / 2, panelW, panelH);

    ctx.lineWidth = Math.max(1, size * 0.09);
    [0.42, 0.62].forEach((radiusFactor) => {
        ctx.beginPath();
        ctx.arc(size * 0.35, 0, size * radiusFactor, Math.PI * 1.15, Math.PI * 1.55);
        ctx.stroke();
    });

    ctx.restore();
}

function drawPauseIcon(ctx: CanvasRenderingContext2D, x: number, y: number, size: number, color: string, sessionActive: boolean, paused: boolean): void {
    ctx.save();
    ctx.fillStyle = color;

    if (!sessionActive) {
        roundRect(ctx, x, y - size / 2, size, size, size * 0.15);
        ctx.fill();
    } else if (paused) {
        const barW = size * 0.28;
        ctx.fillRect(x + size * 0.12, y - size / 2, barW, size);
        ctx.fillRect(x + size * 0.6, y - size / 2, barW, size);
    } else {
        ctx.beginPath();
        ctx.moveTo(x, y - size / 2);
        ctx.lineTo(x + size, y);
        ctx.lineTo(x, y + size / 2);
        ctx.closePath();
        ctx.fill();
    }

    ctx.restore();
}

function drawControlRow(
    ctx: CanvasRenderingContext2D,
    state: OverlayDrawState,
    x: number,
    y: number,
    h: number,
    scale: number,
): void {
    const iconSize = Math.max(14, 18 * scale);
    const rowGap = h / 3;
    const iconX = x + 16 * scale;
    const textX = iconX + iconSize + 10 * scale;
    const fontSize = Math.max(11, 14 * scale);

    ctx.save();
    ctx.textAlign = "left";
    ctx.textBaseline = "middle";
    ctx.font = `bold ${fontSize}px sans-serif`;

    const batteryColor = batteryTone(state.battery_percentage);
    const batteryY = y + rowGap * 0.5;
    drawBatteryIcon(ctx, iconX, batteryY, iconSize, batteryColor, state.battery_percentage);
    ctx.fillStyle = batteryColor;
    ctx.fillText(`${Math.round(state.battery_percentage)}%`, textX, batteryY);

    const satelliteColor = satelliteTone(state.satellite_count);
    const satelliteY = y + rowGap * 1.5;
    drawSatelliteIcon(ctx, iconX, satelliteY, iconSize, satelliteColor);
    ctx.fillStyle = satelliteColor;
    ctx.fillText(`${state.satellite_count}`, textX, satelliteY);

    const pauseColor = pauseTone(state.sessionActive, state.paused);
    const pauseY = y + rowGap * 2.5;
    drawPauseIcon(ctx, iconX, pauseY, iconSize, pauseColor, state.sessionActive, state.paused);

    ctx.restore();
}

export function drawOverlay(ctx: CanvasRenderingContext2D, width: number, height: number, state: OverlayDrawState): void {
    const scale = Math.min(2.4, Math.max(0.6, height / 480));
    const cardRowHeight = Math.min(height * 0.32, Math.max(64 * scale, height * 0.22));
    const controlRowHeight = Math.min(height * 0.24, Math.max(52 * scale, height * 0.16));

    const cardRowY = state.position === "top" ? 0 : height - cardRowHeight;
    const controlRowY = state.position === "top" ? height - controlRowHeight : 0;

    drawCardRow(ctx, state.cards, 0, cardRowY, width, cardRowHeight, scale);
    drawControlRow(ctx, state, 0, controlRowY, controlRowHeight, scale);
}
