<script setup lang="ts">
import type { VideoPlayerItem } from "~/composables/useVideoPlayerParser";

definePageMeta({
  layout: false
});

const toast = useToast();
const { t } = useI18n();
const isImporting = ref(false);
const items = ref<VideoPlayerItem[]>([]);
const activeId = ref<string | null>(null);
const importMessage = ref("");
const isDraggingFiles = ref(false);
let unlistenFileDrop: (() => void) | null = null;
const VIDEO_PLAYER_MIN_WIDTH = 1320;
const VIDEO_PLAYER_MIN_HEIGHT = 860;
const VIDEO_PLAYER_TARGET_WIDTH = 1480;
const VIDEO_PLAYER_TARGET_HEIGHT = 920;

const { parseFiles } = useVideoPlayerParser();
const { deleteTempFile, readDroppedFiles } = useVideoPlayerTauri();
const { userTheme, manualSetTheme } = useThemeAdapter();

const currentItem = computed(() => items.value.find((item) => item.id === activeId.value) || null);
const isDarkMode = computed(() => userTheme.value === "dark");

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
  console.info("[VideoPlayer:DnD] page importFiles called", {
    files: files.map((file) => ({ name: file.name, size: file.size, type: file.type }))
  });

  if (files.length === 0) return;

  isImporting.value = true;
  importMessage.value = t("VideoPlayer.ImportingFiles", { count: files.length });

  try {
    const parsed = await parseFiles(files);

    console.info("[VideoPlayer:DnD] parser returned", {
      inputCount: files.length,
      itemCount: parsed.length,
      items: parsed.map((item) => ({ name: item.name, type: item.type, recordingId: item.recordingId }))
    });

    if (parsed.length === 0) {
      importMessage.value = "";
      toast.add({
        title: t("VideoPlayer.NoPlayableFilesTitle"),
        description: t("VideoPlayer.NoPlayableFilesDescription"),
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
        title: t("VideoPlayer.DuplicatesSkippedTitle"),
        description: t("VideoPlayer.DuplicatesSkippedDescription", { count: duplicates }),
        color: "neutral"
      });
    }
  } catch (error: any) {
    console.error("[VideoPlayer:DnD] import failed", error);
    importMessage.value = "";
    toast.add({
      title: t("VideoPlayer.ImportFailed"),
      description: error?.message || String(error),
      color: "error"
    });
  } finally {
    isImporting.value = false;
  }
}

function toggleThemeMode() {
  manualSetTheme(isDarkMode.value ? "light" : "dark");
}

async function handleWindowDrag(event: MouseEvent) {
  const target = event.target as HTMLElement | null;

  if (
    event.button !== 0
    || target?.closest("button")
    || target?.closest('[role="button"]')
    || target?.closest("input")
    || target?.closest("select")
  ) {
    return;
  }

  try {
    await useTauriWindowGetCurrentWindow().startDragging();
  } catch {
    // ignore when running in browser
  }
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
      await currentWindow.setSize(new useTauriWindowLogicalSize(nextWidth, nextHeight));
    }
  } catch (error) {
    console.debug("optimize video player window failed", error);
  }
}

onMounted(async () => {
  document.title = "JumpServer Video Player";

  try {
    const currentWindow = useTauriWindowGetCurrentWindow();
    unlistenFileDrop = await currentWindow.onDragDropEvent(async ({ payload }) => {
      if (payload.type === "enter" || payload.type === "over") {
        isDraggingFiles.value = true;
        return;
      }

      isDraggingFiles.value = false;

      if (payload.type === "drop" && payload.paths.length > 0) {
        try {
          const files = await readDroppedFiles(payload.paths);
          await importFiles(files);
        } catch (error: any) {
          toast.add({
            title: t("VideoPlayer.ImportFailed"),
            description: error?.message || String(error),
            color: "error"
          });
        }
      }
    });
    await currentWindow.setTitle("JumpServer Video Player");
    await optimizeWindowForVideoPlayer();
  } catch {
    // ignore when running in browser
  }
});

onBeforeUnmount(async () => {
  unlistenFileDrop?.();
  unlistenFileDrop = null;
  await Promise.all(items.value.map((item) => cleanupItem(item)));

  try {
    await useTauriWindowGetCurrentWindow().setMinSize(null);
  } catch {
    // ignore when running in browser
  }
});
</script>

<template>
  <div class="videoplayer-page relative h-screen overflow-hidden">
    <div class="mx-auto flex h-full w-full max-w-[1700px] flex-col px-6 py-5 lg:px-8">
      <header class="relative mb-2 h-8" @mousedown="handleWindowDrag">
        <div data-tauri-drag-region class="absolute inset-0 z-0" />

        <div
          class="relative z-10 flex h-full items-center justify-end gap-2 pointer-events-auto"
          data-tauri-drag-region="false"
        >
          <UButton
            color="neutral"
            variant="ghost"
            :icon="
              isDarkMode
                ? 'line-md:moon-filled-to-sunny-filled-loop-transition'
                : 'line-md:sunny-filled-loop-to-moon-filled-transition'
            "
            data-tauri-drag-region="false"
            @click="toggleThemeMode"
          >
            {{ isDarkMode ? t("VideoPlayer.SwitchToLight") : t("VideoPlayer.SwitchToDark") }}
          </UButton>
        </div>
      </header>

      <p v-if="importMessage && items.length === 0" class="mb-4 text-sm text-(--ui-text-muted)">
        {{ importMessage }}
      </p>

      <div class="grid min-h-0 flex-1 grid-cols-[minmax(0,1.9fr)_minmax(280px,0.78fr)] gap-5">
        <section class="min-h-0 min-w-0 overflow-hidden rounded-xl border-2 border-(--ui-border)">
          <div class="flex h-full min-h-0 overflow-hidden bg-black">
            <div class="h-full min-w-0 flex-1 overflow-hidden bg-black">
              <component
                :is="playerComponent"
                v-if="playerComponent && currentItem"
                :key="currentItem.id"
                :source="currentItem.source"
                :cast-data="currentItem.castData"
              />
              <VideoPlayerDropzone
                v-else
                compact
                :disabled="isImporting"
                @select-files="importFiles"
              />
            </div>
          </div>
        </section>

        <aside class="min-h-0 min-w-0 overflow-hidden">
          <VideoPlayerPlaylist
            v-if="items.length > 0"
            :active-id="activeId"
            :items="items"
            @play="selectItem"
            @remove="removeItem"
            @select-files="importFiles"
          />
          <div
            v-else
            class="flex h-full min-h-0 flex-col rounded-xl border-2 border-(--ui-border) p-4"
          >
            <p class="mb-3 text-[11px] uppercase tracking-[0.2em] text-(--ui-text-dimmed)">
              {{ t("VideoPlayer.Playlist") }}
            </p>

            <div
              class="flex min-h-0 flex-1 items-center justify-center rounded-lg border border-dashed border-(--ui-border) p-3"
            >
              <div class="flex max-w-[240px] flex-col items-center text-center">
                <div
                  class="flex h-12 w-12 items-center justify-center rounded-xl border border-(--ui-border) text-2xl text-(--ui-text-dimmed)"
                >
                  <UIcon name="line-md:list-3" />
                </div>
                <p class="mt-4 text-sm font-medium text-(--ui-text-highlighted)">
                  {{ t("VideoPlayer.EmptyPlaylist") }}
                </p>
                <p class="mt-2 text-xs leading-5 text-(--ui-text-muted)">
                  {{ t("VideoPlayer.EmptyPlaylistDescription") }}
                </p>
              </div>
            </div>
          </div>
        </aside>
      </div>
    </div>

    <div
      v-if="isDraggingFiles"
      class="pointer-events-none absolute inset-3 z-50 flex items-center justify-center rounded-2xl border-2 border-dashed border-(--ui-primary) bg-(--ui-bg)/90 backdrop-blur-sm"
    >
      <div class="flex flex-col items-center gap-4 text-center text-(--ui-primary)">
        <UIcon name="line-md:upload-loop" class="text-5xl" />
        <p class="text-lg font-semibold">
          {{ t("VideoPlayer.DropToImport") }}
        </p>
      </div>
    </div>
  </div>
</template>

<style scoped>
.videoplayer-page {
  color: var(--ui-text);
  background:
    radial-gradient(
      circle at top left,
      color-mix(in srgb, var(--ui-color-primary-500) 14%, transparent) 0%,
      transparent 30%
    ),
    radial-gradient(circle at right, color-mix(in srgb, var(--ui-bg-elevated) 65%, transparent) 0%, transparent 26%),
    linear-gradient(
      180deg,
      color-mix(in srgb, var(--ui-bg) 90%, var(--ui-bg-elevated) 10%) 0%,
      color-mix(in srgb, var(--ui-bg) 98%, black 2%) 100%
    );
}
</style>
