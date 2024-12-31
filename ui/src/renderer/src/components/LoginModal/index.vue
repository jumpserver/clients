<template>
  <n-modal
    :show="showModal"
    :show-icon="false"
    :mask-closable="false"
    preset="dialog"
    class="rounded-[10px]"
    @close="handleMaskClick"
    @mask-click="handleMaskClick"
  >
    <template #header>
      <n-flex align="center">
        <n-text depth="1">{{ t('Common.Tip') }}</n-text>
      </n-flex>
    </template>

    <template #default>
      <n-flex vertical justify="space-evenly" align="flex-start" class="w-full h-[70px]">
        <n-input
          v-model:value="siteLocation"
          clearable
          :status="inputStatus"
          :placeholder="t('Common.LoginModalPlaceholder')"
          class="rounded-[10px]"
        />
      </n-flex>
    </template>

    <template #action>
      <n-button round :disabled="!siteLocation" size="small" type="primary" @click="jumpToLogin">
        {{ t('Common.SignIn') }}
      </n-button>
    </template>
  </n-modal>
</template>

<script setup lang="ts">
import { useI18n } from 'vue-i18n';
import { useMessage } from 'naive-ui';
import { onMounted, ref, watch } from 'vue';
import { readText } from 'clipboard-polyfill';
import { useUserStore } from '@renderer/store/module/userStore';

const URL_REGEXP =
  /^(https?:\/\/)?(([a-zA-Z0-9-]{1,63}\.)+[a-zA-Z]{2,}|(\d{1,3}\.){3}\d{1,3}|\[?[a-fA-F0-9]{1,4}:([a-fA-F0-9]{1,4}:){1,7}[a-fA-F0-9]{1,4}\]?)(:\d{1,5})?$/;

const props = withDefaults(
  defineProps<{
    showModal: boolean;
  }>(),
  { showModal: false }
);

const emits = defineEmits<{
  (e: 'close-mask'): void;
}>();

const { t } = useI18n();
const message = useMessage();
const userStore = useUserStore();

const inputStatus = ref('');
const siteLocation = ref('');

/**
 * @description 不输入站点之前不允许关闭遮罩
 */
const handleMaskClick = (): void => {
  emits('close-mask');

  // const userInfo = userStore.userInfo;
  // if (userInfo && userInfo.length > 0) {
  //   emits('close-mask');
  //   return;
  // }
  // if (siteLocation.value) {
  //   message.error(`${t('Message.ClickSigInToAuth')}`);
  //   return;
  // }
  // message.error(`${t('Message.EnterSiteAddress')}`, { closable: true });
};

/**
 * @description 登录按钮的回调
 */
const jumpToLogin = () => {
  const hasProtocol = /^https?:\/\//i.test(siteLocation.value);

  if (!hasProtocol) {
    message.error(t('Message.ProtocolRequired'), { closable: true });
    inputStatus.value = 'error';
    return;
  }

  const sameSiteUser = userStore.userInfo.filter(item => item.currentSite === siteLocation.value);

  if (sameSiteUser.length !== 0) {
    message.error(t('Message.EnterDiffSite'), { closable: true });
    inputStatus.value = 'error';
    return;
  }

  if (!URL_REGEXP.test(siteLocation.value)) {
    message.error(t('Message.EnterTheCorrectSite'), { closable: true });
    inputStatus.value = 'error';
    return;
  }

  userStore.setCurrentSit(siteLocation.value);
  inputStatus.value = 'success';
  window.open(`${siteLocation.value}/core/auth/login/?next=client`);
};

/**
 * @description 组织鼠标右键默认行为，改为右键粘贴 ip 内容
 */
const handleContextMenu = async () => {
  try {
    const text = await readText();

    if (text) {
      const hasProtocol = /^https?:\/\//i.test(text);

      if (!hasProtocol) {
        siteLocation.value = text;
        message.error(t('Message.ProtocolRequired'), { closable: true });
        return;
      }

      if (URL_REGEXP.test(text)) {
        siteLocation.value = text;
      } else {
        siteLocation.value = text;
        message.error(`${text} ${t('Message.ErrorSiteInput')}`, { closable: true });
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
  window.addEventListener('contextmenu', handleContextMenu, false);
});
</script>
