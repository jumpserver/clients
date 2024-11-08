<template>
  <n-modal
    :show="showModal"
    :mask-closable="false"
    :show-icon="false"
    :closable="false"
    preset="dialog"
    class="rounded-[10px]"
    @mask-click="handleMaskClick"
  >
    <template #header>
      <n-flex align="center">
        <n-icon size="30" :component="Warning24Regular" color="#4B9E5F" />
        <n-text depth="1">提示</n-text>
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
import { Warning24Regular } from '@vicons/fluent';
import { useUserStore } from '@renderer/store/module/userStore';

withDefaults(
  defineProps<{
    showModal: boolean;
  }>(),
  { showModal: false }
);

const urlRegex =
  /^(https?:\/\/)?((([a-zA-Z0-9-]{1,63}\.)+[a-zA-Z]{2,})|((\d{1,3}\.){3}\d{1,3}(?!\d))|(\[?([a-fA-F0-9]{1,4}:){1,7}[a-fA-F0-9]{1,4}\]?))$/;

const message = useMessage();
const userStore = useUserStore();

const siteLocation = ref('');

const handleMaskClick = (): void => {
  message.error('请输入站点地址', { closable: true });
};

const jumpToLogin = (): void => {
  if (urlRegex.test(siteLocation.value)) {
    userStore.setCurrentSit(siteLocation.value);
    window.open(`${siteLocation.value}/core/auth/login/?next=client`);

    return;
  }

  message.error('请输入正确的站点地址', { closable: true });
};
</script>

<style scoped lang="scss"></style>
