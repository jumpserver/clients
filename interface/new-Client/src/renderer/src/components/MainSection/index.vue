<template>
  <div>
    <n-flex class="h-[calc(100vh-135px)]">
      <n-list hoverable clickable :show-divider="false" class="w-full h-full">
        <template #header>
          <n-h3 class="h-full" strong> Hosts </n-h3>
        </template>
        <n-scrollbar
          style="max-height: calc(100vh - 200px)"
          :class="{ 'list-layout': currentLayout !== 'list' }"
        >
          <n-empty
            v-if="listData.length === 0"
            class="w-full absolute top-[50%] left-[50%] -translate-x-[50%] -translate-y-[100%]"
            description="暂无数据"
          />
          <ListItem
            v-else
            v-for="(item, index) of listData"
            :key="index"
            :item-data="item"
            :layout="currentLayout"
            :general-icon-name="generalIconName"
            :class="{ 'bg-secondary': selectedItem === index }"
            @click="selectAccount(index, $event)"
            @contextmenu="handleItemContextMenu(index, $event)"
          />
        </n-scrollbar>
      </n-list>
    </n-flex>

    <!-- 左键点击下拉菜单 -->
    <n-dropdown
      placement="bottom-start"
      trigger="manual"
      size="small"
      :x="xLeft"
      :y="yLeft"
      :show-arrow="true"
      :options="leftOptions"
      :show="showLeftDropdown"
      :on-clickoutside="onClickLeftOutside"
      @select="handleSelect"
      class="w-[150px]"
    />

    <!-- 右键点击下拉菜单 -->
    <n-dropdown
      placement="bottom-start"
      trigger="manual"
      size="small"
      :x="xRight"
      :y="yRight"
      :show-arrow="true"
      :options="rightOptions"
      :show="showRightDropdown"
      :on-clickoutside="onClickRightOutside"
      @select="handleSelect"
      class="w-[150px]"
    />
  </div>
</template>

<script setup lang="ts">
import mittBus from '@renderer/eventBus';
import ListItem from '../ListItem/index.vue';
import { onBeforeUnmount, onMounted, ref, nextTick } from 'vue';

import type { SelectOption } from 'naive-ui';

import { Conf } from 'electron-conf/renderer';

withDefaults(
  defineProps<{
    listData: any;
    generalIconName: string;
  }>(),
  {
    listData: [],
    generalIconName: ''
  }
);

const conf = new Conf();
const xLeft = ref(0);
const yLeft = ref(0);
const xRight = ref(0);
const yRight = ref(0);
const currentLayout = ref('');
const showLeftDropdown = ref(false);
const showRightDropdown = ref(false);
const selectedItem = ref<number | null>(null);

const leftOptions = [
  {
    type: 'group',
    label: '本地账户',
    kay: 'local-account',
    children: [
      {
        label: 'root',
        key: 'root'
      },
      {
        label: 'Administrator',
        key: 'administrator'
      }
    ]
  }
];
const rightOptions = [
  {
    type: 'group',
    label: '连接方式',
    kay: 'connect-type',
    children: [
      {
        label: 'SSH',
        key: 'ssh'
      },
      {
        label: 'SFTP',
        key: 'sftp'
      }
    ]
  }
];

// 初始值
conf.get('defaultSetting').then(res => {
  if (res) {
    // @ts-ignore
    currentLayout.value = res.layout;
  }
});

const handleLayoutChange = async (layout: string) => {
  const currentSettings = (await conf.get('defaultSetting')) as Record<string, any>;

  currentLayout.value = layout;

  await conf.set('defaultSetting', {
    ...currentSettings,
    layout: layout
  });
};

const selectAccount = (index: number, e: MouseEvent) => {
  showLeftDropdown.value = false;
  selectedItem.value = index;

  nextTick().then(() => {
    showLeftDropdown.value = true;
    xLeft.value = e.clientX;
    yLeft.value = e.clientY;
  });
};

const handleItemContextMenu = (index: number, e: MouseEvent) => {
  showRightDropdown.value = true;

  selectedItem.value = index;

  nextTick().then(() => {
    showRightDropdown.value = true;
    xRight.value = e.clientX;
    yRight.value = e.clientY;
  });
};

const onClickLeftOutside = () => {
  showLeftDropdown.value = false;
  selectedItem.value = null;
};

const onClickRightOutside = () => {
  showRightDropdown.value = false;
};

const handleSelect = (_key: string, _option: SelectOption) => {
  showLeftDropdown.value = false;
};

onMounted(() => {
  mittBus.on('changeLayout', handleLayoutChange);
});

onBeforeUnmount(() => {
  mittBus.off('changeLayout', handleLayoutChange);
});
</script>

<style scoped lang="scss">
@use './index.scss';
</style>
