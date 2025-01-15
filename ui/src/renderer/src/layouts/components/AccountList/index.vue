<template>
  <n-flex
    align="center"
    justify="start"
    class="w-full !flex-nowrap cursor-pointer group"
    @click="changeAccount"
  >
    <n-avatar round class="flex-shrink-0" :src="userAvator" />

    <n-flex align="center" justify="start" class="w-full !flex-nowrap">
      <n-text depth="1" class="cursor-pointer truncate">
        {{ username }}
      </n-text>

      <n-popover :delay="500" style="width: 20rem" placement="right">
        <template #trigger>
          <Eye
            :size="18"
            color="gray"
            class="cursor-pointer flex-shrink-0 opacity-0 group-hover:opacity-100 transition-opacity duration-300"
          />
        </template>

        <n-tag :bordered="false" size="small" type="info">
          {{ t('Common.DataSource') }}
        </n-tag>
        : {{ userSite }}
      </n-popover>

      <Check
        :size="18"
        :color="userToken === userStore.currentUser!.token ? '#63e2b7' : 'gray'"
        class="cursor-pointer flex-shrink-0"
      />
    </n-flex>
  </n-flex>
</template>

<script setup lang="ts">
import { useI18n } from 'vue-i18n';
import { Check, Eye } from 'lucide-vue-next';
import { useUserStore } from '@renderer/store/module/userStore';

import type { Component } from 'vue';

const props = withDefaults(
  defineProps<{
    userAvator: string | Component;
    username: string;
    userSite: string;
    userToken: string;
  }>(),
  {
    userAvator: '',
    username: '-',
    userSite: '-',
    userToken: ''
  }
);

const emits = defineEmits<{
  (e: 'changeAccount', token: string): void;
}>();

const { t } = useI18n();
const userStore = useUserStore();

const changeAccount = () => {
  emits('changeAccount', props.userToken);
};
</script>

<style scoped>
:deep(.n-text) {
  width: 100%;
}
</style>
