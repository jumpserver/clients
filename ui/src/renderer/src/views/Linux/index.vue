<template>
  <MainSection
    :list-data="listData"
    :class="active ? 'show-drawer' : ''"
    @loadMore="handleScroll"
  />
</template>

<script setup lang="ts">
import mittBus from '@renderer/eventBus';
import MainSection from '@renderer/components/MainSection/index.vue';

import { onBeforeUnmount, onMounted } from 'vue';
import { useAssetList } from '@renderer/hooks/useAssetList';

defineProps<{
  active: boolean;
}>();

const { listData, handleScroll, getAssetsFromServer } = useAssetList('linux');

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
