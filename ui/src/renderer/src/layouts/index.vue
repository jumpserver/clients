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
import { useRouter } from 'vue-router';

import type { IItemDetail } from '@renderer/components/MainSection/interface';

import mittBus from '@renderer/eventBus';

const router = useRouter();
const active = ref(false);
const drawerDetailMessage = ref<IItemDetail>({} as IItemDetail);

const handleCreateDrawer = () => {
  active.value = !active.value;
};

window.electron.ipcRenderer.on('set-token', (_event, _message) => {
  router.push({ name: 'Linux' });
});

onMounted(() => {
  mittBus.on('createDrawer', handleCreateDrawer);
  mittBus.on('showAssetDetail', ({ detailMessage }) => {
    if (detailMessage.value) {
      console.log(
        '%c DEBUG[ drawerDetailMessage.value ]-29:',
        'font-size:13px; background:#F0FFF0; color:#7B68EE;',
        drawerDetailMessage.value
      );

      drawerDetailMessage.value = detailMessage.value;
    }

    handleCreateDrawer();
  });
});

onBeforeUnmount(() => {
  mittBus.off('createDrawer', handleCreateDrawer);
  mittBus.off('showAssetDetail', handleCreateDrawer);
});
</script>

<style scoped lang="scss">
@use './index.scss';
</style>
