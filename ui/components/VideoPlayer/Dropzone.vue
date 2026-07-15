<script setup lang="ts">
const props = withDefaults(
  defineProps<{
    compact?: boolean
    disabled?: boolean
    inline?: boolean
  }>(),
  {
    compact: false,
    disabled: false,
    inline: false
  }
);

const emit = defineEmits<{
  selectFiles: [File[]]
}>();
const { t } = useI18n();
const selectedFiles = ref<File[] | null>([]);

function describeFiles(files: File[] | FileList | null | undefined) {
  return Array.from(files || []).map((file) => ({
    name: file.name,
    size: file.size,
    type: file.type,
    lastModified: file.lastModified
  }));
}

function logDragEvent(stage: "dragenter" | "drop", event: DragEvent) {
  console.info(`[VideoPlayer:DnD] native ${stage}`, {
    inline: props.inline,
    dataTransferTypes: Array.from(event.dataTransfer?.types || []),
    dataTransferItems: Array.from(event.dataTransfer?.items || []).map((item) => ({
      kind: item.kind,
      type: item.type
    })),
    files: describeFiles(event.dataTransfer?.files)
  });
}

function handleFileUploadChange(event: Event) {
  const files = (event.target as EventTarget & { value?: File[] | null } | null)?.value;

  console.info("[VideoPlayer:DnD] UFileUpload change", {
    inline: props.inline,
    files: describeFiles(files)
  });
}

const dropzoneUi = computed(() => {
  if (props.inline) {
    return {
      root: "shrink-0"
    };
  }

  return {
    root: "h-full w-full",
    base: "h-full min-h-0 cursor-pointer rounded-none border-0 bg-black px-6 py-6 data-[dragging=true]:bg-(--ui-primary)/10",
    wrapper: "h-full",
    label: props.compact
      ? "mt-4 text-lg font-semibold tracking-tight text-white"
      : "mt-4 text-xl font-semibold tracking-tight text-white md:text-2xl",
    description: props.compact
      ? "mt-2 max-w-xl text-xs leading-5 text-(--ui-text-muted)"
      : "mt-2 max-w-xl text-sm leading-6 text-(--ui-text-muted)",
    actions: props.compact ? "mt-5" : "mt-6 md:mt-8"
  };
});

watch(selectedFiles, (files) => {
  console.info("[VideoPlayer:DnD] UFileUpload model updated", {
    inline: props.inline,
    files: describeFiles(files)
  });

  if (!files?.length) return;

  console.info("[VideoPlayer:DnD] emitting selectFiles", {
    inline: props.inline,
    files: describeFiles(files)
  });
  emit("selectFiles", [...files]);
  selectedFiles.value = [];
});
</script>

<template>
  <div
    :class="props.inline ? 'shrink-0' : 'h-full w-full'"
    @dragenter.capture="logDragEvent('dragenter', $event)"
    @drop.capture="logDragEvent('drop', $event)"
  >
    <UFileUpload
      v-model="selectedFiles"
      multiple
      reset
      :preview="false"
      :disabled="props.disabled"
      accept=".mp4,.gz,.tar,.map,.json,.cast"
      :label="props.inline ? t('VideoPlayer.AddRecording') : t('VideoPlayer.ImportRecording')"
      :description="props.inline ? undefined : t('VideoPlayer.DropzoneHint')"
      :ui="dropzoneUi"
      @change="handleFileUploadChange"
    >
      <template v-if="props.inline" #default="{ open }">
        <UButton
          color="neutral"
          variant="outline"
          size="xs"
          @click.stop="open"
        >
          {{ t("VideoPlayer.AddRecording") }}
        </UButton>
      </template>

      <template v-if="!props.inline" #leading>
        <div
          class="flex items-center justify-center rounded-xl bg-white/8 text-(--ui-primary)"
          :class="props.compact ? 'h-16 w-16 text-3xl' : 'h-16 w-16 text-3xl md:h-18 md:w-18 md:text-4xl'"
        >
          <UIcon name="line-md:upload-loop" />
        </div>
      </template>

      <template v-if="!props.inline" #actions="{ open }">
        <UButton
          color="neutral"
          variant="soft"
          :size="props.compact ? 'md' : 'lg'"
          @click.stop="open()"
        >
          {{ t("VideoPlayer.SelectFiles") }}
        </UButton>
      </template>
    </UFileUpload>
  </div>
</template>
