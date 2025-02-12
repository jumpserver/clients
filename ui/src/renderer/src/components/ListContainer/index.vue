<template>
  <n-grid
    v-if="listData.length > 0"
    :x-gap="12"
    :y-gap="12"
    :item-responsive="true"
    :cols="currentLayout === 'grid' ? 3 : 1"
    class="h-full px-4"
    responsive="screen"
  >
    <n-gi v-for="item of listData" :key="item.id">
      <n-card
        hoverable
        size="small"
        class="h-32 rounded-lg cursor-pointer"
        @contextmenu="handleContextMenu(item, $event)"
      >
        <template #header>
          <n-flex align="center" class="w-full !flex-nowrap">
            <!-- @click="selectItem(item, $event)" @contextmenu="handleContextMenuWrapper(item, $event)" -->
            <n-icon
              depth="2"
              class="mr-1 cursor-pointer"
              :size="26"
              :component="renderedIcon(item)"
            />

            <n-ellipsis style="max-width: 180px">
              <n-text depth="2" class="cursor-pointer font-mono font-light text-sm">
                {{ item.name }}
              </n-text>
            </n-ellipsis>
          </n-flex>
        </template>

        <template #header-extra>
          <n-tooltip trigger="hover">
            <template #trigger>
              <Link :size="16" class="outline-none icon-hover" />
              <!-- @click.stop.prevent="handleIconConnect(item, $event)" -->
            </template>

            <n-text depth="2"> 快速连接 </n-text>
          </n-tooltip>
        </template>
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
import { ref } from 'vue';
import { useI18n } from 'vue-i18n';
import { useMessage } from 'naive-ui';
import { useContextMenu } from '@renderer/hooks/useContextMenu';

import { Link } from 'lucide-vue-next';
import { Terminal2 } from '@vicons/tabler';
import { DataBase, Devices } from '@vicons/carbon';
import { DesktopWindowsFilled } from '@vicons/material';

import type { IListItem } from '@renderer/components/MainSection/interface';

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

const rightX = ref(0);
const rightY = ref(0);
const showRightDropdown = ref(false);

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

/**
 * @description 处理右键菜单
 * @param item
 * @param event
 */
const handleContextMenu = async (item: IListItem, event: MouseEvent) => {
  const got = await getAssetDetailMessage(item, event);

  if (!got) {
    showRightDropdown.value = false;
    message.error(`${t('Message.ErrorGetAssetDetail')}`);

    return;
  }

  showRightDropdown.value = true;
  rightX.value = event.clientX;
  rightY.value = event.clientY;
};
</script>

<style scoped></style>
