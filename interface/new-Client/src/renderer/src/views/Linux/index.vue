<template>
  <MainSection
    :list-data="staticListData"
    :general-icon-name="iconName"
    :class="active ? 'show-drawer' : ''"
  />
</template>

<script setup lang="ts">
import MainSection from '@renderer/components/MainSection/index.vue';

defineProps<{
  active: boolean;
}>();

const staticListData = [
  {
    assetName: 'yy1'
  },
  {
    assetName: 'yy2'
  },
  {
    assetName: 'yy3'
  },
  {
    assetName: 'yy4'
  }
];
const iconName = 'terminal';
</script>

<style scoped lang="scss">
:deep(.n-dropdown-option) {
  height: 40px;
}

.show-drawer {
  width: calc(100% - 340px);
}
</style>
