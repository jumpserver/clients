<template>
  <n-spin :show="loadingStatus" class="w-full h-[80%]">
    <MainSection :list-data="listData" :class="''" />
  </n-spin>
</template>

<script setup lang="ts">
import { onBeforeUnmount, onMounted, ref } from 'vue';

import mittBus from '@renderer/eventBus';
import MainSection from '@renderer/components/MainSection/index.vue';
import { useHistoryStore } from '@renderer/store/module/historyStore';

defineProps<{
  active: boolean;
}>();

const listData = ref([]);
const loadingStatus = ref(true);
const historyStore = useHistoryStore();

const getHistoriesFromCache = async (searchInput?: string) => {
  if (searchInput !== undefined) {
    listData.value = [];
  }
  loadingStatus.value = true;
  listData.value = historyStore.history_session;
  loadingStatus.value = false;
};

onMounted(async () => {
  mittBus.on('search', getHistoriesFromCache);
  await getHistoriesFromCache();
});

onBeforeUnmount(() => {
  mittBus.off('search', getHistoriesFromCache);
});
</script>

<style scoped lang="scss">
:deep(.n-dropdown-option) {
  height: 40px;
}

.show-drawer {
  width: calc(100% - 340px);
}
</style>
