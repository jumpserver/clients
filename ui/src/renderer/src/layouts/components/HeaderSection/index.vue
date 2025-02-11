<template>
  <n-grid
    :cols="3"
    x-gap="12"
    class="px-[1.6rem] py-[1.6rem] border-b-2 border-primary bg-primary"
    :class="active ? 'show-drawer' : ''"
  >
    <n-grid-item :span="2">
      <n-input-group>
        <n-input
          round
          clearable
          placeholder="Search for assets"
          v-model:value="searchInput"
          @keypress.native.enter="onKeyEnter"
        />
        <n-button type="primary" secondary round @click="handleSearch">
          <template #icon>
            <Search :size="16" />
          </template>
          {{ t('Common.Search') }}
        </n-button>
      </n-input-group>
    </n-grid-item>

    <n-grid-item :span="1">
      <n-flex class="h-full !flex-nowrap" justify="space-around" align="center">
        <n-select
          class="w-1/2 !rounded-4xl"
          :options="orginizationList"
          v-model:value="userStore.currentOrginization"
          @update:value="handleChangeOrginization"
        />

        <RightIconZone />
      </n-flex>
    </n-grid-item>
  </n-grid>
</template>

<script setup lang="ts">
import { useI18n } from 'vue-i18n';
import { Search } from 'lucide-vue-next';
import { ref, computed, nextTick } from 'vue';
import { RightIconZone } from './helper/renderer';
import { useUserStore } from '@renderer/store/module/userStore';

import mittBus from '@renderer/eventBus';

defineProps<{
  active: boolean;
}>();

const { t } = useI18n();
const userStore = useUserStore();

const searchInput = ref('');

const orginizationList = computed(() => {
  return userStore.orginization?.map(item => ({
    label: item.name,
    value: item.id
  }));
});

const handleChangeOrginization = (value: string) => {
  userStore.setCurrentOrginization(value);

  nextTick(() => {
    mittBus.emit('search', '');
  });
};

/**
 * @description 搜索
 */
const handleSearch = () => {
  mittBus.emit('search', searchInput.value);
};

const onKeyEnter = (event: KeyboardEvent) => {
  if (!event.shiftKey || event.ctrlKey) {
    event.preventDefault();
    mittBus.emit('search', searchInput.value);
  }
};
</script>

<style scoped lang="scss">
@use './index.scss';

::v-deep(.n-base-selection) {
  border-radius: 2.125rem;
}

::v-deep(.n-popselect-menu .n-base-select-option) {
  padding: 0 1rem !important;
}
</style>
