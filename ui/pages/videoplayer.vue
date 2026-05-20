<script setup lang="ts">
import type { VideoPlayerItem } from "~/composables/useVideoPlayerParser";

definePageMeta({
  layout: false
});

const toast = useToast();
const fileInputRef = ref<HTMLInputElement | null>(null);
const isImporting = ref(false);
const items = ref<VideoPlayerItem[]>([]);
const activeId = ref<string | null>(null);
const importMessage = ref("");
const VIDEO_PLAYER_MIN_WIDTH = 1320;
const VIDEO_PLAYER_MIN_HEIGHT = 860;
const VIDEO_PLAYER_TARGET_WIDTH = 1480;
const VIDEO_PLAYER_TARGET_HEIGHT = 920;

const { parseFiles } = useVideoPlayerParser();
const { deleteTempFile } = useVideoPlayerTauri();

const currentItem = computed(() => items.value.find((item) => item.id === activeId.value) || null);
const currentMeta = computed(() => currentItem.value?.meta || items.value[0]?.meta || {});
const currentIndex = computed(() => items.value.findIndex((item) => item.id === activeId.value) + 1);

const playerComponent = computed(() => {
  switch (currentItem.value?.type) {
    case "cast":
      return resolveComponent("VideoPlayerPlayersAsciinemaPlayer");
    case "gua":
    case "part":
      return resolveComponent("VideoPlayerPlayersGuaPlayer");
    case "mp4":
      return resolveComponent("VideoPlayerPlayersMp4Player");
    default:
      return null;
  }
});

const infoCards = computed(() => {
  const meta = currentMeta.value;

  return [
    { label: "账号", value: meta.account || "-" },
    { label: "用户", value: meta.user || "-" },
    { label: "资产", value: meta.asset || "-" },
    { label: "协议", value: meta.protocol || "-" },
    { label: "开始时间", value: meta.date_start || "-" },
    { label: "时长", value: meta.duration || formatDuration(meta.date_start, meta.date_end) },
    { label: "片段", value: currentIndex.value > 0 ? `${currentIndex.value} / ${items.value.length}` : "-" }
  ];
});

function formatDuration(startAt?: string, endAt?: string) {
  if (!startAt || !endAt) return "-";

  const start = new Date(startAt);
  const end = new Date(endAt);
  const diff = end.getTime() - start.getTime();

  if (!Number.isFinite(diff) || diff < 0) return "-";

  const totalSeconds = Math.floor(diff / 1000);
  const hours = Math.floor(totalSeconds / 3600);
  const minutes = Math.floor((totalSeconds % 3600) / 60);
  const seconds = totalSeconds % 60;

  return [hours, minutes, seconds].map((value) => `${value}`.padStart(2, "0")).join(":");
}

async function cleanupItem(item: VideoPlayerItem) {
  if (item.source.startsWith("blob:")) {
    URL.revokeObjectURL(item.source);
  }

  if (item.tempPath) {
    try {
      await deleteTempFile(item.tempPath);
    } catch {
      // ignore cleanup errors
    }
  }
}

function selectItem(item: VideoPlayerItem) {
  activeId.value = item.id;
}

async function removeItem(item: VideoPlayerItem) {
  await cleanupItem(item);
  items.value = items.value.filter((entry) => entry.id !== item.id);

  if (activeId.value === item.id) {
    activeId.value = items.value[0]?.id || null;
  }
}

async function importFiles(files: File[]) {
  if (files.length === 0) return;

  isImporting.value = true;
  importMessage.value = `正在导入 ${files.length} 个文件…`;

  try {
    const parsed = await parseFiles(files);

    if (parsed.length === 0) {
      importMessage.value = "";
      toast.add({
        title: "未识别到可播放文件",
        description: "请导入 mp4、cast.gz、replay.gz、part.gz 或包含这些文件的 tar 包。",
        color: "warning"
      });
      return;
    }

    const existingNames = new Set(items.value.map((item) => item.name));
    const incoming = parsed.filter((item) => !existingNames.has(item.name));
    const duplicates = parsed.length - incoming.length;

    items.value.push(...incoming);

    if (!activeId.value && incoming[0]) {
      activeId.value = incoming[0].id;
    }

    importMessage.value = "";

    if (duplicates > 0) {
      toast.add({
        title: "部分文件已跳过",
        description: `有 ${duplicates} 个同名条目已存在，未重复导入。`,
        color: "neutral"
      });
    }
  } catch (error: any) {
    importMessage.value = "";
    toast.add({
      title: "导入失败",
      description: error?.message || String(error),
      color: "error"
    });
  } finally {
    isImporting.value = false;
  }
}

function openFilePicker() {
  fileInputRef.value?.click();
}

function handleInputChange(event: Event) {
  const target = event.target as HTMLInputElement;
  const files = Array.from(target.files || []);
  target.value = "";
  void importFiles(files);
}

async function optimizeWindowForVideoPlayer() {
  try {
    const currentWindow = useTauriWindowGetCurrentWindow();
    const minSize = new useTauriWindowLogicalSize(VIDEO_PLAYER_MIN_WIDTH, VIDEO_PLAYER_MIN_HEIGHT);

    await currentWindow.setMinSize(minSize);

    const currentSize = await currentWindow.innerSize();
    const scaleFactor = await currentWindow.scaleFactor();
    const currentLogicalWidth = currentSize.width / scaleFactor;
    const currentLogicalHeight = currentSize.height / scaleFactor;
    const nextWidth = Math.max(currentLogicalWidth, VIDEO_PLAYER_TARGET_WIDTH);
    const nextHeight = Math.max(currentLogicalHeight, VIDEO_PLAYER_TARGET_HEIGHT);

    if (nextWidth !== currentLogicalWidth || nextHeight !== currentLogicalHeight) {
      await currentWindow.setSize(
        new useTauriWindowLogicalSize(nextWidth, nextHeight)
      );
    }
  } catch (error) {
    console.debug("optimize video player window failed", error);
  }
}

onMounted(async () => {
  document.title = "JumpServer Video Player";

  try {
    const currentWindow = useTauriWindowGetCurrentWindow();
    await currentWindow.setTitle("JumpServer Video Player");
    await optimizeWindowForVideoPlayer();
  } catch {
    // ignore when running in browser
  }
});

onBeforeUnmount(async () => {
  await Promise.all(items.value.map((item) => cleanupItem(item)));

  try {
    await useTauriWindowGetCurrentWindow().setMinSize(null);
  } catch {
    // ignore when running in browser
  }
});
</script>

<template>
  <div class="videoplayer-page h-screen overflow-hidden">
    <input
      id="videoplayer-file-input"
      ref="fileInputRef"
      class="sr-only"
      type="file"
      multiple
      accept=".mp4,.gz,.tar,.json"
      @change="handleInputChange"
    >

    <div class="mx-auto flex h-full w-full max-w-[1700px] flex-col px-6 py-5 lg:px-8">
      <header data-tauri-drag-region class="mb-5 flex items-center justify-between gap-4">
        <div data-tauri-drag-region>
          <p class="text-xs uppercase tracking-[0.32em] text-(--ui-text-dimmed)">
            JumpServer
          </p>
          <h1 class="mt-2 text-3xl font-semibold tracking-tight text-(--ui-text-highlighted)">
            Video Player
          </h1>
        </div>

        <UButton color="neutral" variant="ghost" to="/linux">
          返回主界面
        </UButton>
      </header>

      <p v-if="importMessage && items.length === 0" class="mb-4 text-sm text-(--ui-text-muted)">
        {{ importMessage }}
      </p>

      <div class="grid min-h-0 flex-1 grid-cols-[minmax(0,1.9fr)_minmax(280px,0.78fr)] gap-5">
        <section class="grid min-h-0 min-w-0 grid-rows-[minmax(0,1.7fr)_minmax(180px,0.9fr)] gap-4">
          <div class="flex min-h-0 overflow-hidden rounded-xl border-0 bg-black shadow-xl shadow-black/10 backdrop-blur">
            <div class="h-full min-w-0 flex-1 overflow-hidden bg-black">
              <component
                :is="playerComponent"
                v-if="playerComponent && currentItem"
                :source="currentItem.source"
              />
              <label
                v-else
                for="videoplayer-file-input"
                class="group flex h-full w-full cursor-pointer flex-col items-center justify-center gap-4 px-6 py-6 text-center"
              >
                <div class="flex h-16 w-16 items-center justify-center rounded-xl bg-white/8 text-3xl text-(--ui-primary)">
                  <UIcon name="line-md:upload-loop" />
                </div>
                <div class="max-w-xl">
                  <p class="text-xl font-semibold tracking-tight text-(--ui-text-highlighted)">
                    导入录像文件
                  </p>
                  <p class="mt-2 text-sm leading-6 text-(--ui-text-muted)">
                    将录像拖入播放区，或点击这里选择 `.mp4`、`.cast.gz`、`.replay.gz`、`.part.gz`、`.tar` 文件。
                  </p>
                </div>
                <div class="rounded-full border-0 bg-(--ui-bg-muted) px-4 py-2 text-sm text-(--ui-text-toned) transition group-hover:bg-(--ui-bg-accented)">
                  选择文件
                </div>
              </label>
            </div>
          </div>

          <div class="min-h-0 overflow-hidden rounded-xl border-0 bg-(--ui-bg-elevated)/60 px-4 py-3 shadow-lg shadow-black/5 backdrop-blur">
            <div class="mb-3 flex items-center gap-3">
              <p class="text-[11px] uppercase tracking-[0.2em] text-(--ui-text-dimmed)">
                录像信息
              </p>
              <div class="h-px flex-1 bg-(--ui-border)/60" />
            </div>

            <div class="grid grid-cols-4 gap-2 xl:grid-cols-4 lg:grid-cols-3 md:grid-cols-2">
              <div
                v-for="card in infoCards"
                :key="card.label"
                class="rounded-lg border-0 bg-(--ui-bg-muted)/90 px-3 py-2.5"
              >
                <p class="text-[10px] uppercase tracking-[0.14em] text-(--ui-text-dimmed)">
                  {{ card.label }}
                </p>
                <p class="mt-1 truncate text-[13px] font-medium text-(--ui-text-highlighted)">
                  {{ card.value }}
                </p>
              </div>
            </div>
          </div>
        </section>

        <aside class="min-h-0 min-w-0 overflow-hidden">
          <VideoPlayerPlaylist
            v-if="items.length > 0"
            :active-id="activeId"
            :items="items"
            :meta="currentMeta"
            @play="selectItem"
            @remove="removeItem"
          />
          <div
            v-else
            class="flex h-full min-h-0 flex-col rounded-2xl border-0 bg-(--ui-bg-elevated)/70 p-5 shadow-xl shadow-black/10 backdrop-blur"
          >
            <div class="mb-4">
              <p class="text-[11px] uppercase tracking-[0.2em] text-(--ui-text-dimmed)">
                播放列表
              </p>
              <h3 class="mt-1 text-sm font-semibold tracking-wide text-(--ui-text-highlighted)">
                播放列表
              </h3>
            </div>

            <div class="flex min-h-0 flex-1 items-center justify-center rounded-xl border border-dashed border-(--ui-border) bg-(--ui-bg-muted) p-3">
              <div class="flex max-w-[240px] flex-col items-center text-center">
                <div class="flex h-12 w-12 items-center justify-center rounded-xl bg-(--ui-bg-accented) text-2xl text-(--ui-text-dimmed)">
                  <UIcon name="line-md:list-3" />
                </div>
                <p class="mt-4 text-sm font-medium text-(--ui-text-highlighted)">
                  暂无播放片段
                </p>
                <p class="mt-2 text-xs leading-5 text-(--ui-text-muted)">
                  导入录像后，这里会显示可切换的片段列表。
                </p>
              </div>
            </div>
          </div>
        </aside>
      </div>
    </div>
  </div>
</template>

<style scoped>
.videoplayer-page {
  color: var(--ui-text);
  background:
    radial-gradient(circle at top left, color-mix(in srgb, var(--ui-color-primary-500) 14%, transparent) 0%, transparent 30%),
    radial-gradient(circle at right, color-mix(in srgb, var(--ui-bg-elevated) 65%, transparent) 0%, transparent 26%),
    linear-gradient(
      180deg,
      color-mix(in srgb, var(--ui-bg) 90%, var(--ui-bg-elevated) 10%) 0%,
      color-mix(in srgb, var(--ui-bg) 98%, black 2%) 100%
    );
}
</style>
