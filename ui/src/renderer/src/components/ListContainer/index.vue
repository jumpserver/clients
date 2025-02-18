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
        @contextmenu="handleContextMenu(item, $event)"
      >
        <template #header>
          <n-flex align="center" class="w-full !flex-nowrap">
            <!-- @click="selectItem(item, $event)" @contextmenu="handleContextMenuWrapper(item, $event)" -->
            <n-icon
              depth="2"
              class="mr-1 cursor-pointer"
              :size="32"
              :component="renderedIcon(item)"
            />

            <n-flex vertical align="start" class="w-full !flex-nowrap !gap-y-0">
              <n-ellipsis style="max-width: 180px">
                <n-text depth="2" class="cursor-pointer font-mono font-light">
                  {{ item.name }}
                </n-text>
              </n-ellipsis>

              <n-flex justify="start" align="center" class="w-full !gap-x-0 !flex-nowrap">
                <!-- <n-popover trigger="hover">
                  <template #trigger>
                    <n-button text type="primary" tag="div">
                      <template #icon>
                        <BadgeCheck :size="16" />
                      </template>
                    </n-button>
                  </template>

                  <n-text depth="2"> 可连接 </n-text>
                </n-popover> -->

                <n-flex class="w-full !flex-nowrap">
                  <n-ellipsis style="max-width: 300px" class="w-1/2">
                    <n-text depth="1" class="font-normal text-xs font-mono">
                      {{ t('Common.CurrentAccount') }}:
                    </n-text>

                    <n-text depth="2" class="font-normal text-xs font-mono">
                      {{ getAssetAccount(item.id) || '-' }}
                    </n-text>
                  </n-ellipsis>

                  <n-ellipsis style="max-width: 300px" class="w-1/2">
                    <n-text depth="1" class="font-normal text-xs font-mono">
                      {{ t('Common.CurrentProtocol') }}:
                    </n-text>

                    <n-text depth="2" class="font-normal text-xs font-mono">
                      {{ getAssetProtocol(item.id) || '-' }}
                    </n-text>
                  </n-ellipsis>
                </n-flex>
              </n-flex>
            </n-flex>
          </n-flex>
        </template>

        <template #header-extra>
          <n-tooltip trigger="hover">
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
          </n-tooltip>
        </template>

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

import { Link, BadgeCheck } from 'lucide-vue-next';
import { Terminal2 } from '@vicons/tabler';
import { DataBase, Devices } from '@vicons/carbon';
import { DesktopWindowsFilled } from '@vicons/material';
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

const clicked = ref(false);
const showRightDropdown = ref(false);
const savedData = ref<Partial<IAssetDetailMessageReturn>>();

const renderedIcon = item => {
  switch (item.type?.value) {
    case 'linux':
      return Terminal2;
    case 'general':
      return Devices;
    case 'windows':
      return DesktopWindowsFilled;
    default:
      return DataBase;
  }
};

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

/**
 * @description 处理右键菜单
 * @param item
 * @param event
 */
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

//! todo
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

/**
 * @description 处理快速连接
 * @param item
 * @param event
 */
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

<style scoped></style>
