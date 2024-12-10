<template>
  <n-list-item class="h-[80px] group">
    <n-flex class="h-full w-full" align="center" justify="space-between">
      <n-flex align="center" :class="layout === 'grid' ? 'inner-grid' : ''">
        <div class="icon-zone flex items-center">
          <n-icon :component="renderedIcon" size="35" />
        </div>
        <n-flex vertical align="flex-start" justify="center" class="description-zone">
          <template v-if="layout !== 'grid'">
            <n-tag :bordered="false" type="success" round size="small" style="letter-spacing: 1px">
              <template #icon>
                <n-icon :component="ComputerFilled" size="12" />
              </template>
              资产名称: {{ itemData.name }}
            </n-tag>
            <n-tag :bordered="false" type="info" round size="small" style="letter-spacing: 1px">
              <template #icon>
                <n-icon :component="Link" size="12" />
              </template>
              资产地址: {{ itemData.address }}
            </n-tag>
          </template>

          <template v-else>
            <n-ellipsis :style="{ maxWidth: layout === 'grid' ? '110px' : '' }">
              <n-popover>
                <template #trigger>
                  {{ itemData.name }}
                </template>
                资产名称: {{ itemData.name }}
              </n-popover>
            </n-ellipsis>
            <n-ellipsis :style="{ maxWidth: layout === 'grid' ? '110px' : '' }">
              {{ itemData.address }}
              <template #tooltip>
                <div style="text-align: center">资产地址: {{ itemData.address }}</div>
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
import { Link } from '@vicons/fa';
import { DataBase } from '@vicons/carbon';
import { Terminal2 } from '@vicons/tabler';
import { DesktopWindowsFilled, ComputerFilled } from '@vicons/material';
import { watch, shallowRef } from 'vue';

const props = withDefaults(defineProps<{ layout: string; itemData: any }>(), {
  layout: '',
  itemData: {}
});

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
      default:
        renderedIcon.value = DataBase;
    }
  },
  { immediate: true }
);
</script>
