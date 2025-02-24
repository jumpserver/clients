<template>
  <n-grid
    v-if="listData.length > 0"
    :x-gap="12"
    :y-gap="12"
    :item-responsive="true"
    :cols="currentLayout === 'grid' ? 2 : 1"
    class="h-full px-4"
    responsive="screen"
  >
    <n-gi v-for="item of listData" :key="item.id">
      <n-card
        hoverable
        size="small"
        class="h-36 rounded-lg cursor-pointer"
        :content-style="{
          display: 'flex',
          flexDirection: 'column',
          alignItems: 'center'
        }"
        @contextmenu="handleContextMenu(item, $event)"
      >
        <template #header>
          <n-flex vertical align="start" class="w-full !gap-y-2">
            <n-text depth="2" class="cursor-pointer font-mono font-normal text-sm">
              {{ item.name }}
            </n-text>

            <n-flex justify="start" align="center" class="w-full">
              <n-flex class="w-full">
                <n-popover trigger="hover" placement="top">
                  <template #trigger>
                    <n-tag round size="small" type="info" :bordered="false" class="cursor-pointer">
                      <span class="font-normal tracking-wider">
                        {{ t('Common.CurrentAccount') }}:
                      </span>

                      <span class="font-normal">
                        {{ getAssetAccount(item.id) || '-' }}
                      </span>
                    </n-tag>
                  </template>

                  {{ t('Common.CurrentAccount') }}: {{ getAssetAccount(item.id) || '-' }}
                </n-popover>

                <n-popover trigger="hover" placement="top">
                  <template #trigger>
                    <n-tag round size="small" type="info" :bordered="false" class="cursor-pointer">
                      <span class="font-normal tracking-wider">
                        {{ t('Common.CurrentProtocol') }}:
                      </span>
                      <span class="font-normal">
                        {{ getAssetProtocol(item.id) || '-' }}
                      </span>
                    </n-tag>
                  </template>

                  {{ t('Common.CurrentProtocol') }}: {{ getAssetProtocol(item.id) || '-' }}
                </n-popover>

                <n-tag size="small" :bordered="false" type="success" round>
                  可连接
                  <template #icon>
                    <n-icon :component="CheckmarkCircle" />
                  </template>
                </n-tag>
              </n-flex>
            </n-flex>
          </n-flex>
        </template>

        <template #header-extra>
          <n-popover trigger="hover">
            <template #trigger>
              <Link
                :size="16"
                class="outline-none icon-hover"
                @click="handleConnect(item, $event)"
              />
            </template>

            <span>
              {{ t('Common.QuickConnect') }}
            </span>
          </n-popover>
        </template>

        <n-divider class="!m-0" />

        <n-grid :cols="1" class="h-full">
          <n-gi>
            <n-flex align="center" justify="start" class="h-full w-full">
              <n-text depth="1" class="font-normal">{{ t('Common.Address') }}: </n-text>

              <n-ellipsis style="max-width: 240px">
                <n-text depth="2"> {{ item.address }} </n-text>
              </n-ellipsis>
            </n-flex>
          </n-gi>

          <n-gi>
            <n-flex align="center" justify="start" class="h-full w">
              <n-text depth="1" class="font-normal"> {{ t('Common.IsActivated') }}: </n-text>

              <n-text depth="2">
                {{ item.is_active }}
              </n-text>
            </n-flex>
          </n-gi>
        </n-grid>
      </n-card>
    </n-gi>
  </n-grid>

  <n-dropdown
    trigger="manual"
    size="small"
    :x="rightX"
    :y="rightY"
    :show-arrow="true"
    :options="rightOptions"
    :show="showRightDropdown"
    :on-clickoutside="onClickRightOutside"
    @select="handleOptionSelect"
  />
</template>

<script setup lang="ts">
import mittBus from '@renderer/eventBus';
import { ref, watch } from 'vue';
import { useI18n } from 'vue-i18n';
import { useMessage } from 'naive-ui';
import { useContextMenu } from '@renderer/hooks/useContextMenu';
import { useAssetStore } from '@renderer/store/module/assetStore';
import { useHistoryStore } from '@renderer/store/module/historyStore';
import { useAccountModal } from '@renderer/components/MainSection/helper';

import { Link } from 'lucide-vue-next';
import { CheckmarkCircle } from '@vicons/ionicons5';
import { createConnectToken, getLocalClientUrl } from '@renderer/api/modals/asset';

import type { IListItem } from '@renderer/components/MainSection/interface';
import type { IAssetDetailMessageReturn } from '@renderer/hooks/useContextMenu';

const props = withDefaults(
  defineProps<{
    currentLayout: string;
    listData: IListItem[];
  }>(),
  {
    currentLayout: 'list',
    listData: () => []
  }
);

const { t } = useI18n();
const { rightOptions, getAssetDetailMessage, handleOptionSelect } = useContextMenu();

const message = useMessage();
const assetStore = useAssetStore();
const historyStore = useHistoryStore();

const rightX = ref(0);
const rightY = ref(0);

const showRightDropdown = ref(false);
const savedData = ref<Partial<IAssetDetailMessageReturn>>();

const onClickRightOutside = () => {
  showRightDropdown.value = false;
};

const getAssetAccount = (assetId: string) => {
  const assetInfo = assetStore.getAssetMap(assetId);
  return assetInfo?.account?.label;
};

const getAssetProtocol = (assetId: string) => {
  const assetInfo = assetStore.getAssetMap(assetId);
  return assetInfo?.protocol?.label;
};

const handleConnectionError = (error: any) => {
  const errorData = error?.response?.data;

  if (errorData) {
    // 除开通知和允许，其余情况一率弹窗
    if (errorData?.code !== 'notice') {
      message.error(`${t('Message.AssetNotice')}`);
      return;
    }

    if (errorData?.code !== 'reject') {
      message.error(`${t('Message.AssetDeny')}`);
      return;
    }
  }
};

const handleContextMenu = async (item: IListItem, event: MouseEvent) => {
  const result = await getAssetDetailMessage(item, event, false);

  if (!result || typeof result === 'boolean') {
    showRightDropdown.value = false;
    message.error(`${t('Message.ErrorGetAssetDetail')}`);
    return;
  }

  // 用于处理用户右键点击，然后去点击外层的 icon 触发连接的逻辑
  const { id, detailMessage, connectionData } = result;

  savedData.value = {
    id,
    detailMessage,
    connectionData
  };

  showRightDropdown.value = true;
  rightX.value = event.clientX;
  rightY.value = event.clientY;
};

const connectionDispatch = async (id, detailMessage, connectionData) => {
  let method: string;
  const neededInput = assetStore.getAssetMap(id)?.account?.has_secret;

  if (!neededInput) {
    const { inputPassword, confirmed } = useAccountModal('@OTHER', t);

    watch(confirmed, async newValue => {
      // prettier-ignore
      if (newValue && inputPassword.value) connectionData.value.input_secret = inputPassword.value;
    });
  }

  switch (connectionData.value.protocol) {
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

  const token = await createConnectToken(connectionData.value, method);

  if (token) {
    mittBus.emit('checkMatch', connectionData.value.protocol as string);
    historyStore.setHistorySession({ ...detailMessage.value });

    getLocalClientUrl(token).then(res => {
      if (res) {
        window.electron.ipcRenderer.send('open-client', res.url);
      }
    });
  }
};

const handleConnect = async (item: IListItem, event: MouseEvent) => {
  try {
    const result = await getAssetDetailMessage(item, event, true);

    if (!result || typeof result === 'boolean') {
      message.error(`${t('Message.ErrorGetAssetDetail')}`);
      return;
    }

    const { id, detailMessage, connectionData } = result;

    await connectionDispatch(id, detailMessage, connectionData);
  } catch (error) {
    handleConnectionError(error);
  }
};
</script>
