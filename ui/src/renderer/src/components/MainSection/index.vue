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
            @contextmenu="handleContextMenuWrapper(item, $event)"
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
      class="min-w-40"
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
      class="min-w-40"
    />
  </div>
</template>

<script setup lang="ts">
import mittBus from '@renderer/eventBus';
import ListItem from '../ListItem/index.vue';

import { createConnectToken, getAssetDetail, getLocalClientUrl } from '@renderer/api/modals/asset';
import { moveElementToEnd, renderCustomHeader, useAccountModal } from './helper/index';
import { useHistoryStore } from '@renderer/store/module/historyStore';
import { createDiscreteApi, useLoadingBar } from 'naive-ui';
import { computed, onBeforeUnmount, onMounted, Ref, ref } from 'vue';
import { useDebounceFn } from '@vueuse/core';

import { Conf } from 'electron-conf/renderer';

import type {
  IItemDetail,
  IListItem,
  Permed_accounts,
  Permed_protocols
} from '@renderer/components/MainSection/interface';
import type { DropdownOption } from 'naive-ui';
import type { IConnectData } from '@renderer/store/interface';

import { ClipboardList, PlugConnected } from '@vicons/tabler';
import { ArrowEnterLeft20Filled, ProtocolHandler24Regular } from '@vicons/fluent';

import { storeToRefs } from 'pinia';
import { useUserStore } from '@renderer/store/module/userStore';

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

const loadingBar = useLoadingBar();
// @ts-ignore
const historyStore = useHistoryStore();

const userStore = useUserStore();
const { currentUser: storeCurrentUser } = storeToRefs(userStore);
const currentUser = computed(() => storeCurrentUser?.value);

const xLeft = ref(0);
const yLeft = ref(0);
const xRight = ref(0);
const yRight = ref(0);

const showLeftDropdown = ref(false);
const showRightDropdown = ref(false);

const selectedItem: Ref<IListItem> = ref({} as IListItem);
const connectData: Ref<IConnectData> = ref({} as IConnectData);
const detailMessage: Ref<IItemDetail> = ref({} as IItemDetail);
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
    render: renderCustomHeader(PlugConnected, '快速连接', 'fast-connection')
  },
  {
    key: 'detail-message',
    type: 'render',
    render: renderCustomHeader(
      ArrowEnterLeft20Filled,
      '资产详情',
      'detail-message',
      detailMessage,
      () => {
        showRightDropdown.value = false;
      }
    )
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

const currentLayout = ref('');

const emit = defineEmits(['loadMore']);

// 初始值
conf.get('defaultSetting').then(res => {
  if (res) {
    // @ts-ignore
    currentLayout.value = res?.layout;
  }
});

const handleLoad = () => {
  emit('loadMore');
};

/**
 * @description 重置左键菜单
 */
const resetLeftOptions = () => {
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
 * @description 重置右键菜单
 */
const resetRightOptions = () => {
  rightOptions.value = [
    {
      key: 'fast-connection',
      type: 'render',
      render: renderCustomHeader(PlugConnected, '快速连接', 'fast-connection')
    },
    {
      key: 'detail-message',
      type: 'render',
      render: renderCustomHeader(ArrowEnterLeft20Filled, '资产详情', 'detail-message')
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
 * @description 关闭 dropdown
 */
const onClickLeftOutside = () => {
  showLeftDropdown.value = false;

  resetLeftOptions();
};

/**
 * @description 关闭 dropdown
 */
const onClickRightOutside = () => {
  showRightDropdown.value = false;

  resetRightOptions();
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
    message.error('获取资产数列表失败');
    return Promise.resolve(false);
  }
};

/**
 * @description listItem 的点击事件
 * @param item
 * @param _event
 */
const selectItem = useDebounceFn(async (item: IListItem, _event: MouseEvent) => {
  loadingBar.start();
  selectedItem.value = item;

  try {
    const hasGetMessage: boolean = await getAssetDetailFromServer(item.id);

    if (hasGetMessage) {
      detailMessage.value.permed_accounts
        .filter((item: Permed_accounts) => item.alias !== '@ANON')
        .forEach((item: Permed_accounts) => {
          leftOptions.value.push({
            key: item.id,
            label:
              item.name +
              (item.alias === item.username || item.alias.startsWith('@')
                ? ''
                : '(' + item.username + ')')
          });
        });

      leftOptions.value = moveElementToEnd(leftOptions.value, '@INPUT', '手动输入');

      loadingBar.finish();
      showLeftDropdown.value = true;
      xLeft.value = _event.clientX;
      yLeft.value = _event.clientY;
    }
  } catch (e) {
    message.error('获取资产数据详情失败');
    loadingBar.error();
    showLeftDropdown.value = false;
  }
}, 300);

/**
 * @description 由于加了 useDebounceFn 会导致 stopPropagation 的行为无法被触发
 * @param event
 * @param item
 */
const handleContextMenuWrapper = (item: IListItem, event: MouseEvent) => {
  event.stopPropagation();

  loadingBar.start();
  handleItemContextMenu(item, event);
};

/**
 * @description contextmenu 的回调
 */
const handleItemContextMenu = useDebounceFn(async (_item: IListItem, _event: MouseEvent) => {
  selectedItem.value = _item;

  try {
    const hasGetMessage: boolean = await getAssetDetailFromServer(_item.id);

    if (hasGetMessage) {
      detailMessage.value.permed_protocols
        .filter((item: Permed_protocols) => item.public)
        .forEach((item: Permed_protocols) => {
          rightOptions.value.push({
            key: item.name,
            label: item.name
          });
        });

      loadingBar.finish();
      showRightDropdown.value = true;
      xRight.value = _event.clientX;
      yRight.value = _event.clientY;
    }
  } catch (e) {
    loadingBar.error();
    showRightDropdown.value = false;
  }
}, 300);

/**
 * @description 左键选择账号
 * @param key
 */
const handleAccountSelect = (key: string) => {
  connectData.value = {
    asset: '',
    account: '',
    protocol: '',
    input_username: '',
    input_secret: ''
  };

  const currentAccount = detailMessage.value.permed_accounts.find(
    (item: Permed_accounts) => item.id === key
  );

  if (currentAccount) {
    connectData.value.asset = detailMessage.value.id;
    connectData.value.account = currentAccount.name;
    connectData.value.input_username = '';
    connectData.value.input_secret = '';

    const showManualUsernameInput = currentAccount.has_secret;

    switch (currentAccount.alias) {
      case '@USER':
        // 同名账号
        connectData.value.account = '@USER';
        connectData.value.input_username = currentUser.value?.username;

        if (showManualUsernameInput) {
          const { inputPassword } = useAccountModal('@USER');

          connectData.value.input_secret = inputPassword.value;
        }
        break;
      case '@INPUT':
        // 手动输入
        connectData.value.account = '@INPUT';
        const { inputPassword, inputUsername } = useAccountModal('@INPUT');

        connectData.value.input_username = inputUsername.value;
        connectData.value.input_secret = inputPassword.value;

        break;
      default:
        connectData.value.input_username = currentAccount.username;

        if (showManualUsernameInput) {
          const { inputPassword } = useAccountModal('@OTHER');

          connectData.value.input_secret = inputPassword.value;
        }
    }
    message.success(`账号 ${currentAccount.name} 已选择`);
  }

  resetLeftOptions();
  showLeftDropdown.value = false;
};

/**
 * @description 右键选择协议
 * @param key
 */
const handleSelect = async (key: string) => {
  if (!connectData.value.account) {
    message.error('请先选择账号');
    return;
  }

  const isCurrentIndex = detailMessage.value.permed_accounts.findIndex(
    item => item.name === connectData.value.account || item.alias === connectData.value.account
  );

  if (isCurrentIndex === -1) {
    message.error('当前账号不在资产账号列表中');
    return;
  }

  const currentProtocol =
    key === 'fast-connection'
      ? detailMessage.value.permed_protocols[0]
      : detailMessage.value.permed_protocols.find((item: Permed_protocols) => item.name === key);

  if (currentProtocol) {
    if (selectedItem.value) {
      let method: string;
      switch (currentProtocol.name) {
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

      try {
        connectData.value.protocol = currentProtocol.name;
        const token = await createConnectToken(connectData.value, method);

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

  resetRightOptions();
  showRightDropdown.value = false;
};

/**
 * @description 快速连接
 */
const handleFastConnect = async () => {
  await handleSelect('fast-connection');
};

onMounted(() => {
  mittBus.on('connectAsset', handleFastConnect);
  mittBus.on('changeLayout', handleLayoutChange);
});

onBeforeUnmount(() => {
  mittBus.off('connectAsset', handleFastConnect);
  mittBus.off('changeLayout', handleLayoutChange);
});
</script>

<style scoped lang="scss">
@use './index.scss';
</style>
