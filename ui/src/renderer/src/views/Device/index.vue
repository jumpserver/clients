<template>
  <MainSection
    :list-data="listData"
    :class="active ? 'show-drawer' : ''"
    @load-more="handleScroll"
  />
</template>

<script setup lang="ts">
import { onBeforeUnmount, onMounted } from 'vue';
import mittBus from '@renderer/eventBus';
import MainSection from '@renderer/components/MainSection/index.vue';
import { useAssetList } from '@renderer/hooks/useAssetList';

defineProps<{
  active: boolean;
}>();

const { listData, handleScroll, getAssetsFromServer } = useAssetList('device');

onMounted(() => {
  mittBus.on('search', getAssetsFromServer);
});

onBeforeUnmount(() => {
  mittBus.off('search', getAssetsFromServer);
});
</script>

<style scoped>
:deep(.n-dropdown-option) {
  height: 40px;
}

.show-drawer {
  width: calc(100% - 340px);
}
</style>
