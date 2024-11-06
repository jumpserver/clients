<template>
  <n-list-item class="h-[60px] group">
    <n-flex class="h-full w-full" align="center" justify="space-between">
      <n-flex align="center" :class="layout === 'grid' ? 'inner-grid' : ''">
        <div class="icon-zone flex items-center">
          <n-icon :component="renderedIcon" size="30" />
        </div>
        <div class="description-zone">
          <div class="asset-name">{{ itemData.name }}</div>
        </div>
      </n-flex>
    </n-flex>
  </n-list-item>
</template>

<script setup lang="ts">
import { DataBase } from '@vicons/carbon';
import { Terminal2 } from '@vicons/tabler';
import { DesktopWindowsFilled } from '@vicons/material';
import { watch, shallowRef } from 'vue';

const props = withDefaults(
  defineProps<{ layout: string; itemData: any; generalIconName: string }>(),
  {
    layout: '',
    itemData: {},
    generalIconName: ''
  }
);

const renderedIcon = shallowRef();

watch(
  () => props.generalIconName,
  newValue => {
    switch (newValue) {
      case 'terminal':
        renderedIcon.value = Terminal2;
        break;
      case 'windows':
        renderedIcon.value = DesktopWindowsFilled;
        break;
      case 'database':
        renderedIcon.value = DataBase;
        break;
    }
  },
  { immediate: true }
);
</script>
