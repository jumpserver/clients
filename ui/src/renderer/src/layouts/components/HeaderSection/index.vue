<template>
  <n-flex
    justify="center"
    align="center"
    vertical
    class="px-[20px] pt-[20px] pb-[5px] border-b border-primary bg-primary"
    :class="active ? 'show-drawer' : ''"
  >
    <n-input-group>
      <n-input
        clearable
        class="rounded-xl"
        placeholder="Search for assets"
        v-model:value="searchInput"
        @keypress.native.enter="onKeyEnter"
      />
      <n-button type="primary" round secondary @click="handleSearch">
        <template #icon>
          <Search :size="16" />
        </template>
        {{ t('Common.Search') }}
      </n-button>
    </n-input-group>

    <n-flex justify="space-between" align="center" class="w-full h-9">
      <n-flex class="h-full w-60 rounded-xl" align="center">
        <n-select size="small" v-model:value="value" :options="options" />
      </n-flex>

      <!-- 右侧 icon  -->
      <n-flex class="h-full" align="center">
        <RightIconZone />
      </n-flex>
    </n-flex>
  </n-flex>
</template>

<script setup lang="ts">
import { RightIconZone } from './helper/renderer';

import { ref } from 'vue';
import { useI18n } from 'vue-i18n';
import { Search } from 'lucide-vue-next';

import mittBus from '@renderer/eventBus';

defineProps<{
  active: boolean;
}>();

const { t } = useI18n();

const searchInput = ref('');

const value = ref('');

const options = ref([
  {
    label: "Everybody's Got Something to Hide Except Me and My Monkey",
    value: 'song0',
    disabled: true
  },
  {
    label: 'Drive My Car',
    value: 'song1'
  },
  {
    label: 'Norwegian Wood',
    value: 'song2'
  },
  {
    label: "You Won't See",
    value: 'song3',
    disabled: true
  },
  {
    label: 'Nowhere Man',
    value: 'song4'
  },
  {
    label: 'Think For Yourself',
    value: 'song5'
  },
  {
    label: 'The Word',
    value: 'song6'
  },
  {
    label: 'Michelle',
    value: 'song7',
    disabled: true
  },
  {
    label: 'What goes on',
    value: 'song8'
  },
  {
    label: 'Girl',
    value: 'song9'
  },
  {
    label: "I'm looking through you",
    value: 'song10'
  },
  {
    label: 'In My Life',
    value: 'song11'
  },
  {
    label: 'Wait',
    value: 'song12'
  }
]);

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

.n-base-select-menu {
  display: none !important;
}

::v-deep(.n-popselect-menu .n-base-select-option) {
  padding: 0 1rem !important;
}
</style>
