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
            :class="{ 'bg-secondary': selectedItem.name === item.name }"
            @click="selectItem(item, $event)"
            @contextmenu="handleItemContextMenu(item, $event)"
          />
        </n-infinite-scroll>
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
      @select="handleAccountSelect"
      class="w-40"
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
      class="w-40"
    />
  </div>
</template>

<script setup lang="ts">
import mittBus from '@renderer/eventBus';
import ListItem from '../ListItem/index.vue';

import { createConnectToken, getAssetDetail, getLocalClientUrl } from '@renderer/api/modals/asset';
import { moveElementToEnd, renderCustomHeader } from '@renderer/components/MainSection/helper';
import { useHistoryStore } from '@renderer/store/module/historyStore';
import { onBeforeUnmount, onMounted, ref } from 'vue';
import { createDiscreteApi } from 'naive-ui';

import { Conf } from 'electron-conf/renderer';
import {
  IItemDetail,
  IListItem,
  Permed_accounts,
  Permed_protocols
} from '@renderer/components/MainSection/interface';

import type { Ref } from 'vue';
import type { DropdownOption } from 'naive-ui';
import type { IConnectData } from '@renderer/store/interface';

import { ClipboardList, PlugConnected } from '@vicons/tabler';
import { ProtocolHandler24Regular } from '@vicons/fluent';

const { message } = createDiscreteApi(['message']);

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

const xLeft = ref(0);
const yLeft = ref(0);
const xRight = ref(0);
const yRight = ref(0);

const showConnectModal = ref(false);
const showLeftDropdown = ref(false);
const showRightDropdown = ref(false);

const leftOptions: Ref<DropdownOption[]> = ref([
  {
    key: 'header',
    type: 'render',
    render: renderCustomHeader(ClipboardList, '账号列表')
  },
  {
    key: 'header-divider',
    type: 'divider'
  }
]);
const rightOptions: Ref<DropdownOption[]> = ref([
  {
    key: 'fast-connection',
    type: 'render',
    render: renderCustomHeader(PlugConnected, '快速连接')
  },
  {
    key: 'header-divider',
    type: 'divider'
  },
  {
    key: 'connect-protocol',
    type: 'render',
    render: renderCustomHeader(ProtocolHandler24Regular, '连接协议')
  }
]);
const selectedItem: Ref<IListItem> = ref({} as IListItem);
const connectData: Ref<IConnectData> = ref({} as IConnectData);
const detailMessage: Ref<IItemDetail> = ref({} as IItemDetail);

const currentLayout = ref('');

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
 * @description 关闭 dropdown
 */
const onClickLeftOutside = () => {
  showLeftDropdown.value = false;

  leftOptions.value = [
    {
      key: 'header',
      type: 'render',
      render: renderCustomHeader(ClipboardList, '账号列表')
    },
    {
      key: 'header-divider',
      type: 'divider'
    }
  ];
};

/**
 * @description 关闭 dropdown
 */
const onClickRightOutside = () => {
  showRightDropdown.value = false;

  rightOptions.value = [
    {
      key: 'fast-connection',
      type: 'render',
      render: renderCustomHeader(PlugConnected, '快速连接')
    },
    {
      key: 'header-divider',
      type: 'divider'
    },
    {
      key: 'connect-protocol',
      type: 'render',
      render: renderCustomHeader(ProtocolHandler24Regular, '连接协议')
    }
  ];
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
const getAssetDetailFromServer = async (id: string): Promise<boolean> => {
  try {
    const res: IItemDetail = await getAssetDetail(id);

    detailMessage.value = res;

    if (res) {
      return Promise.resolve(true);
    }

    return Promise.resolve(false);
  } catch (e) {
    console.log(e);
    message.error('获取资产数据列表失败');
    return Promise.resolve(false);
  }
};

/**
 * @description 左键选择账号
 * @param item
 * @param _event
 */
const selectItem = async (item: IListItem, _event: MouseEvent) => {
  selectedItem.value = item;

  try {
    const hasGetMessage: boolean = await getAssetDetailFromServer(item.id);

    if (hasGetMessage) {
      detailMessage.value.permed_accounts
        .filter((item: Permed_accounts) => item.alias !== '@ANON')
        .forEach((item: Permed_accounts) => {
          leftOptions.value.push({
            key: item.id,
            label: item.username
          });
        });

      leftOptions.value = moveElementToEnd(leftOptions.value, '@INPUT', '手动输入');

      showLeftDropdown.value = true;
      xLeft.value = _event.clientX;
      yLeft.value = _event.clientY;
    }
  } catch (e) {
    message.error('获取资产数据详情失败');
    showLeftDropdown.value = false;
  }
};

/**
 * @description 右键选择协议
 * @param _item
 * @param _event
 */
const handleItemContextMenu = async (_item: IListItem, _event: MouseEvent) => {
  try {
    const hasGetMessage: boolean = await getAssetDetailFromServer(_item.id);

    detailMessage.value.permed_protocols
      .filter((item: Permed_protocols) => item.public)
      .forEach((item: Permed_protocols) => {
        rightOptions.value.push({
          key: item.name,
          label: item.name
        });
      });

    showRightDropdown.value = true;
    xRight.value = _event.clientX;
    yRight.value = _event.clientY;
  } catch (e) {
    showRightDropdown.value = false;
  }
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
      } catch (error: any) {
        const errorData = error?.response.data;

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

const handleAccountSelect = (key: string) => {
  connectData.value = {
    protocol: '',
    asset: '',
    account: '',
    input_username: '',
    input_secret: ''
  };

  const currentAccount = detailMessage.value.permed_accounts.find(
    (item: Permed_accounts) => item.id === key
  );

  if (currentAccount) {
    connectData.value.asset = currentAccount.id;
    connectData.value.account = currentAccount.username;
  }

  console.log(selectedItem.value);
  console.log(detailMessage.value);

  showLeftDropdown.value = false;
};

const handleSelect = (key: string) => {
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
