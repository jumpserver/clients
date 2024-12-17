<template>
  <n-layout has-sider>
    <n-layout-sider bordered class="!w-[185px]">
      <SideMenu />
    </n-layout-sider>
    <n-layout>
      <n-layout-content>
        <header-section :active="active" />
        <n-loading-bar-provider>
          <router-view :active="active" />
          <div id="drawer-target"></div>
        </n-loading-bar-provider>
      </n-layout-content>
    </n-layout>
  </n-layout>
  <Drawer :active="active" />
</template>

<script setup lang="ts">
import HeaderSection from './components/HeaderSection/index.vue';
import Drawer from '@renderer/components/Drawer/index.vue';
import SideMenu from './components/sideMenu/index.vue';

import { onBeforeUnmount, onMounted, ref } from 'vue';

import mittBus from '@renderer/eventBus';

const active = ref(false);

const handleCreateDrawer = () => {
  active.value = !active.value;
};

onMounted(() => {
  mittBus.on('createDrawer', handleCreateDrawer);
});

onBeforeUnmount(() => {
  mittBus.off('createDrawer', handleCreateDrawer);
});
</script>

<style scoped lang="scss">
@use './index.scss';
</style>
