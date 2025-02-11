<template>
  <div style="height: calc(100vh - 3.2rem)">
    <n-flex class="h-full">
      <n-list hoverable clickable :show-divider="false" class="relative w-full h-full">
        <template #header>
          <n-h3 class="h-full" strong> {{ t('Common.AssetsList') }} </n-h3>
        </template>

        <n-infinite-scroll
          v-if="listData.length > 0"
          style="max-height: calc(100vh - 10.625rem)"
          :class="{ 'list-layout': currentLayout !== 'list' }"
          :distance="10"
          @load="debounceLoad"
        >
          <ListItem
            v-for="item of listData"
            :key="item.id"
            :item-data="item"
            :layout="currentLayout"
            :class="{ 'bg-secondary': selectedItem.id === item.id }"
            @click="selectItem(item, $event)"
            @contextmenu="handleContextMenuWrapper(item, $event)"
          />
        </n-infinite-scroll>

        <n-empty
          v-else
          :description="t('Common.NoData')"
          class="absolute top-0 left-0 h-full w-full items-center justify-center"
        />
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
      class="min-w-60 max-h-80 scrollbar-dark overflow-y-auto p-2"
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
      class="min-w-60"
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
import { onBeforeUnmount, onMounted, ref, watch } from 'vue';
import { useDebounceFn } from '@vueuse/core';
import { useI18n } from 'vue-i18n';

import type {
  IItemDetail,
  IListItem,
  Permed_accounts,
  Permed_protocols
} from '@renderer/components/MainSection/interface';
import type { Ref } from 'vue';
import type { DropdownOption } from 'naive-ui';
import type { IConnectData } from '@renderer/store/interface';

import { computed } from 'vue';
import { ClipboardList, PlugConnected } from '@vicons/tabler';
import { ArrowEnterLeft20Filled, ProtocolHandler24Regular } from '@vicons/fluent';

import { storeToRefs } from 'pinia';
import { useUserStore } from '@renderer/store/module/userStore';
import { useElectronConfig } from '@renderer/hooks/useElectronConfig';

const { t } = useI18n();
const { message } = createDiscreteApi(['message']);
const { getDefaultSetting, setDefaultSetting } = useElectronConfig();

withDefaults(
  defineProps<{
    listData?: IListItem[];
  }>(),
  {
    listData: () => [] as IListItem[]
  }
);

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
const lastSelectedAccount = ref<Permed_accounts | null>(null);

const selectedItem: Ref<IListItem> = ref({} as IListItem);
const connectData: Ref<IConnectData> = ref({} as IConnectData);
const detailMessage: Ref<IItemDetail> = ref({} as IItemDetail);
const leftOptions: Ref<DropdownOption[]> = ref([
  {
    key: 'header',
    type: 'render',
    render: renderCustomHeader(ClipboardList, `${t('Common.AccountList')}`)
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
    render: renderCustomHeader(PlugConnected, `${t('Common.QuickConnect')}`, 'fast-connection')
  },
  {
    key: 'detail-message',
    type: 'render',
    render: renderCustomHeader(
      ArrowEnterLeft20Filled,
      `${t('Common.AssetDetails')}`,
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
    render: renderCustomHeader(ProtocolHandler24Regular, `${t('Common.ConnectionProtocol')}`)
  }
]);

const currentLayout = ref('');

const emit = defineEmits(['loadMore']);

const debounceLoad = useDebounceFn(() => {
  emit('loadMore');
}, 500);

/**
 * @description 重置左键菜单
 */
const resetLeftOptions = () => {
  leftOptions.value = [
    {
      key: 'header',
      type: 'render',
      render: renderCustomHeader(ClipboardList, `${t('Common.AccountList')}`)
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
      render: renderCustomHeader(PlugConnected, `${t('Common.QuickConnect')}`, 'fast-connection')
    },
    {
      key: 'detail-message',
      type: 'render',
      render: renderCustomHeader(
        ArrowEnterLeft20Filled,
        `${t('Common.AssetDetails')}`,
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
      render: renderCustomHeader(ProtocolHandler24Regular, `${t('Common.ConnectionProtocol')}`)
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
  currentLayout.value = layout;

  await setDefaultSetting({ layout });
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
    message.error(`${t('Message.ErrorGetAssetDetail')}`);
    return Promise.resolve(false);
  }
};

// 修改 selectedProtocol 的类型，使用 Map 来存储每个资产的选中协议
const selectedProtocols = ref(new Map<string, Permed_protocols>());

/**
 * @description 右键选择协议
 * @param key
 */
const handleSelect = async (key: string) => {
  if (key === 'detail-message') return;

  // 找到选中的协议
  const currentProtocol = detailMessage.value.permed_protocols.find(
    (item: Permed_protocols) => item.name === key
  );

  if (currentProtocol) {
    selectedProtocols.value.set(detailMessage.value.id, currentProtocol);

    message.success(t('Message.ProtocolSelected'));

    showRightDropdown.value = false;
  }
};

/**
 * @description 快速连接
 */
const handleFastConnect = async () => {
  if (!detailMessage.value) return;

  // 获取要使用的账号：上次选择的账号或第一个普通账号（排除 @USER 和 @INPUT）
  const accountToUse =
    (lastSelectedAccount.value?.alias !== '@USER' && lastSelectedAccount.value?.alias !== '@INPUT'
      ? lastSelectedAccount.value
      : null) ||
    detailMessage.value.permed_accounts.find(
      account =>
        account.alias !== '@ANON' && account.alias !== '@USER' && account.alias !== '@INPUT'
    );

  if (!accountToUse) return;

  // 获取要使用的协议：已选择的协议或第一个协议
  const protocolToUse =
    selectedProtocols.value.get(detailMessage.value.id) || detailMessage.value.permed_protocols[0];

  if (!protocolToUse) return;

  // 设置连接数据
  connectData.value = {
    asset: detailMessage.value.id,
    account: accountToUse.name,
    protocol: protocolToUse.name,
    input_username: accountToUse.username,
    input_secret: ''
  };

  // 由于已经排除了 @USER 和 @INPUT，这里只需要处理普通账号的情况
  if (!accountToUse.has_secret) {
    const { inputPassword, confirmed } = useAccountModal('@OTHER', t);
    watch(confirmed, async newValue => {
      if (newValue && inputPassword.value) {
        connectData.value.input_secret = inputPassword.value;
        await handleConnection(protocolToUse);
      }
    });
  } else {
    await handleConnection(protocolToUse);
  }
};

// 修改 handleAccountSelect，记录选择的账号
const handleAccountSelect = async (key: string) => {
  const currentAccount = detailMessage.value.permed_accounts.find(
    (item: Permed_accounts) => item.id === key
  );

  if (currentAccount) {
    lastSelectedAccount.value = currentAccount;
    // 重置连接数据
    connectData.value = {
      asset: '',
      account: '',
      protocol: '',
      input_username: '',
      input_secret: ''
    };

    connectData.value.asset = detailMessage.value.id;
    connectData.value.account = currentAccount.name;
    connectData.value.input_username = '';
    connectData.value.input_secret = '';

    // 获取当前资产的选中协议，如果没有则使用默认协议
    const protocol =
      selectedProtocols.value.get(detailMessage.value.id) ||
      detailMessage.value.permed_protocols[0];

    if (!protocol) {
      return;
    }

    connectData.value.protocol = protocol.name;

    const handleConnection = async () => {
      try {
        let method: string;
        switch (protocol.name) {
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

        const token = await createConnectToken(connectData.value, method);

        if (token) {
          mittBus.emit('checkMatch', connectData.value.protocol as string);
          historyStore.setHistorySession({ ...detailMessage.value });
          getLocalClientUrl(token).then(res => {
            if (res) {
              window.electron.ipcRenderer.send('open-client', res.url);
            }
          });
        }
      } catch (error: any) {
        handleConnectionError(error);
      }
    };

    switch (currentAccount.alias) {
      case '@USER':
        // 同名账号
        connectData.value.account = '@USER';
        connectData.value.input_username = currentUser.value?.username;

        if (!currentAccount.has_secret) {
          const { inputPassword, confirmed } = useAccountModal('@USER', t);
          // 等待用户点击确认按钮
          watch(confirmed, async newValue => {
            if (newValue && inputPassword.value) {
              connectData.value.input_secret = inputPassword.value;
              await handleConnection();
            }
          });
        } else {
          await handleConnection();
        }
        break;

      case '@INPUT':
        // 手动输入
        connectData.value.account = '@INPUT';
        const { inputPassword, inputUsername, confirmed } = useAccountModal('@INPUT', t);

        // 等待用户点击确认按钮
        watch(confirmed, async newValue => {
          if (newValue && inputUsername.value && inputPassword.value) {
            connectData.value.input_username = inputUsername.value;
            connectData.value.input_secret = inputPassword.value;
            await handleConnection();
          }
        });
        break;

      default:
        connectData.value.input_username = currentAccount.username;

        if (!currentAccount.has_secret) {
          const { inputPassword, confirmed } = useAccountModal('@OTHER', t);
          watch(confirmed, async newValue => {
            if (newValue && inputPassword.value) {
              connectData.value.input_secret = inputPassword.value;
              await handleConnection();
            }
          });
        } else {
          await handleConnection();
        }
    }
  }

  resetLeftOptions();
  showLeftDropdown.value = false;
};

// 抽离连接逻辑到单独的函数
const handleConnection = async (protocol: Permed_protocols) => {
  try {
    let method: string;
    switch (protocol.name) {
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

    const token = await createConnectToken(connectData.value, method);

    if (token) {
      mittBus.emit('checkMatch', connectData.value.protocol as string);
      historyStore.setHistorySession({ ...detailMessage.value });
      getLocalClientUrl(token).then(res => {
        if (res) {
          window.electron.ipcRenderer.send('open-client', res.url);
        }
      });
    }
  } catch (error: any) {
    handleConnectionError(error);
  }
};

const handleConnectionError = (error: any) => {
  const errorData = error?.response?.data;

  if (errorData) {
    // 除开通知和允许，其余情况一率弹窗
    if (errorData?.code !== 'notice') {
      // todo)) ACL 的 Action 区别加在 message 中
      message.error(`${t('Message.AssetNotice')}`);
      return;
    }

    if (errorData?.code !== 'reject') {
      message.error(`${t('Message.AssetDeny')}`);
      return;
    }
  }
};

// 在选择新的资产项时重置左侧菜单
const selectItem = useDebounceFn(async (item: IListItem, _event: MouseEvent) => {
  loadingBar.start();
  selectedItem.value = item;
  resetLeftOptions();

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
    message.error(`${t('Message.ErrorGetAssetDetail')}`);
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
  resetRightOptions();
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

onMounted(async () => {
  const { layout } = await getDefaultSetting();

  currentLayout.value = layout;

  mittBus.on('connectAsset', handleFastConnect);
  mittBus.on('changeLayout', handleLayoutChange);
});

onBeforeUnmount(() => {
  mittBus.off('connectAsset', handleFastConnect);
  mittBus.off('changeLayout', handleLayoutChange);
  selectedProtocols.value.clear(); // 清理所有存储的协议
});
</script>

<style scoped lang="scss">
@use './index.scss';
</style>
