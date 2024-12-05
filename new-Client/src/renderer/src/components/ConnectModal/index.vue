<template>
  <n-modal
    :show="true"
    :mask-closable="false"
    :show-icon="false"
    preset="dialog"
    class="rounded-[10px]"
    @close="handleCloseClick"
    @mask-click="handleMaskClick"
  >
    <template #header>
      <n-flex align="center">
        <n-text depth="1">登录</n-text>
      </n-flex>
    </template>

    <template #default>
      <n-flex vertical justify="space-evenly" align="flex-start" class="w-full h-[70px]">
        <n-input
          v-model:value="siteLocation"
          clearable
          placeholder="请输入 IP 地址或域名作为登录站点"
          class="rounded-[10px]"
        />
      </n-flex>
    </template>
    <template #action>
      <n-button round :disabled="!siteLocation" size="small" type="primary" @click="jumpToLogin">
        登录
      </n-button>
    </template>
  </n-modal>
</template>

<script setup lang="ts">
import { ref } from 'vue';
import { useMessage } from 'naive-ui';
import { Add24Regular } from '@vicons/fluent';
import { useUserStore } from '@renderer/store/module/userStore';

withDefaults(
  defineProps<{
    showModal: boolean;
  }>(),
  { showModal: false }
);

const emit = defineEmits(['CloseClick']);

const message = useMessage();
const userStore = useUserStore();

const siteLocation = ref('');

const handleMaskClick = (): void => {
  message.error('请输入站点地址', { closable: true });
};

const handleCloseClick = (): void => {
  emit('CloseClick');
};
</script>

<style scoped lang="scss"></style>
