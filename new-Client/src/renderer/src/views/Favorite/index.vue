<template>
  <n-spin :show="loadingStatus" class="w-full h-[80%]">
    <MainSection :list-data="listData" :class="active ? 'show-drawer' : ''" />
  </n-spin>
</template>

<script setup lang="ts">
import { nextTick, onBeforeUnmount, onMounted, ref } from 'vue';
import { getFavoriteAssets } from '@renderer/api/modals/asset';
import { useMessage } from 'naive-ui';

import mittBus from '@renderer/eventBus';
import MainSection from '@renderer/components/MainSection/index.vue';

defineProps<{
  active: boolean;
}>();

const message = useMessage();
const params = {
  offset: 0,
  limit: 100,
  search: ''
};

const listData = ref([]);
const loadingStatus = ref(true);

const getAssetsFromServer = async (searchInput?: string) => {
  if (searchInput) params.search = searchInput;

  loadingStatus.value = true;

  try {
    const res = await getFavoriteAssets(params);

    if (res) {
      listData.value = res.results;

      await nextTick(() => {
        loadingStatus.value = false;
      });
    }
  } catch (e) {
    message.error('获取资产数据列表失败', { closable: true });
  }
};

onMounted(async () => {
  mittBus.on('search', getAssetsFromServer);
  await getAssetsFromServer();
});

onBeforeUnmount(() => {
  mittBus.off('search', getAssetsFromServer);
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
