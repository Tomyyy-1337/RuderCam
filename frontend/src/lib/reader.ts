type OnError = (err: string) => void
type OnTrack = (evt: RTCTrackEvent) => void
type OnDataChannel = (evt: RTCDataChannelEvent) => void

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

type ReaderState = 'getting_codecs' | 'running' | 'restarting' | 'failed' | 'closed'
type CandidateBuckets = Record<number, RTCIceCandidate[]>

class MediaMTXWebRTCReader {
  static #RETRY_PAUSE = 2000;

  #conf: ReaderConfig;
  #state: ReaderState = 'getting_codecs';
  #restartTimeout: number | null = null;
  #pc: RTCPeerConnection | null = null;
  #offerData: OfferData | null = null;
  #sessionUrl: string | null = null;
  #queuedCandidates: RTCIceCandidate[] = [];
  #nonAdvertisedCodecs: string[] = [];

  constructor(conf: ReaderConfig) {
    this.#conf = conf;
    this.#getNonAdvertisedCodecs();
  }

  close() {
    this.#state = 'closed';

    if (this.#pc !== null) {
      this.#pc.close();
    }

    if (this.#restartTimeout !== null) {
      clearTimeout(this.#restartTimeout);
    }
  }

  static #supportsNonAdvertisedCodec(codec: string, fmtp?: string) {
    return new Promise<boolean>((resolve) => {
      const pc = new RTCPeerConnection({ iceServers: [] });
      const mediaType = 'audio';
      let payloadType = '';

      pc.addTransceiver(mediaType, { direction: 'recvonly' });
      pc.createOffer()
        .then((offer) => {
          if (!offer.sdp) {
            throw new Error('SDP not present');
          }
          if (offer.sdp.includes(` ${codec}`)) {
            throw new Error('already present');
          }

          const sections = offer.sdp.split(`m=${mediaType}`);

          const payloadTypes = sections
            .slice(1)
            .map((s) => s.split('\r\n')[0].split(' ').slice(3))
            .reduce<string[]>((prev, cur) => [...prev, ...cur], []);
          payloadType = this.#reservePayloadType(payloadTypes);

          const lines = sections[1].split('\r\n');
          lines[0] += ` ${payloadType}`;
          lines.splice(lines.length - 1, 0, `a=rtpmap:${payloadType} ${codec}`);
          if (fmtp !== undefined) {
            lines.splice(lines.length - 1, 0, `a=fmtp:${payloadType} ${fmtp}`);
          }
          sections[1] = lines.join('\r\n');
          offer.sdp = sections.join(`m=${mediaType}`);
          return pc.setLocalDescription(offer);
        })
        .then(() =>
          pc.setRemoteDescription(
            new RTCSessionDescription({
              type: 'answer',
              sdp:
                'v=0\r\n' +
                'o=- 6539324223450680508 0 IN IP4 0.0.0.0\r\n' +
                's=-\r\n' +
                't=0 0\r\n' +
                'a=fingerprint:sha-256 0D:9F:78:15:42:B5:4B:E6:E2:94:3E:5B:37:78:E1:4B:54:59:A3:36:3A:E5:05:EB:27:EE:8F:D2:2D:41:29:25\r\n' +
                `m=${mediaType} 9 UDP/TLS/RTP/SAVPF ${payloadType}\r\n` +
                'c=IN IP4 0.0.0.0\r\n' +
                'a=ice-pwd:7c3bf4770007e7432ee4ea4d697db675\r\n' +
                'a=ice-ufrag:29e036dc\r\n' +
                'a=sendonly\r\n' +
                'a=rtcp-mux\r\n' +
                `a=rtpmap:${payloadType} ${codec}\r\n` +
                (fmtp !== undefined ? `a=fmtp:${payloadType} ${fmtp}\r\n` : ''),
            }),
          ),
        )
        .then(() => {
          resolve(true);
        })
        .catch(() => {
          resolve(false);
        })
        .finally(() => {
          pc.close();
        });
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

  static #reservePayloadType(payloadTypes: string[]) {
    for (let i = 30; i <= 127; i++) {
      if ((i <= 63 || i >= 96) && !payloadTypes.includes(i.toString())) {
        const payload = i.toString();
        payloadTypes.push(payload);
        return payload;
      }
    }
    throw Error('unable to find a free payload type');
  }

  static #enableStereoPcmau(payloadTypes: string[], section: string) {
    const lines = section.split('\r\n');

    let payloadType = this.#reservePayloadType(payloadTypes);
    lines[0] += ` ${payloadType}`;
    lines.splice(lines.length - 1, 0, `a=rtpmap:${payloadType} PCMU/8000/2`);
    lines.splice(lines.length - 1, 0, `a=rtcp-fb:${payloadType} transport-cc`);

    payloadType = this.#reservePayloadType(payloadTypes);
    lines[0] += ` ${payloadType}`;
    lines.splice(lines.length - 1, 0, `a=rtpmap:${payloadType} PCMA/8000/2`);
    lines.splice(lines.length - 1, 0, `a=rtcp-fb:${payloadType} transport-cc`);

    return lines.join('\r\n');
  }

  static #enableMultichannelOpus(payloadTypes: string[], section: string) {
    const lines = section.split('\r\n');

    let payloadType = this.#reservePayloadType(payloadTypes);
    lines[0] += ` ${payloadType}`;
    lines.splice(lines.length - 1, 0, `a=rtpmap:${payloadType} multiopus/48000/3`);
    lines.splice(lines.length - 1, 0, `a=fmtp:${payloadType} channel_mapping=0,2,1;num_streams=2;coupled_streams=1`);
    lines.splice(lines.length - 1, 0, `a=rtcp-fb:${payloadType} transport-cc`);

    payloadType = this.#reservePayloadType(payloadTypes);
    lines[0] += ` ${payloadType}`;
    lines.splice(lines.length - 1, 0, `a=rtpmap:${payloadType} multiopus/48000/4`);
    lines.splice(lines.length - 1, 0, `a=fmtp:${payloadType} channel_mapping=0,1,2,3;num_streams=2;coupled_streams=2`);
    lines.splice(lines.length - 1, 0, `a=rtcp-fb:${payloadType} transport-cc`);

    payloadType = this.#reservePayloadType(payloadTypes);
    lines[0] += ` ${payloadType}`;
    lines.splice(lines.length - 1, 0, `a=rtpmap:${payloadType} multiopus/48000/5`);
    lines.splice(lines.length - 1, 0, `a=fmtp:${payloadType} channel_mapping=0,4,1,2,3;num_streams=3;coupled_streams=2`);
    lines.splice(lines.length - 1, 0, `a=rtcp-fb:${payloadType} transport-cc`);

    payloadType = this.#reservePayloadType(payloadTypes);
    lines[0] += ` ${payloadType}`;
    lines.splice(lines.length - 1, 0, `a=rtpmap:${payloadType} multiopus/48000/6`);
    lines.splice(lines.length - 1, 0, `a=fmtp:${payloadType} channel_mapping=0,4,1,2,3,5;num_streams=4;coupled_streams=2`);
    lines.splice(lines.length - 1, 0, `a=rtcp-fb:${payloadType} transport-cc`);

    payloadType = this.#reservePayloadType(payloadTypes);
    lines[0] += ` ${payloadType}`;
    lines.splice(lines.length - 1, 0, `a=rtpmap:${payloadType} multiopus/48000/7`);
    lines.splice(lines.length - 1, 0, `a=fmtp:${payloadType} channel_mapping=0,4,1,2,3,5,6;num_streams=4;coupled_streams=4`);
    lines.splice(lines.length - 1, 0, `a=rtcp-fb:${payloadType} transport-cc`);

    payloadType = this.#reservePayloadType(payloadTypes);
    lines[0] += ` ${payloadType}`;
    lines.splice(lines.length - 1, 0, `a=rtpmap:${payloadType} multiopus/48000/8`);
    lines.splice(lines.length - 1, 0, `a=fmtp:${payloadType} channel_mapping=0,6,1,4,5,2,3,7;num_streams=5;coupled_streams=4`);
    lines.splice(lines.length - 1, 0, `a=rtcp-fb:${payloadType} transport-cc`);

    return lines.join('\r\n');
  }

  static #enableL16(payloadTypes: string[], section: string) {
    const lines = section.split('\r\n');

    let payloadType = this.#reservePayloadType(payloadTypes);
    lines[0] += ` ${payloadType}`;
    lines.splice(lines.length - 1, 0, `a=rtpmap:${payloadType} L16/8000/2`);
    lines.splice(lines.length - 1, 0, `a=rtcp-fb:${payloadType} transport-cc`);

    payloadType = this.#reservePayloadType(payloadTypes);
    lines[0] += ` ${payloadType}`;
    lines.splice(lines.length - 1, 0, `a=rtpmap:${payloadType} L16/16000/2`);
    lines.splice(lines.length - 1, 0, `a=rtcp-fb:${payloadType} transport-cc`);

    payloadType = this.#reservePayloadType(payloadTypes);
    lines[0] += ` ${payloadType}`;
    lines.splice(lines.length - 1, 0, `a=rtpmap:${payloadType} L16/48000/2`);
    lines.splice(lines.length - 1, 0, `a=rtcp-fb:${payloadType} transport-cc`);

    return lines.join('\r\n');
  }

  static #enableStereoOpus(section: string) {
    let opusPayloadFormat = '';
    const lines = section.split('\r\n');

    for (let i = 0; i < lines.length; i++) {
      if (lines[i].startsWith('a=rtpmap:') && lines[i].toLowerCase().includes('opus/')) {
        opusPayloadFormat = lines[i].slice('a=rtpmap:'.length).split(' ')[0];
        break;
      }
    }

    if (opusPayloadFormat === '') {
      return section;
    }

    for (let i = 0; i < lines.length; i++) {
      if (lines[i].startsWith(`a=fmtp:${opusPayloadFormat} `)) {
        if (!lines[i].includes('stereo')) {
          lines[i] += ';stereo=1';
        }
        if (!lines[i].includes('sprop-stereo')) {
          lines[i] += ';sprop-stereo=1';
        }
      }
    }

    return lines.join('\r\n');
  }

  static #editOffer(sdp: string, nonAdvertisedCodecs: string[]) {
    const sections = sdp.split('m=');

    const payloadTypes = sections
      .slice(1)
      .map((s) => s.split('\r\n')[0].split(' ').slice(3))
      .reduce<string[]>((prev, cur) => [...prev, ...cur], []);

    for (let i = 1; i < sections.length; i++) {
      if (sections[i].startsWith('audio')) {
        sections[i] = this.#enableStereoOpus(sections[i]);

        if (nonAdvertisedCodecs.includes('pcma/8000/2')) {
          sections[i] = this.#enableStereoPcmau(payloadTypes, sections[i]);
        }
        if (nonAdvertisedCodecs.includes('multiopus/48000/6')) {
          sections[i] = this.#enableMultichannelOpus(payloadTypes, sections[i]);
        }
        if (nonAdvertisedCodecs.includes('L16/48000/2')) {
          sections[i] = this.#enableL16(payloadTypes, sections[i]);
        }

        break;
      }
    }

    return sections.join('m=');
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

  #handleError(err: string) {
    if (this.#state === 'running') {
      if (this.#pc !== null) {
        this.#pc.close();
        this.#pc = null;
      }

      this.#offerData = null;

      if (this.#sessionUrl !== null) {
        fetch(this.#sessionUrl, {
          method: 'DELETE',
        });
        this.#sessionUrl = null;
      }

      this.#queuedCandidates = [];
      this.#state = 'restarting';

      this.#restartTimeout = window.setTimeout(
        () => this.#restart(),
        MediaMTXWebRTCReader.#RETRY_PAUSE,
      );

      this.#conf.onError?.(`${err}, retrying in some seconds`);
    } else if (this.#state === 'getting_codecs') {
      this.#state = 'failed';
      this.#conf.onError?.(err);
    }
  }

  #restart() {
    this.#restartTimeout = null;
    this.#state = 'running';
    this.#start();
  }

  #getNonAdvertisedCodecs() {
    Promise.all(
      [
        ['pcma/8000/2'],
        ['multiopus/48000/6', 'channel_mapping=0,4,1,2,3,5;num_streams=4;coupled_streams=2'],
        ['L16/48000/2'],
      ].map((codec) =>
        MediaMTXWebRTCReader.#supportsNonAdvertisedCodec(codec[0], codec[1]).then(
          (result) => (result ? codec[0] : false),
        ),
      ),
    )
      .then((codecs) => codecs.filter((value): value is string => value !== false))
      .then((codecs) => {
        if (this.#state !== 'getting_codecs') {
          throw new Error('closed');
        }

        this.#nonAdvertisedCodecs = codecs;
        this.#state = 'running';
        this.#start();
      })
      .catch((err: unknown) => {
        this.#handleError(String(err));
      });
  }

  #start() {
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
    return fetch(this.#conf.url, {
      method: 'OPTIONS',
      headers: this.#authHeader(),
    }).then((res) => MediaMTXWebRTCReader.#linkToIceServers(res.headers.get('Link')));
  }

  #setupPeerConnection(iceServers: RTCIceServer[]) {
    if (this.#state !== 'running') {
      throw new Error('closed');
    }

    this.#pc = new RTCPeerConnection({ iceServers });

    const direction = 'recvonly';
    this.#pc.addTransceiver('video', { direction });
    this.#pc.addTransceiver('audio', { direction });
    this.#pc.createDataChannel('');

    this.#pc.onicecandidate = (evt) => this.#onLocalCandidate(evt);
    this.#pc.onconnectionstatechange = () => this.#onConnectionState();
    this.#pc.ontrack = (evt) => this.#onTrack(evt);
    this.#pc.ondatachannel = (evt) => this.#onDataChannel(evt);

    return this.#pc.createOffer().then((offer) => {
      if (!offer.sdp) {
        throw new Error('missing offer SDP');
      }

      offer.sdp = MediaMTXWebRTCReader.#editOffer(offer.sdp, this.#nonAdvertisedCodecs);
      this.#offerData = MediaMTXWebRTCReader.#parseOffer(offer.sdp);

      return this.#pc!.setLocalDescription(offer).then(() => offer.sdp as string);
    });
  }

  #sendOffer(offer: string) {
    if (this.#state !== 'running') {
      throw new Error('closed');
    }

    return fetch(this.#conf.url, {
      method: 'POST',
      headers: {
        ...this.#authHeader(),
        'Content-Type': 'application/sdp',
      },
      body: offer,
    }).then((res) => {
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

    fetch(this.#sessionUrl, {
      method: 'PATCH',
      headers: {
        'Content-Type': 'application/trickle-ice-sdpfrag',
        'If-Match': '*',
      },
      body: MediaMTXWebRTCReader.#generateSdpFragment(this.#offerData, candidates),
    })
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
        this.#handleError(String(err));
      });
  }

  #onConnectionState() {
    if (this.#state !== 'running' || this.#pc === null) {
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

export { MediaMTXWebRTCReader }