<template>
  <div>
    <n-flex class="h-[calc(100vh-135px)]">
      <n-list hoverable clickable :show-divider="false" class="w-full h-full">
        <template #header>
          <n-h3 class="h-full" strong> Hosts</n-h3>
        </template>
        <n-infinite-scroll
          style="max-height: calc(100vh - 200px)"
          :class="{ 'list-layout': currentLayout !== 'list' }"
          :distance="10"
          @load="handleLoad"
        >
          <ListItem
            v-for="(item, index) of listData"
            :key="index"
            :item-data="item"
            :layout="currentLayout"
            :class="{ 'bg-secondary': selectedItem === index }"
            @click="selectAccount(index, item, $event)"
          />
        </n-infinite-scroll>
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
  </div>
</template>

<script setup lang="ts">
import mittBus from '@renderer/eventBus';
import ListItem from '../ListItem/index.vue';
import { onBeforeUnmount, onMounted, ref } from 'vue';

import { createDiscreteApi, SelectOption } from 'naive-ui';

import { Conf } from 'electron-conf/renderer';
import { createConnectToken, getAssetDetail, getLocalClientUrl } from '@renderer/api/modals/asset';

const { message } = createDiscreteApi(['message']);

interface Itype {
  value: string;
  label: string;
}

interface Item {
  id?: string;
  type?: Itype;
}

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
const currentLayout = ref('');
const showLeftDropdown = ref(false);
const selectedItem = ref<Item>({});

const leftOptions = ref([]);

const emit = defineEmits(['loadMore']);

// 初始值
conf.get('defaultSetting').then(res => {
  if (res) {
    // @ts-ignore
    currentLayout.value = res.layout;
  }
});

const handleLoad = () => {
  emit('loadMore');
};

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

const selectAccount = (_: number, item: any, e: MouseEvent) => {
  showLeftDropdown.value = false;
  selectedItem.value = item;

  getAssetDetailFromServer(item.id).then(() => {
    showLeftDropdown.value = true;
    xLeft.value = e.clientX;
    yLeft.value = e.clientY;
  });
};

const onClickLeftOutside = () => {
  showLeftDropdown.value = false;
  selectedItem.value = {};
};

const handleSelect = async (_key: string, _option: SelectOption) => {
  showLeftDropdown.value = false;
  let method: string;
  let protcol: string;
  if (selectedItem) {
    switch (selectedItem.value.type!.value) {
      case 'linux':
        method = 'ssh_client';
        protcol = 'ssh';
        break;
      case 'windows':
        method = 'mstsc';
        protcol = 'rdp';
        break;
      default:
        method = 'db_client';
        protcol = 'mysql';
    }
    if (selectedItem.value.id) {
      const token = await createConnectToken(
        selectedItem.value.id,
        _option as { name?: string },
        '',
        method,
        protcol
      );
      message.success('连接成功', { closable: true });
      const res = await getLocalClientUrl(token);
      if (res) {
        window.electron.ipcRenderer.send('open-client', res.url);
      }
    }
  }
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
