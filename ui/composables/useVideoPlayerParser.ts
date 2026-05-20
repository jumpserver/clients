// @ts-expect-error library ships without useful ESM typings
import untar from "js-untar";
import { gunzipSync } from "fflate";

export type VideoPlayerItemType = "mp4" | "cast" | "gua" | "part";

export interface VideoPlayerMeta {
  account?: string
  user?: string
  asset?: string
  protocol?: string
  command_amount?: number
  date_end?: string
  date_start?: string
  duration?: string
  files?: VideoPlayerFileMeta[]
}

export interface VideoPlayerFileMeta {
  name?: string
  start?: number
  end?: number
  duration?: number
}

interface EffectiveItemMeta extends VideoPlayerMeta {
  fileStart?: number
  fileEnd?: number
  fileDuration?: number
}

export interface VideoPlayerItem {
  id: string
  name: string
  source: string
  type: VideoPlayerItemType
  meta: VideoPlayerMeta
  recordingId: string
  recordingLabel: string
  partIndex?: number
  partTotal?: number
  tempPath?: string
}

interface ParseResult {
  items: VideoPlayerItem[]
}

interface UntarEntry {
  name: string
  buffer: ArrayBuffer
}

const REGEXP = /\.(json|replay|cast|part)(\.mp4|\.json|\.gz)?$/;

function createId(prefix: string) {
  return `${prefix}-${Date.now()}-${Math.random().toString(36).slice(2)}`;
}

function stripArchiveExtension(fileName: string) {
  return fileName
    .replace(/\.tar$/i, "")
    .replace(/\.cast\.gz$/i, "")
    .replace(/\.replay\.gz$/i, "")
    .replace(/\.part\.gz$/i, "")
    .replace(/\.mp4$/i, "");
}

function safeParseJson(buffer: ArrayBuffer): VideoPlayerMeta | null {
  try {
    const text = new TextDecoder("utf-8").decode(new Uint8Array(buffer));
    return JSON.parse(text) as VideoPlayerMeta;
  } catch {
    return null;
  }
}

function formatMillisDuration(millis?: number) {
  if (!millis || millis < 0) return undefined;

  const totalSeconds = Math.floor(millis / 1000);
  const hours = Math.floor(totalSeconds / 3600);
  const minutes = Math.floor((totalSeconds % 3600) / 60);
  const seconds = totalSeconds % 60;

  return `${hours}:${`${minutes}`.padStart(2, "0")}:${`${seconds}`.padStart(2, "0")}`;
}

function formatTimestamp(millis?: number) {
  if (!millis || millis < 0) return undefined;

  const date = new Date(millis);

  if (Number.isNaN(date.getTime())) return undefined;

  const year = date.getFullYear();
  const month = `${date.getMonth() + 1}`.padStart(2, "0");
  const day = `${date.getDate()}`.padStart(2, "0");
  const hours = `${date.getHours()}`.padStart(2, "0");
  const minutes = `${date.getMinutes()}`.padStart(2, "0");
  const seconds = `${date.getSeconds()}`.padStart(2, "0");

  return `${year}/${month}/${day} ${hours}:${minutes}:${seconds}`;
}

function resolveItemMeta(meta: VideoPlayerMeta | null, entryName: string): EffectiveItemMeta {
  if (!meta) return {};

  const fileMeta = meta.files?.find(file => file.name === entryName);

  return {
    ...meta,
    date_start: formatTimestamp(fileMeta?.start) || meta.date_start,
    date_end: formatTimestamp(fileMeta?.end) || meta.date_end,
    duration: formatMillisDuration(fileMeta?.duration) || meta.duration,
    fileStart: fileMeta?.start,
    fileEnd: fileMeta?.end,
    fileDuration: fileMeta?.duration
  };
}

function toMp4Url(buffer: ArrayBuffer) {
  const blob = new Blob([new Uint8Array(buffer)], { type: "video/mp4" });
  return URL.createObjectURL(blob);
}

function toCastUrl(buffer: ArrayBuffer) {
  const blob = new Blob([buffer], { type: "application/json" });
  return URL.createObjectURL(blob);
}

function toGzipUrl(buffer: ArrayBuffer) {
  const blob = new Blob([buffer], { type: "application/gzip" });
  return URL.createObjectURL(blob);
}

function withMeta(item: Omit<VideoPlayerItem, "id" | "meta">, meta: VideoPlayerMeta | null): VideoPlayerItem {
  return {
    id: createId(item.name),
    meta: meta || {},
    ...item
  };
}

export function useVideoPlayerParser() {
  async function buildItemFromEntry(
    entry: UntarEntry,
    meta: VideoPlayerMeta | null
  ): Promise<VideoPlayerItem | null> {
    const match = entry.name.match(REGEXP);
    const kind = match?.[1];
    const effectiveMeta = resolveItemMeta(meta, entry.name);

    switch (kind) {
      case "replay": {
        const isGua = entry.name.split(".")[2] === "gz";

        if (isGua) {
          return withMeta(
            {
              name: entry.name,
              source: toGzipUrl(entry.buffer),
              type: "gua"
            },
            effectiveMeta
          );
        }

        return withMeta(
          {
            name: entry.name,
            source: toMp4Url(entry.buffer),
            type: "mp4"
          },
          effectiveMeta
        );
      }
      case "cast": {
        const output = gunzipSync(new Uint8Array(entry.buffer));
        return withMeta(
          {
            name: entry.name,
            source: toCastUrl(output.buffer.slice(output.byteOffset, output.byteOffset + output.byteLength)),
            type: "cast"
          },
          effectiveMeta
        );
      }
      case "part": {
        return withMeta(
          {
            name: entry.name,
            source: toGzipUrl(entry.buffer),
            type: "part"
          },
            effectiveMeta
          );
      }
      default:
        return null;
    }
  }

  async function parseTarFile(file: File): Promise<ParseResult> {
    const recordingId = createId(file.name);
    const recordingLabel = stripArchiveExtension(file.name);
    const extractedFiles = await untar(await file.arrayBuffer()).progress(() => {});
    let meta: VideoPlayerMeta | null = null;
    const items: VideoPlayerItem[] = [];

    for (const entry of extractedFiles as UntarEntry[]) {
      const match = entry.name.match(REGEXP);

      if (!match) continue;

      if (match[0] === ".replay.json" || match[1] === "json") {
        meta = safeParseJson(entry.buffer) || meta;
      }
    }

    for (const entry of extractedFiles as UntarEntry[]) {
      const match = entry.name.match(REGEXP);

      if (!match || match[1] === "json" || match[0] === ".replay.json") continue;

      const item = await buildItemFromEntry(entry, meta);

      if (item) {
        items.push({
          ...item,
          recordingId,
          recordingLabel
        });
      }
    }

    const hasExplicitParts = (meta?.files?.length || 0) > 1;

    if (hasExplicitParts) {
      items.forEach((item, index) => {
        if (item.type === "part") {
          item.partIndex = index + 1;
          item.partTotal = items.length;
        }
      });
    }

    return { items };
  }

  async function parseSingleFile(file: File): Promise<ParseResult> {
    const fileName = file.name;
    const recordingId = createId(fileName);
    const recordingLabel = stripArchiveExtension(fileName);
    const items: VideoPlayerItem[] = [];

    if (fileName.endsWith(".mp4")) {
      items.push(withMeta({
        name: fileName,
        source: toMp4Url(await file.arrayBuffer()),
        type: "mp4",
        recordingId,
        recordingLabel
      }, null));
      return { items };
    }

    if (fileName.endsWith(".cast.gz")) {
      const output = gunzipSync(new Uint8Array(await file.arrayBuffer()));
      items.push(withMeta({
        name: fileName,
        source: toCastUrl(output.buffer.slice(output.byteOffset, output.byteOffset + output.byteLength)),
        type: "cast",
        recordingId,
        recordingLabel
      }, null));
      return { items };
    }

    if (fileName.endsWith(".replay.gz")) {
      items.push(withMeta({
        name: fileName,
        source: URL.createObjectURL(file),
        type: "gua",
        recordingId,
        recordingLabel
      }, null));
      return { items };
    }

    if (fileName.endsWith(".part.gz")) {
      items.push(withMeta({
        name: fileName,
        source: URL.createObjectURL(file),
        type: "part",
        recordingId,
        recordingLabel
      }, null));
      return { items };
    }

    return { items };
  }

  async function parseFiles(files: File[]) {
    const items: VideoPlayerItem[] = [];

    for (const file of files) {
      const result = file.name.includes(".tar")
        ? await parseTarFile(file)
        : await parseSingleFile(file);
      items.push(...result.items);
    }

    return items;
  }

  return {
    parseFiles
  };
}
