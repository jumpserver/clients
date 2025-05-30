<template>
  <div style="height: calc(100vh - 3.2rem)">
    <n-flex align="center" class="title h-12">
      <n-h3 strong class="flex items-center h-full pl-4 !mb-0"> {{ t('Common.AssetsList') }} </n-h3>

      <n-popover trigger="hover" placement="bottom-end">
        <template #trigger>
          <CircleHelp :size="18" class="icon-hover mt-1" />
        </template>

        <n-text depth="2">
          {{ t('Common.ConnectionTip') }}
        </n-text>
      </n-popover>
    </n-flex>

    <n-infinite-scroll
      style="max-height: calc(100vh - 11rem)"
      class="h-full"
      :class="{ 'list-layout': currentLayout !== 'list' }"
      :distance="10"
      @load="debounceLoad"
    >
      <list-container
        v-if="listData.length > 0"
        :current-layout="currentLayout"
        :list-data="listData"
      />
      <n-empty
        v-else
        :description="t('Common.NoData')"
        class="absolute top-0 left-0 h-full w-full items-center justify-center"
      />
    </n-infinite-scroll>
  </div>
</template>

<script setup lang="ts">
import mittBus from '@renderer/eventBus';
import ListContainer from '@renderer/components/ListContainer/index.vue';

import { useI18n } from 'vue-i18n';
import { useDebounceFn } from '@vueuse/core';
import { onBeforeUnmount, onMounted, ref } from 'vue';

import { CircleHelp } from 'lucide-vue-next';
import { useElectronConfig } from '@renderer/hooks/useElectronConfig';

import type { IListItem } from '@renderer/components/MainSection/interface';

const { t } = useI18n();
const { getDefaultSetting, setDefaultSetting } = useElectronConfig();

withDefaults(
  defineProps<{
    listData?: IListItem[];
  }>(),
  {
    listData: () => [] as IListItem[]
  }
);

const emit = defineEmits(['loadMore']);

const currentLayout = ref('');

const debounceLoad = useDebounceFn(() => {
  emit('loadMore');
}, 500);

const handleLayoutChange = async (layout: string) => {
  currentLayout.value = layout;

  await setDefaultSetting({ layout });
};

onMounted(async () => {
  const { layout } = await getDefaultSetting();

  currentLayout.value = layout;

  mittBus.on('changeLayout', handleLayoutChange);
});

onBeforeUnmount(() => {
  mittBus.off('changeLayout', handleLayoutChange);
});
</script>

<style scoped lang="scss">
@use './index.scss';
</style>
