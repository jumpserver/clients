<script setup lang="ts">
import * as AsciinemaPlayer from "@cyolosecurity/asciinema-player";

const props = defineProps<{
  source: string
}>();

const terminalRef = ref<HTMLElement | null>(null);

function mountPlayer() {
  if (!terminalRef.value) return;

  terminalRef.value.innerHTML = "";
  AsciinemaPlayer.create(props.source, terminalRef.value, {
    fit: "both",
    preload: true,
    autoplay: true,
    terminalFontSize: "14px"
  });
}

onMounted(mountPlayer);
watch(() => props.source, () => nextTick(mountPlayer));
</script>

<template>
  <div class="relative isolate h-full w-full overflow-hidden bg-black">
    <div ref="terminalRef" class="h-full w-full overflow-hidden" />
  </div>
</template>

<style scoped>
@import "@cyolosecurity/asciinema-player/dist/bundle/asciinema-player.css";

:deep(.ap-wrapper) {
  position: relative;
  z-index: 0;
  width: 100%;
  height: 100%;
  overflow: hidden;
  box-sizing: border-box;
}

:deep(.ap-wrapper),
:deep(.ap-player) {
  width: 100%;
  height: 100%;
}

:deep(.ap-player) {
  overflow: hidden;
}

:deep(.ap-player),
:deep(.ap-terminal),
:deep(.xterm) {
  position: relative;
  z-index: 0;
  max-width: 100%;
}

:deep(.ap-terminal) {
  width: 100%;
  height: 100%;
  overflow: hidden;
}

:deep(.xterm),
:deep(.xterm-screen),
:deep(.xterm-viewport) {
  width: 100% !important;
  max-width: 100%;
  box-sizing: border-box;
}

:deep(.xterm-viewport) {
  overflow-y: auto !important;
  overflow-x: hidden !important;
  scrollbar-gutter: stable;
}

:deep(.xterm-screen canvas) {
  max-width: 100%;
}
</style>
