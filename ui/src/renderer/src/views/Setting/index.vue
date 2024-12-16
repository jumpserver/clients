<template>
  <n-flex justify="center" align="center" class="w-full h-full px-4 py-4">
    <n-scrollbar>
      <n-flex class="w-full h-full" align="center">
        <n-card
          v-for="item in 1"
          :key="item"
          :bordered="false"
          size="small"
          header-style="font-size: 13px;"
          class="rounded-xl !bg-secondary"
        >
          <n-collapse :default-expanded-names="enabledItems">
            <n-collapse-item :name="1" title="SSH">
              <n-card
                v-for="item in 8"
                :key="item"
                title="卡片"
                class="rounded-xl !bg-secondary w-1/3 my-2 mx-1"
              >
                卡片内容
              </n-card>
            </n-collapse-item>
          </n-collapse>
        </n-card>
      </n-flex>
    </n-scrollbar>
  </n-flex>

  <!--  <template #header-extra>-->
  <!--    <n-switch-->
  <!--      size="small"-->
  <!--      v-model:value="is_set"-->
  <!--      @update:value="handleSwitchValueChange"-->
  <!--    >-->
  <!--      <template #checked> 已启用</template>-->
  <!--      <template #unchecked> 未启用</template>-->
  <!--    </n-switch>-->
  <!--  </template>-->
</template>

<script setup lang="ts">
import { ref } from 'vue';

import { computed } from 'vue';

// const is_set = ref('');

// const mockData = [{}];

const enabledItems = computed(() => {});

// const handleSwitchValueChange = () => {};
</script>

<style scoped lang="scss">
:deep(.n-collapse-item__content-inner) {
  display: flex;
  flex-wrap: wrap;
}
</style>
