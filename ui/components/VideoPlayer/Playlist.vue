<script setup lang="ts">
import type { VideoPlayerItem, VideoPlayerMeta } from "~/composables/useVideoPlayerParser";

const props = defineProps<{
  activeId: string | null
  items: VideoPlayerItem[]
  meta?: VideoPlayerMeta
}>();

const emit = defineEmits<{
  play: [VideoPlayerItem]
  remove: [VideoPlayerItem]
}>();

function formatLocalStartTime(value?: string) {
  if (!value) return "-";

  const normalized = value
    .replace(/\//g, "-")
    .replace(" ", "T")
    .replace(/ ([+-]\d{2})(\d{2})$/, "$1:$2");

  const date = new Date(normalized);

  if (Number.isNaN(date.getTime())) {
    return value.replace(/\s+[+-]\d{4}$/, "");
  }

  return new Intl.DateTimeFormat("zh-CN", {
    year: "numeric",
    month: "2-digit",
    day: "2-digit",
    hour: "2-digit",
    minute: "2-digit",
    second: "2-digit",
    hour12: false
  }).format(date);
}

function formatDuration(duration?: string, startAt?: string, endAt?: string) {
  if (duration) return duration;
  if (!startAt || !endAt) return "-";

  const normalizedStart = startAt
    .replace(/\//g, "-")
    .replace(" ", "T")
    .replace(/ ([+-]\d{2})(\d{2})$/, "$1:$2");
  const normalizedEnd = endAt
    .replace(/\//g, "-")
    .replace(" ", "T")
    .replace(/ ([+-]\d{2})(\d{2})$/, "$1:$2");

  const start = new Date(normalizedStart);
  const end = new Date(normalizedEnd);
  const diff = end.getTime() - start.getTime();

  if (!Number.isFinite(diff) || diff < 0) return "-";

  const totalSeconds = Math.floor(diff / 1000);
  const hours = Math.floor(totalSeconds / 3600);
  const minutes = Math.floor((totalSeconds % 3600) / 60);
  const seconds = totalSeconds % 60;

  return [hours, minutes, seconds].map((value) => `${value}`.padStart(2, "0")).join(":");
}

</script>

<template>
  <div class="flex h-full min-h-0 flex-col rounded-2xl border-0 bg-(--ui-bg-elevated)/70 p-5 shadow-xl shadow-black/10 backdrop-blur">
    <div class="mb-3 flex items-center justify-between">
      <div class="min-w-0">
        <h3 class="truncate text-sm font-semibold tracking-wide text-(--ui-text-highlighted)">
          播放列表
        </h3>
      </div>
      <div class="flex items-center gap-2">
        <span class="rounded-full bg-(--ui-primary)/15 px-2.5 py-1 text-xs font-medium text-(--ui-primary)">
          {{ items.length }}
        </span>
        <label
          for="videoplayer-file-input"
          class="cursor-pointer rounded-full bg-(--ui-bg-muted) px-3 py-1 text-xs font-medium text-(--ui-text-toned) transition hover:bg-(--ui-bg-accented)"
        >
          添加录像
        </label>
      </div>
    </div>

    <div class="flex h-[calc(100%-3.2rem)] flex-col gap-2 overflow-y-auto pr-1">
      <button
        v-for="(item, index) in items"
        :key="item.id"
        class="group flex items-start justify-between rounded-2xl border px-3 py-3 text-left transition-colors duration-150"
        :class="item.id === activeId
          ? 'border-(--ui-primary) bg-(--ui-primary)/10'
          : 'border-(--ui-border) bg-(--ui-bg-muted) hover:border-(--ui-primary)/50 hover:bg-(--ui-bg-accented)'"
        @click="emit('play', item)"
      >
        <div class="min-w-0">
          <div class="flex items-center gap-2">
            <p
              v-if="item.partTotal && item.partTotal > 1"
              class="text-[11px] uppercase tracking-[0.18em] text-(--ui-text-dimmed)"
            >
              Part {{ item.partIndex }} / {{ item.partTotal }}
            </p>
            <span class="rounded-full bg-(--ui-bg-muted) px-2 py-0.5 text-[10px] uppercase tracking-[0.14em] text-(--ui-text-toned)">
              {{ item.type }}
            </span>
          </div>
          <p class="mt-1 truncate text-sm font-medium text-(--ui-text-highlighted)">
            {{ item.meta?.asset || item.recordingLabel || item.name }}
          </p>
          <p class="mt-1 truncate text-[11px] text-(--ui-text-muted)">
            开始时间 {{ formatLocalStartTime(item.meta?.date_start) }}
          </p>
          <p class="mt-1 text-[11px] text-(--ui-text-muted)">
            总时长 {{ formatDuration(item.meta?.duration, item.meta?.date_start, item.meta?.date_end) }}
          </p>
        </div>
        <UButton
          color="neutral"
          variant="ghost"
          icon="line-md:close-small"
          class="opacity-70 group-hover:opacity-100"
          @click.stop="emit('remove', item)"
        />
      </button>
    </div>
  </div>
</template>
