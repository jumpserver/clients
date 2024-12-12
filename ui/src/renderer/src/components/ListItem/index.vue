<template>
  <!--  <n-popover :show-arrow="true" :delay="1000" style="width: 15rem">-->
  <!--    <template #trigger>-->
  <!--      -->
  <!--    </template>-->
  <!--    <n-descriptions label-placement="left" title="更多信息" :columns="1">-->
  <!--      <n-descriptions-item>-->
  <!--        <template #label>-->
  <!--          <n-text depth="1" strong> 资产名称 </n-text>-->
  <!--        </template>-->
  <!--        {{ itemData.name }}-->
  <!--      </n-descriptions-item>-->
  <!--      <n-descriptions-item>-->
  <!--        <template #label>-->
  <!--          <n-text depth="1" strong> 资产地址 </n-text>-->
  <!--        </template>-->
  <!--        {{ itemData.address }}-->
  <!--      </n-descriptions-item>-->
  <!--      <n-descriptions-item>-->
  <!--        <template #label>-->
  <!--          <n-text depth="1" strong> 归属组织 </n-text>-->
  <!--        </template>-->
  <!--        {{ itemData.org_name }}-->
  <!--      </n-descriptions-item>-->
  <!--      <n-descriptions-item>-->
  <!--        <template #label>-->
  <!--          <n-text depth="1" strong> 连接性 </n-text>-->
  <!--        </template>-->
  <!--        {{ itemData.connectivity.label }}-->
  <!--      </n-descriptions-item>-->
  <!--      <n-descriptions-item>-->
  <!--        <template #label>-->
  <!--          <n-text depth="1" strong> 平台 </n-text>-->
  <!--        </template>-->
  <!--        {{ itemData.platform.name }}-->
  <!--      </n-descriptions-item>-->
  <!--      <n-descriptions-item>-->
  <!--        <template #label>-->
  <!--          <n-text depth="1" strong> 创建者 </n-text>-->
  <!--        </template>-->
  <!--        {{ itemData.created_by }}-->
  <!--      </n-descriptions-item>-->
  <!--    </n-descriptions>-->
  <!--  </n-popover>-->

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
            <n-descriptions label-placement="left" class="w-full" :column="1" size="small">
              <n-descriptions-item content-style="width: 5rem; text-wrap: nowrap;">
                <template #label>
                  <n-tag
                    type="success"
                    round
                    size="small"
                    class="font-bold tracking-wide"
                    :bordered="false"
                  >
                    资产名称
                  </n-tag>
                </template>

                <n-popover>
                  <template #trigger>
                    {{ itemData.name }}
                  </template>
                  <n-text depth="1">
                    {{ itemData.name }}
                  </n-text>
                </n-popover>
              </n-descriptions-item>
              <!-- 暂时舍弃 -->
              <!--              <n-descriptions-item content-style="width: 5rem" :span="2">-->
              <!--                <template #label>-->
              <!--                  <n-tag-->
              <!--                    type="info"-->
              <!--                    round-->
              <!--                    size="small"-->
              <!--                    class="font-bold tracking-wide"-->
              <!--                    :bordered="false"-->
              <!--                  >-->
              <!--                    资产地址-->
              <!--                  </n-tag>-->
              <!--                </template>-->

              <!--                <n-popover>-->
              <!--                  <template #trigger>-->
              <!--                    {{ itemData.address }}-->
              <!--                  </template>-->
              <!--                  <n-text depth="1">-->
              <!--                    {{ itemData.address }}-->
              <!--                  </n-text>-->
              <!--                </n-popover>-->
              <!--              </n-descriptions-item>-->
              <n-descriptions-item content-style="width: 5rem">
                <template #label>
                  <n-tag
                    type="info"
                    round
                    size="small"
                    class="font-bold tracking-wider"
                    :bordered="false"
                  >
                    归属组织
                  </n-tag>
                </template>
                <n-popover>
                  <template #trigger>
                    {{ itemData.org_name }}
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
              <n-popover>
                <template #trigger>
                  {{ itemData.name }}
                </template>
                资产名称: {{ itemData.name }}
              </n-popover>
            </n-ellipsis>
            <!-- 暂时舍弃 -->
            <!--            <n-ellipsis :style="{ maxWidth: layout === 'grid' ? '110px' : '' }">-->
            <!--              {{ itemData.address }}-->
            <!--              <template #tooltip>-->
            <!--                <div style="text-align: center">资产地址: {{ itemData.address }}</div>-->
            <!--              </template>-->
            <!--            </n-ellipsis>-->
          </template>
        </n-flex>
        <div></div>
      </n-flex>
    </n-flex>
  </n-list-item>
</template>

<script setup lang="ts">
import { DataBase } from '@vicons/carbon';
import { Terminal2 } from '@vicons/tabler';
import { DesktopWindowsFilled } from '@vicons/material';

import { watch, shallowRef } from 'vue';

import type { IListItem } from '@renderer/components/MainSection/interface';
import { NIcon } from 'naive-ui';

const props = withDefaults(defineProps<{ layout: string; itemData: IListItem }>(), {
  layout: '',
  itemData: () => ({}) as IListItem
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
