type OnError = (err: string) => void;
type OnTrack = (evt: RTCTrackEvent) => void;
type OnDataChannel = (evt: RTCDataChannelEvent) => void;

interface ReaderConfig {
    url: string;
    user: string;
    pass: string;
    token: string;
    onError?: OnError;
    onTrack?: OnTrack;
    onDataChannel?: OnDataChannel;
}

interface OfferData {
    iceUfrag: string;
    icePwd: string;
    medias: string[];
}

type ReaderState = 'running' | 'failed' | 'closed';
type CandidateBuckets = Record<number, RTCIceCandidate[]>;

interface VideoStatsSnapshot {
    timestamp: number;
    jitterBufferDelay: number;
    jitterBufferEmittedCount: number;
    framesDecoded: number;
    bytesReceived: number;
}

class MediaMTXWebRTCReader {
    static #DISCONNECTED_RETRY_PAUSE = 1000;

    // Signaling requests (ICE servers, offer, trickle ICE) are aborted if they take longer than this,
    // so a stalled request can never block reconnection indefinitely.
    static #FETCH_TIMEOUT = 6000;

    // Small non-zero jitter buffer to absorb periodic I-frame timing spikes (encoder emits a
    // keyframe every ~1s) without a visible micro-freeze, while adding minimal latency.
    static #JITTER_BUFFER_TARGET_MS = 120;

    // WebRTC statistics.
    static #STATS_INTERVAL = 750;

    // Reconnect if interval latency stays above this value for too long.
    static #HIGH_LATENCY_THRESHOLD = 0.9;
    static #HIGH_LATENCY_DURATION = 1800;

    // Reconnect immediately on critical interval latency.
    static #CRITICAL_LATENCY_THRESHOLD = 1.7;

    // If no decode/network progress is observed while connected, reconnect quickly.
    static #NO_PROGRESS_DURATION = 1500;
    static #MIN_PROGRESS_BYTES = 1024;

    #conf: ReaderConfig;
    #state: ReaderState = 'running';
    #disconnectTimeout: number | null = null;
    #statsTimeout: number | null = null;
    #pc: RTCPeerConnection | null = null;
    #offerData: OfferData | null = null;
    #sessionUrl: string | null = null;
    #queuedCandidates: RTCIceCandidate[] = [];
    #pendingControllers: Set<AbortController> = new Set();

    #lastVideoStats: VideoStatsSnapshot | null = null;
    #highLatencySince: number | null = null;
    #noProgressSince: number | null = null;
    #errorHandled = false;

    constructor(conf: ReaderConfig) {
        this.#conf = conf;
        this.#start();
    }

    close() {
        this.#state = 'closed';
        this.#errorHandled = false;
        this.#stopStatsMonitor();
        this.#abortPendingRequests();

        if (this.#disconnectTimeout !== null) {
            clearTimeout(this.#disconnectTimeout);
            this.#disconnectTimeout = null;
        }

        if (this.#pc !== null) {
            this.#pc.onicecandidate = null;
            this.#pc.onconnectionstatechange = null;
            this.#pc.ontrack = null;
            this.#pc.ondatachannel = null;
            this.#pc.close();
            this.#pc = null;
        }

        if (this.#sessionUrl !== null) {
            const sessionUrl = this.#sessionUrl;
            this.#sessionUrl = null;
            this.#deleteSessionBestEffort(sessionUrl);
        }

        this.#offerData = null;
        this.#queuedCandidates = [];
    }

    #abortPendingRequests() {
        for (const controller of this.#pendingControllers) {
            controller.abort(new Error('closed'));
        }

        this.#pendingControllers.clear();
    }

    #deleteSessionBestEffort(sessionUrl: string) {
        const controller = new AbortController();
        const timeoutId = window.setTimeout(() => controller.abort(new Error('timeout')), 3000);

        fetch(sessionUrl, { method: 'DELETE', signal: controller.signal })
            .catch(() => {
                // The session may already have disappeared.
            })
            .finally(() => {
                clearTimeout(timeoutId);
            });
    }

    #fetchWithTimeout(input: string, init: RequestInit, timeoutMs: number): Promise<Response> {
        const controller = new AbortController();
        this.#pendingControllers.add(controller);

        const timeoutId = window.setTimeout(() => {
            controller.abort(new Error('timeout'));
        }, timeoutMs);

        return fetch(input, { ...init, signal: controller.signal }).finally(() => {
            clearTimeout(timeoutId);
            this.#pendingControllers.delete(controller);
        });
    }

    static #unquoteCredential(value: string) {
        return JSON.parse(`"${value}"`) as string;
    }

    static #linkToIceServers(links: string | null): RTCIceServer[] {
        if (links === null) {
            return [];
        }

        return links
            .split(', ')
            .map((link) => {
                const match = link.match(
                    /^<(.+?)>; rel="ice-server"(; username="(.*?)"; credential="(.*?)"; credential-type="password")?/i,
                );

                if (!match) {
                    return null;
                }

                const server: RTCIceServer = {
                    urls: [match[1]],
                };

                if (match[3] !== undefined) {
                    server.username = this.#unquoteCredential(match[3]);
                    server.credential = this.#unquoteCredential(match[4]);
                }

                return server;
            })
            .filter((server): server is RTCIceServer => server !== null);
    }

    static #parseOffer(sdp: string): OfferData {
        const ret: OfferData = {
            iceUfrag: '',
            icePwd: '',
            medias: [],
        };

        for (const line of sdp.split('\r\n')) {
            if (line.startsWith('m=')) {
                ret.medias.push(line.slice('m='.length));
            } else if (ret.iceUfrag === '' && line.startsWith('a=ice-ufrag:')) {
                ret.iceUfrag = line.slice('a=ice-ufrag:'.length);
            } else if (ret.icePwd === '' && line.startsWith('a=ice-pwd:')) {
                ret.icePwd = line.slice('a=ice-pwd:'.length);
            }
        }

        return ret;
    }

    static #generateSdpFragment(offerData: OfferData, candidates: RTCIceCandidate[]) {
        const candidatesByMedia: CandidateBuckets = {};

        for (const candidate of candidates) {
            const mid = candidate.sdpMLineIndex ?? 0;

            if (candidatesByMedia[mid] === undefined) {
                candidatesByMedia[mid] = [];
            }

            candidatesByMedia[mid].push(candidate);
        }

        let fragment = `a=ice-ufrag:${offerData.iceUfrag}\r\n` + `a=ice-pwd:${offerData.icePwd}\r\n`;

        let mid = 0;

        for (const media of offerData.medias) {
            if (candidatesByMedia[mid] !== undefined) {
                fragment += `m=${media}\r\n` + `a=mid:${mid}\r\n`;

                for (const candidate of candidatesByMedia[mid]) {
                    fragment += `a=${candidate.candidate}\r\n`;
                }
            }

            mid++;
        }

        return fragment;
    }

    #startStatsMonitor() {
        this.#stopStatsMonitor();

        const check = async () => {
            if (this.#state !== 'running' || this.#pc === null) {
                return;
            }

            try {
                const stats = await this.#pc.getStats();
                let videoReportFound = false;

                for (const report of stats.values()) {
                    if (report.type !== 'inbound-rtp') {
                        continue;
                    }

                    const kind = report.kind ?? report.mediaType;

                    if (kind !== 'video') {
                        continue;
                    }

                    videoReportFound = true;

                    const emittedCount = report.jitterBufferEmittedCount;
                    const jitterBufferDelay = report.jitterBufferDelay;

                    const currentSnapshot: VideoStatsSnapshot = {
                        timestamp: typeof report.timestamp === 'number' ? report.timestamp : performance.now(),
                        jitterBufferDelay: typeof jitterBufferDelay === 'number' ? jitterBufferDelay : 0,
                        jitterBufferEmittedCount: typeof emittedCount === 'number' ? emittedCount : 0,
                        framesDecoded: typeof report.framesDecoded === 'number' ? report.framesDecoded : 0,
                        bytesReceived: typeof report.bytesReceived === 'number' ? report.bytesReceived : 0,
                    };

                    const previousSnapshot = this.#lastVideoStats;
                    this.#lastVideoStats = currentSnapshot;

                    if (previousSnapshot === null) {
                        break;
                    }

                    const deltaEmittedCount =
                        currentSnapshot.jitterBufferEmittedCount - previousSnapshot.jitterBufferEmittedCount;
                    const deltaJitterBufferDelay =
                        currentSnapshot.jitterBufferDelay - previousSnapshot.jitterBufferDelay;
                    const deltaFramesDecoded =
                        currentSnapshot.framesDecoded - previousSnapshot.framesDecoded;
                    const deltaBytesReceived =
                        currentSnapshot.bytesReceived - previousSnapshot.bytesReceived;

                    const intervalJitterBufferDelay =
                        deltaEmittedCount > 0 && deltaJitterBufferDelay >= 0
                            ? deltaJitterBufferDelay / deltaEmittedCount
                            : null;

                    const hasDecodeProgress = deltaFramesDecoded > 0;
                    const hasNetworkProgress = deltaBytesReceived > MediaMTXWebRTCReader.#MIN_PROGRESS_BYTES;
                    const hasProgress = hasDecodeProgress || hasNetworkProgress;

                    const isConnected = this.#pc?.connectionState === 'connected';

                    if (isConnected && !hasProgress) {
                        if (this.#noProgressSince === null) {
                            this.#noProgressSince = performance.now();
                        }

                        const noProgressDuration = performance.now() - this.#noProgressSince;

                        if (noProgressDuration >= MediaMTXWebRTCReader.#NO_PROGRESS_DURATION) {
                            this.#handleError(
                                `video stalled (${Math.round(noProgressDuration)}ms without progress)`,
                            );
                            return;
                        }
                    } else {
                        this.#noProgressSince = null;
                    }

                    console.debug('[WebRTC]', {
                        intervalJitterBufferDelay,
                        deltaFramesDecoded,
                        deltaBytesReceived,
                        jitter: report.jitter,
                        packetsLost: report.packetsLost,
                        packetsDiscarded: report.packetsDiscarded,
                        framesDecoded: report.framesDecoded,
                        framesPerSecond: report.framesPerSecond,
                    });

                    if (
                        intervalJitterBufferDelay !== null &&
                        intervalJitterBufferDelay >= MediaMTXWebRTCReader.#CRITICAL_LATENCY_THRESHOLD
                    ) {
                        this.#handleError(
                            `video latency too high (${intervalJitterBufferDelay.toFixed(2)}s)`,
                        );
                        return;
                    }

                    if (
                        intervalJitterBufferDelay !== null &&
                        intervalJitterBufferDelay >= MediaMTXWebRTCReader.#HIGH_LATENCY_THRESHOLD
                    ) {
                        if (this.#highLatencySince === null) {
                            this.#highLatencySince = performance.now();
                        }

                        const duration = performance.now() - this.#highLatencySince;

                        if (duration >= MediaMTXWebRTCReader.#HIGH_LATENCY_DURATION) {
                            this.#handleError(
                                `video latency too high (${intervalJitterBufferDelay.toFixed(2)}s for ${Math.round(duration)}ms)`,
                            );
                            return;
                        }
                    } else {
                        this.#highLatencySince = null;
                    }

                    break;
                }

                if (!videoReportFound) {
                    this.#lastVideoStats = null;
                    this.#noProgressSince = null;
                    this.#highLatencySince = null;
                }
            } catch (err) {
                console.warn('[WebRTC] getStats failed:', err);
            }

            if (this.#state === 'running') {
                this.#statsTimeout = window.setTimeout(
                    check,
                    MediaMTXWebRTCReader.#STATS_INTERVAL,
                );
            }
        };

        this.#statsTimeout = window.setTimeout(check, 1000);
    }

    #stopStatsMonitor() {
        if (this.#statsTimeout !== null) {
            clearTimeout(this.#statsTimeout);
            this.#statsTimeout = null;
        }

        this.#lastVideoStats = null;
        this.#highLatencySince = null;
        this.#noProgressSince = null;
    }

    #handleError(err: string) {
        if (this.#state === 'closed' || this.#errorHandled) {
            return;
        }

        this.#errorHandled = true;
        this.#stopStatsMonitor();
        this.#abortPendingRequests();

        if (this.#disconnectTimeout !== null) {
            clearTimeout(this.#disconnectTimeout);
            this.#disconnectTimeout = null;
        }

        if (this.#pc !== null) {
            this.#pc.onicecandidate = null;
            this.#pc.onconnectionstatechange = null;
            this.#pc.ontrack = null;
            this.#pc.ondatachannel = null;
            this.#pc.close();
            this.#pc = null;
        }

        this.#offerData = null;

        if (this.#sessionUrl !== null) {
            const sessionUrl = this.#sessionUrl;
            this.#sessionUrl = null;
            this.#deleteSessionBestEffort(sessionUrl);
        }

        this.#queuedCandidates = [];

        // The reader is single-use: any error here is terminal. The caller owns all backoff/retry
        // timing and is responsible for constructing a new MediaMTXWebRTCReader to retry, instead of
        // racing with an internal retry loop.
        this.#state = 'failed';
        this.#conf.onError?.(err);
    }

    #start() {
        if (this.#state !== 'running') {
            return;
        }

        this.#requestICEServers()
            .then((iceServers) => this.#setupPeerConnection(iceServers))
            .then((offer) => this.#sendOffer(offer))
            .then((answer) => this.#setAnswer(answer))
            .catch((err: unknown) => {
                this.#handleError(String(err));
            });
    }

    #authHeader(): Record<string, string> {
        if (this.#conf.user !== undefined && this.#conf.user !== '') {
            const credentials = btoa(`${this.#conf.user}:${this.#conf.pass}`);
            return { Authorization: `Basic ${credentials}` };
        }

        if (this.#conf.token !== undefined && this.#conf.token !== '') {
            return { Authorization: `Bearer ${this.#conf.token}` };
        }

        return {};
    }

    #requestICEServers() {
        return this.#fetchWithTimeout(
            this.#conf.url,
            {
                method: 'OPTIONS',
                headers: this.#authHeader(),
            },
            MediaMTXWebRTCReader.#FETCH_TIMEOUT,
        ).then((res) => MediaMTXWebRTCReader.#linkToIceServers(res.headers.get('Link')));
    }

    #setupPeerConnection(iceServers: RTCIceServer[]) {
        if (this.#state !== 'running') {
            throw new Error('closed');
        }

        this.#pc = new RTCPeerConnection({ iceServers });

        const direction = 'recvonly';

        const videoTransceiver = this.#pc.addTransceiver('video', { direction });

        try {
            videoTransceiver.receiver.jitterBufferTarget = MediaMTXWebRTCReader.#JITTER_BUFFER_TARGET_MS;
        } catch (err) {
            console.warn('[WebRTC] Could not set jitter buffer target:', err);
        }
        this.#pc.createDataChannel('');

        this.#pc.onicecandidate = (evt) => this.#onLocalCandidate(evt);
        this.#pc.onconnectionstatechange = () => this.#onConnectionState();
        this.#pc.ontrack = (evt) => this.#onTrack(evt);
        this.#pc.ondatachannel = (evt) => this.#onDataChannel(evt);

        return this.#pc.createOffer().then((offer) => {
            if (!offer.sdp) {
                throw new Error('missing offer SDP');
            }

            this.#offerData = MediaMTXWebRTCReader.#parseOffer(offer.sdp);

            return this.#pc!.setLocalDescription(offer).then(() => offer.sdp as string);
        });
    }

    #sendOffer(offer: string) {
        if (this.#state !== 'running') {
            throw new Error('closed');
        }

        return this.#fetchWithTimeout(
            this.#conf.url,
            {
                method: 'POST',
                headers: {
                    ...this.#authHeader(),
                    'Content-Type': 'application/sdp',
                },
                body: offer,
            },
            MediaMTXWebRTCReader.#FETCH_TIMEOUT,
        ).then((res) => {
            switch (res.status) {
                case 201:
                    break;
                case 404:
                    throw new Error('stream not found');
                case 400:
                    return res.json().then((error: { error?: string }) => {
                        throw new Error(error.error ?? 'bad request');
                    });
                default:
                    throw new Error(`bad status code ${res.status}`);
            }

            const location = res.headers.get('location');

            if (!location) {
                throw new Error('missing session location');
            }

            this.#sessionUrl = new URL(location, this.#conf.url).toString();

            return res.text();
        });
    }

    #setAnswer(answer: string) {
        if (this.#state !== 'running' || this.#pc === null) {
            throw new Error('closed');
        }

        return this.#pc
            .setRemoteDescription(
                new RTCSessionDescription({
                    type: 'answer',
                    sdp: answer,
                }),
            )
            .then(() => {
                if (this.#state !== 'running') {
                    return;
                }

                if (this.#queuedCandidates.length !== 0) {
                    this.#sendLocalCandidates(this.#queuedCandidates);
                    this.#queuedCandidates = [];
                }

                this.#startStatsMonitor();
            });
    }

    #onLocalCandidate(evt: RTCPeerConnectionIceEvent) {
        if (this.#state !== 'running' || evt.candidate === null) {
            return;
        }

        if (this.#sessionUrl === null) {
            this.#queuedCandidates.push(evt.candidate);
        } else {
            this.#sendLocalCandidates([evt.candidate]);
        }
    }

    #sendLocalCandidates(candidates: RTCIceCandidate[]) {
        if (this.#sessionUrl === null || this.#offerData === null) {
            return;
        }

        const sessionUrl = this.#sessionUrl;

        this.#fetchWithTimeout(
            sessionUrl,
            {
                method: 'PATCH',
                headers: {
                    'Content-Type': 'application/trickle-ice-sdpfrag',
                    'If-Match': '*',
                },
                body: MediaMTXWebRTCReader.#generateSdpFragment(this.#offerData, candidates),
            },
            MediaMTXWebRTCReader.#FETCH_TIMEOUT,
        )
            .then((res) => {
                switch (res.status) {
                    case 204:
                        break;
                    case 404:
                        throw new Error('stream not found');
                    default:
                        throw new Error(`bad status code ${res.status}`);
                }
            })
            .catch((err: unknown) => {
                if (this.#sessionUrl !== sessionUrl || this.#state === 'closed') {
                    return;
                }

                this.#handleError(String(err));
            });
    }

    #onConnectionState() {
        if (this.#state !== 'running' || this.#pc === null) {
            return;
        }

        if (this.#pc.connectionState === 'connected' || this.#pc.connectionState === 'connecting') {
            if (this.#disconnectTimeout !== null) {
                clearTimeout(this.#disconnectTimeout);
                this.#disconnectTimeout = null;
            }

            return;
        }

        if (this.#pc.connectionState === 'disconnected') {
            if (this.#disconnectTimeout === null) {
                this.#disconnectTimeout = window.setTimeout(() => {
                    this.#disconnectTimeout = null;

                    if (this.#state === 'running' && this.#pc?.connectionState === 'disconnected') {
                        this.#handleError('peer connection disconnected');
                    }
                }, MediaMTXWebRTCReader.#DISCONNECTED_RETRY_PAUSE);
            }

            return;
        }

        if (this.#pc.connectionState === 'failed' || this.#pc.connectionState === 'closed') {
            this.#handleError('peer connection closed');
        }
    }

    #onTrack(evt: RTCTrackEvent) {
        this.#conf.onTrack?.(evt);
    }

    #onDataChannel(evt: RTCDataChannelEvent) {
        this.#conf.onDataChannel?.(evt);
    }
}

export { MediaMTXWebRTCReader };
