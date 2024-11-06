<template>
  <MainSection
    :list-data="listData"
    :general-icon-name="iconName"
    :class="active ? 'show-drawer' : ''"
  />
</template>

<script setup lang="ts">
import MainSection from '@renderer/components/MainSection/index.vue';
import { getAsset } from '@renderer/api/modals/asset';
import { onMounted, ref } from 'vue';

defineProps<{
  active: boolean;
}>();

const listData = ref([]);
const iconName = 'terminal';

onMounted(async () => {
  const res = await getAsset('linux', 0, 100);
  listData.value = res.results;
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
