<template>
  <n-spin :show="loadingStatus" class="w-full h-[80%]">
    <MainSection
      :list-data="listData"
      :class="active ? 'show-drawer' : ''"
      @loadMore="handleScroll"
    />
  </n-spin>
</template>

<script setup lang="ts">
import { onBeforeUnmount, onMounted } from 'vue';
import mittBus from '@renderer/eventBus';
import MainSection from '@renderer/components/MainSection/index.vue';
import { useAssetList } from '@renderer/composables/useAssetList';

defineProps<{
  active: boolean;
}>();

const { hasMore, loadingStatus, listData, handleScroll, getAssetsFromServer } = useAssetList('favorite');

onMounted(() => {
  getAssetsFromServer();
  mittBus.on('search', getAssetsFromServer);
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
