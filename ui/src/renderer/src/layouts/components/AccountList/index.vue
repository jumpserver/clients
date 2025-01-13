<template>
  <n-flex
    align="center"
    justify="start"
    class="w-full !flex-nowrap cursor-pointer"
    @click="changeAccount"
  >
    <n-avatar round class="flex-shrink-0" :src="userAvator">
      <n-icon v-if="typeof userAvator !== 'string'" :component="userAvator" />
    </n-avatar>

    <n-flex class="flex-1 min-w-0">
      <n-popover :delay="1000" style="width: 20rem" placement="right">
        <template #trigger>
          <n-flex align="center" justify="start" class="w-full !flex-nowrap">
            <n-text depth="1" class="cursor-pointer truncate">
              {{ username }}
            </n-text>

            <Check
              :size="18"
              :color="userToken === userStore.currentUser!.token ? '#63e2b7' : 'gray'"
              class="cursor-pointer flex-shrink-0"
            />
          </n-flex>
        </template>

        <n-tag :bordered="false" size="small" type="info">
          {{ t('Common.DataSource') }}
        </n-tag>
        : {{ userSite }}
      </n-popover>
    </n-flex>
  </n-flex>
</template>

<script setup lang="ts">
import { NIcon } from 'naive-ui';
import { useI18n } from 'vue-i18n';
import { Check } from 'lucide-vue-next';
import { UserCircleRegular } from '@vicons/fa';
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
    userAvator: UserCircleRegular,

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
