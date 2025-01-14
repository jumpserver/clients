<template>
  <n-list-item class="h-20 group">
    <n-flex class="h-full w-full" align="center" justify="space-between">
      <n-flex
        align="center"
        :class="layout === 'grid' ? 'inner-grid' : ''"
        class="w-full !flex-nowrap"
      >
        <div class="icon-zone flex items-center">
          <n-icon :component="renderedIcon" size="35" />
        </div>
        <n-flex
          vertical
          align="flex-start"
          justify="center"
          class="description-zone w-full !flex-nowrap"
        >
          <template v-if="layout !== 'grid'">
            <n-descriptions
              label-placement="left"
              class="w-full"
              :column="1"
              :separator="''"
              size="small"
            >
              <n-descriptions-item content-style="width: 5rem; text-wrap: nowrap;">
                <template #label>
                  <n-tag
                    round
                    type="success"
                    size="small"
                    class="tracking-widest"
                    :bordered="false"
                  >
                    {{ t('Common.AssetName') }}
                  </n-tag>
                </template>

                <n-popover>
                  <template #trigger>
                    <span class="ml-2">
                      {{ itemData.name }}
                    </span>
                  </template>
                  <n-text depth="1">
                    {{ itemData.name }}
                  </n-text>
                </n-popover>
              </n-descriptions-item>
              <n-descriptions-item content-style="width: 5rem">
                <template #label>
                  <n-tag round type="info" size="small" class="tracking-widest" :bordered="false">
                    {{ t('Common.Organization') }}
                  </n-tag>
                </template>
                <n-popover>
                  <template #trigger>
                    <span class="ml-2">
                      {{ itemData.org_name }}
                    </span>
                  </template>
                  <n-text depth="1">
                    {{ itemData.org_name }}
                  </n-text>
                </n-popover>
              </n-descriptions-item>
            </n-descriptions>
          </template>

          <template v-else>
            <n-ellipsis :style="{ maxWidth: layout === 'grid' ? '110px' : '' }">
              {{ itemData.name }}
              <template #tooltip>
                {{ t('Message.Organization') }}: {{ itemData.name }}
              </template>
            </n-ellipsis>
          </template>
        </n-flex>
        <div></div>
      </n-flex>
    </n-flex>
  </n-list-item>
</template>

<script setup lang="ts">
import { Terminal2 } from '@vicons/tabler';
import { DataBase, Devices } from '@vicons/carbon';
import { DesktopWindowsFilled } from '@vicons/material';

import { useI18n } from 'vue-i18n';
import { watch, shallowRef } from 'vue';

import type { IListItem } from '@renderer/components/MainSection/interface';

const props = withDefaults(defineProps<{ layout: string; itemData: IListItem }>(), {
  layout: '',
  itemData: () => ({}) as IListItem
});

const { t } = useI18n();
const renderedIcon = shallowRef();

watch(
  () => props.itemData,
  newValue => {
    switch (newValue.type.value) {
      case 'linux':
        renderedIcon.value = Terminal2;
        break;
      case 'windows':
        renderedIcon.value = DesktopWindowsFilled;
        break;
      case 'general':
        renderedIcon.value = Devices;
        break;
      default:
        renderedIcon.value = DataBase;
    }
  },
  { immediate: true }
);
</script>
