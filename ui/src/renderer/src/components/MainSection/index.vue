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

import {createConnectToken, getAssetDetail, getLocalClientUrl} from '@renderer/api/modals/asset';
import {moveElementToEnd, renderCustomHeader} from '@renderer/components/MainSection/helper';
import {useHistoryStore} from '@renderer/store/module/historyStore';
import type {Ref} from 'vue';
import {onBeforeUnmount, onMounted, ref} from 'vue';
import type {DropdownOption} from 'naive-ui';
import {createDiscreteApi} from 'naive-ui';

import {Conf} from 'electron-conf/renderer';
import {IItemDetail, IListItem, Permed_accounts, Permed_protocols} from '@renderer/components/MainSection/interface';
import type {IConnectData} from '@renderer/store/interface';

import {ClipboardList, PlugConnected} from '@vicons/tabler';
import {ArrowEnterLeft20Filled, ProtocolHandler24Regular} from '@vicons/fluent';

const {message} = createDiscreteApi(['message']);

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
}
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
}
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
    message.error('获取资产数据列表失败');
    return Promise.resolve(false);
  }
};

/**
 * @description listItem 的点击事件
 * @param item
 * @param _event
 */
const selectItem = async (item: IListItem, _event: MouseEvent) => {
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
            label: item.name +
              (option.alias === option.username || option.alias.startsWith('@')
                ? ''
                : '(' + option.username + ')')
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
 * @description contextmenu 的回调
 * @param _item
 * @param _event
 */
const handleItemContextMenu = async (_item: IListItem, _event: MouseEvent) => {
  _event.stopPropagation();

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

      showRightDropdown.value = true;
      xRight.value = _event.clientX;
      yRight.value = _event.clientY;
    }
  } catch (e) {
    showRightDropdown.value = false;
  }
};

/**
 * @description 左键选择账号
 * @param key
 */
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
    connectData.value.asset = detailMessage.value.id;

    connectData.value.account = currentAccount.name;
  }

  showLeftDropdown.value = false;
};

/**
 * @description 右键选择协议
 * @param key
 */
const handleSelect = async (key: string) => {
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
        connectData.value.protocol = currentProtocol.name
        const token = await createConnectToken(connectData.value, method);

        if (token) {
          message.success('连接成功', {closable: true});

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
  showRightDropdown.value = false;
};

/**
 * @description 快速连接
 */
const handleFastConnect = async () => {
  await handleSelect('fast-connection')
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
