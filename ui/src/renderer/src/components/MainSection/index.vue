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
import ConnectModal from '@renderer/components/ConnectModal/index.vue';

import { createConnectToken, getAssetDetail, getLocalClientUrl } from '@renderer/api/modals/asset';
import { useHistoryStore } from '@renderer/store/module/historyStore';
import { onBeforeUnmount, onMounted, ref } from 'vue';
import { createDiscreteApi } from 'naive-ui';

import { Conf } from 'electron-conf/renderer';
import type { IConnectData } from '@renderer/store/interface';
import type { IListItem } from '@renderer/components/MainSection/interface';
import type { InternalAxiosError } from 'axios';

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
    listData?: IListItem[];
  }>(),
  {
    listData: () => [] as IListItem[]
  }
);

const conf = new Conf();

// @ts-ignore
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

/**
 * @description grid 布局与 list 布局的切换
 * @param layout
 */
const handleLayoutChange = async (layout: string) => {
  const currentSettings = (await conf.get('defaultSetting')) as Record<string, any>;
  currentLayout.value = layout;
  await conf.set('defaultSetting', {
    ...currentSettings,
    layout: layout
  });
};

/**
 * @description 获取资产详情
 * @param id
 */
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

/**
 * @description 选择账号
 * @param _
 * @param item
 * @param _event
 */
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

/**
 * @description 连接资产
 * @param connectData
 */
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
      try {
        const token = await createConnectToken(connectData, method);

        if (token) {
          message.success('连接成功', { closable: true });

          // todo)) 设置历史
          // historyStore.setHistorySession({ ...selectedItem.value });

          getLocalClientUrl(token).then(res => {
            if (res) {
              window.electron.ipcRenderer.send('open-client', res.url);
            }
          });
        }
      } catch (error: InternalAxiosError) {
        const errorData = error.response.data;

        if (errorData) {
          // 除开通知和允许，其余情况一率弹窗
          if (errorData?.code !== 'notice') {
            message.error(`当前资产仅支持通过 Web 方式访问（ACL：Action）`);

            return;
          }

          if (errorData?.code !== 'reject') {
            message.error(`当前资产拒绝连接`);
            return;
          }
        }
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
