<template>
  <RenderList type="linux" />
</template>

<script setup lang="ts">
import { onBeforeUnmount, onMounted } from 'vue';
import mittBus from '@renderer/eventBus';
import MainSection from '@renderer/components/MainSection/index.vue';
import { useAssetList } from '@renderer/hooks/useAssetList';
import RenderList from '@renderer/components/RenderList/index.vue';

defineProps<{
  active: boolean;
}>();

const { hasMore, loadingStatus, listData, handleScroll, getAssetsFromServer } =
  useAssetList('linux');

onMounted(() => {
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
