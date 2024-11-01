<template>
  <n-modal
    :show="showModal"
    :mask-closable="false"
    :show-icon="false"
    :closable="false"
    preset="dialog"
    class="rounded-[10px]"
  >
    <template #header>
      <n-flex align="center">
        <n-icon size="30" :component="Warning24Regular" color="#4B9E5F" />
        <n-text depth="1">提示</n-text>
      </n-flex>
    </template>

    <template #default> 您还未登录，请点击登录按钮跳转到登录页面进行登录认证 </template>
    <template #action>
      <n-button round size="small" type="primary" @click="jumpToLogin"> 登录 </n-button>
    </template>
  </n-modal>
</template>

<script setup lang="ts">
import { Warning24Regular } from '@vicons/fluent';

withDefaults(
  defineProps<{
    showModal: boolean;
  }>(),
  { showModal: false }
);

const jumpToLogin = () => {
  window.open('https://jumpserver-test.cmdb.cc/core/auth/login/?next=client');
};
</script>

<style scoped lang="scss"></style>
