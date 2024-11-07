<template>
  <MainSection :list-data="listData" :class="active ? 'show-drawer' : ''" />
</template>

<script setup lang="ts">
import MainSection from '@renderer/components/MainSection/index.vue';
import { getAssets } from '@renderer/api/modals/asset';
import { onBeforeUnmount, onMounted, ref } from 'vue';
import mittBus from '@renderer/eventBus';

defineProps<{
  active: boolean;
}>();

const listData = ref([]);

let params = {
  type: 'windows',
  offset: 0,
  limit: 100,
  search: ''
};

onMounted(async () => {
  mittBus.on('search', getAssetsFromServer);
  await getAssetsFromServer();
});
onBeforeUnmount(() => {
  mittBus.off('search', getAssetsFromServer);
});

const getAssetsFromServer = async (searchInput?: string) => {
  if (searchInput) params.search = searchInput;
  const res = await getAssets(params);
  listData.value = res.results;
};
</script>

<style scoped lang="scss">
:deep(.n-dropdown-option) {
  height: 40px;
}

.show-drawer {
  width: calc(100% - 340px);
}
</style>
