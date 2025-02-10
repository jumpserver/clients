<template>
  <n-grid
    :cols="3"
    x-gap="12"
    class="px-5 py-5 border-b border-primary bg-primary"
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
          v-model:value="currentOrginization"
          @update:value="handleChangeOrginization"
        />

        <RightIconZone />
      </n-flex>
    </n-grid-item>
  </n-grid>
</template>

<script setup lang="ts">
import { useI18n } from 'vue-i18n';
import { useMessage } from 'naive-ui';
import { Search } from 'lucide-vue-next';
import { ref, computed, onMounted, nextTick } from 'vue';
import { RightIconZone } from './helper/renderer';
import { getOrginization } from '@renderer/api/modals/user';
import { useUserStore } from '@renderer/store/module/userStore';

import mittBus from '@renderer/eventBus';

defineProps<{
  active: boolean;
}>();

const { t } = useI18n();
const message = useMessage();
const userStore = useUserStore();

const searchInput = ref('');
const currentOrginization = ref('');

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

onMounted(async () => {
  try {
    const res = await getOrginization();

    if (res) {
      currentOrginization.value = res?.id;
      userStore.setCurrentOrginization(res?.id);
    }
  } catch (e) {
    message.error(t('Message.GetOrginizationFailed'));
  }
});
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
