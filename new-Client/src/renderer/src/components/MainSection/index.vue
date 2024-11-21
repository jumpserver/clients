<template>
  <div>
    <n-flex class="h-[calc(100vh-135px)]">
      <n-list hoverable clickable :show-divider="false" class="w-full h-full">
        <template #header>
          <n-h3 class="h-full" strong> Hosts</n-h3>
        </template>
        <n-scrollbar
          style="max-height: calc(100vh - 200px)"
          :class="{ 'list-layout': currentLayout !== 'list' }"
        >
          <ListItem
            v-for="(item, index) of listData"
            :key="index"
            :item-data="item"
            :layout="currentLayout"
            :class="{ 'bg-secondary': selectedItem === index }"
            @click="selectAccount(index, item, $event)"
            @contextmenu="handleItemContextMenu(index, item, $event)"
          />
        </n-scrollbar>
      </n-list>
    </n-flex>

    <!-- 左键点击下拉菜单 -->
    <n-dropdown
      placement="bottom-start"
      trigger="manual"
      size="small"
      label-field="name"
      key-field="username"
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
import { nextTick, onBeforeUnmount, onMounted, ref } from 'vue';

import { createDiscreteApi, SelectOption } from 'naive-ui';

import { Conf } from 'electron-conf/renderer';
import { createConnectToken, getAssetDetail } from '@renderer/api/modals/asset';

const { message } = createDiscreteApi(['message']);

withDefaults(
  defineProps<{
    listData: any;
  }>(),
  {
    listData: []
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
const selectedItem = ref(null);

const leftOptions = ref([]);
const rightOptions = ref([]);

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

const getAssetDetailFromServer = async (id: string) => {
  try {
    const res = await getAssetDetail(id);
    if (res) {
      leftOptions.value = res.permed_accounts;
    }
  } catch (e) {
    message.error('获取资产数据列表失败', { closable: true });
  }
};

const selectAccount = (index: number, item: any, e: MouseEvent) => {
  showLeftDropdown.value = false;
  selectedItem.value = item;

  getAssetDetailFromServer(item.id).then(() => {
    showLeftDropdown.value = true;
    xLeft.value = e.clientX;
    yLeft.value = e.clientY;
  });
};

const handleItemContextMenu = (index: number, item: any, e: MouseEvent) => {
  showRightDropdown.value = true;

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
  createConnectToken(selectedItem.value?.id, _option, 'ssh_client', 'ssh').then(res => {
    if (res) {
      message.success('连接成功', { closable: true });
    }
  });
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
