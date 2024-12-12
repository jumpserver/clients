<template>
  <n-modal
    :show="showModal"
    :show-icon="false"
    :closable="false"
    :mask-closable="false"
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
          :status="inputStatus"
          placeholder="请填写完整的域名或者 ip 站点地址"
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
import { useMessage } from 'naive-ui';
import { readText } from 'clipboard-polyfill';
import { watch, onMounted, ref } from 'vue';
import { useUserStore } from '@renderer/store/module/userStore';

import { Warning24Regular } from '@vicons/fluent';

const props = withDefaults(
  defineProps<{
    showModal: boolean;
  }>(),
  { showModal: false }
);

const emits = defineEmits<{
  (e: 'close-mask'): void;
}>();

const urlRegex =
  /^(https?:\/\/)?((([a-zA-Z0-9-]{1,63}\.)+[a-zA-Z]{2,})|(\d{1,3}\.){3}\d{1,3}|\[?[a-fA-F0-9]{1,4}:([a-fA-F0-9]{1,4}:){1,7}[a-fA-F0-9]{1,4}\]?)$/;

const message = useMessage();
const userStore = useUserStore();

const inputStatus = ref('');
const siteLocation = ref('');

/**
 * @description 不输入站点之前不允许关闭遮罩
 */
const handleMaskClick = (): void => {
  const userInfo = userStore.userInfo;

  if (userInfo && userInfo.length > 0) {
    emits('close-mask');

    return;
  }

  if (siteLocation.value) {
    message.error('请点击登录进行认证');
    return;
  }

  message.error('请输入站点地址', { closable: true });
};

/**
 * @description 登录按钮的回调
 */
const jumpToLogin = () => {
  const isIpAddress = /^(\d{1,3}\.){3}\d{1,3}$/.test(siteLocation.value);

  if (!isIpAddress && !/^https?:\/\//i.test(siteLocation.value)) {
    message.error('请输入带有 http:// 或 https:// 协议的站点地址', { closable: true });
    inputStatus.value = 'error';
    return;
  }

  if (urlRegex.test(siteLocation.value)) {
    userStore.setCurrentSit(siteLocation.value);
    inputStatus.value = 'success';

    window.open(`${siteLocation.value}/core/auth/login/?next=client`);

    return;
  }

  message.error('请输入正确的站点地址', { closable: true });
  inputStatus.value = 'success';
};

/**
 * @description 组织鼠标右键默认行为，改为右键粘贴 ip 内容
 */
const handleContextMenu = async () => {
  try {
    const text = await readText();

    if (text) {
      if (urlRegex.test(text)) {
        siteLocation.value = text;
      } else {
        siteLocation.value = text;
        message.error(`${text} 站点信息不符合规则`, { closable: true });
      }
    }
  } catch (e) {}
};

watch(
  () => props.showModal,
  newValue => {
    if (!newValue) {
      window.removeEventListener('contextmenu', handleContextMenu, false);
    }
  },
  { immediate: true }
);

onMounted(() => {
  // window.addEventListener('contextmenu', handleContextMenu, false);
});
</script>
