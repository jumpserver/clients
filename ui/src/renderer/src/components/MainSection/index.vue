<template>
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
  <ConnectModal
    :show-modal="showConnectModal"
    :permed-data="permedData"
    @close-click="handleDialogClose"
    @connect-asset="handleConnectAsset"
  />
</template>

<script setup lang="ts">
import mittBus from '@renderer/eventBus';
import ListItem from '../ListItem/index.vue';
import { onBeforeUnmount, onMounted, ref } from 'vue';
import { createDiscreteApi } from 'naive-ui';
import { Conf } from 'electron-conf/renderer';
import { createConnectToken, getAssetDetail, getLocalClientUrl } from '@renderer/api/modals/asset';
import { useHistoryStore } from '@renderer/store/module/historyStore';
import ConnectModal from '@renderer/components/ConnectModal/index.vue';
import { IConnectData } from '@renderer/store/interface';

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
const historyStore = useHistoryStore();
const currentLayout = ref('');
const showConnectModal = ref(false);
const selectedItem = ref<Item>({});
const permedData = ref(null);

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
      permedData.value = res;
      console.log(res);
    }
  } catch (e) {
    message.error('获取资产数据列表失败', { closable: true });
  }
};

const selectAccount = (_: number, item: any, _event: Event) => {
  showConnectModal.value = false;
  selectedItem.value = item;

  getAssetDetailFromServer(item.id).then(() => {
    showConnectModal.value = true;
  });
};

const handleDialogClose = () => {
  showConnectModal.value = false;
  selectedItem.value = {};
};

const handleConnectAsset = async (connectData: IConnectData) => {
  showConnectModal.value = false;
  if (selectedItem) {
    let method = '';
    switch (connectData.protocol) {
      case 'ssh':
      case 'telnet':
        method = 'ssh_client';
        break;
      case 'rdp':
        method = 'mstsc';
        break;
      case 'sftp':
        method = 'sftp_client';
        break;
      case 'vnc':
        method = 'vnc_client';
        break;
      default:
        method = 'db_client';
    }
    if (selectedItem.value.id) {
      const token = await createConnectToken(connectData, method);
      message.success('连接成功', { closable: true });
      historyStore.setHistorySession({ ...selectedItem.value });
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
