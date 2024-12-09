<template>
  <n-modal
    :show="showModal"
    :mask-closable="false"
    :show-icon="false"
    preset="dialog"
    class="rounded-[10px]"
    @close="handleCloseClick"
  >
    <template #header>
      <n-flex align="center">
        <n-text depth="1">连接</n-text>
        <n-text depth="1">{{ permedData.name }}</n-text>
      </n-flex>
    </template>

    <template #default>
      <n-flex vertical align="flex-start" class="w-full min-h-[120px]">
        <n-tabs v-model:value="connectData.protocol">
          <n-tab v-for="tab in protocols" :key="tab.name" :name="tab.name">
            {{ tab.name.toUpperCase() }}
          </n-tab>
        </n-tabs>
        <n-form size="small" class="w-full">
          <n-form-item label="选择账号">
            <n-select
              class="rounded-[10px]"
              size="small"
              label-field="name"
              key-field="username"
              placeholder="请选择账号"
              v-model:value="accountSelected"
              :options="accounts"
              @update:value="handleItemChange"
            />
          </n-form-item>

          <n-form-item v-if="showManualUsernameInput" label="用户名">
            <n-input v-model:value="connectData.input_username" placeholder="请输入用户名" />
          </n-form-item>
          <n-form-item v-if="showManualPasswdInput" label="密码">
            <n-input
              v-model:value="connectData.input_secret"
              type="password"
              show-password-on="mousedown"
              placeholder="请输入密码"
            />
          </n-form-item>
        </n-form>
      </n-flex>
    </template>
    <template #action>
      <n-button round size="small" type="primary" @click="connectAsset"> 连接</n-button>
    </template>
  </n-modal>
</template>

<script setup lang="ts">
import { computed, ref, watch } from 'vue';
import { IConnectData } from '@renderer/store/interface';
import { useUserStore } from '@renderer/store/module/userStore';
import { storeToRefs } from 'pinia';

const emit = defineEmits(['CloseClick', 'ConnectAsset']);

const props = withDefaults(
  defineProps<{
    showModal: boolean;
    permedData: object;
  }>(),
  {
    showModal: false,
    permedData: {
      permed_protocols: [],
      permed_accounts: []
    }
  }
);
const showManualUsernameInput = ref(false);
const showManualPasswdInput = ref(false);
const accountSelected = ref(null);
const connectData = ref<IConnectData>({});

const userStore = useUserStore();
const { currentUser: storeCurrentUser } = storeToRefs(userStore);
const currentUser = computed(() => storeCurrentUser?.value);

const protocols = computed(() => {
  if (props.permedData.permed_protocols?.length) {
    const protocol_enabled = props.permedData.permed_protocols.filter(item => item.public);
    if (protocol_enabled.length > 0) {
      connectData.value.protocol = protocol_enabled[0].name;
    }
    return protocol_enabled;
  }
  return [];
});

const accounts = computed(() => {
  if (props.permedData.permed_accounts?.length) {
    return props.permedData.permed_accounts.filter(item => item.alias !== '@ANON');
  }
  return [];
});

watch(
  () => props.showModal,
  () => {
    accountSelected.value = null;
    connectData.value = {
      protocol: protocols.value[0]?.name || '',
      asset: '',
      account: '',
      input_username: '',
      input_secret: ''
    };
  }
);

const handleCloseClick = (): void => {
  emit('CloseClick');
};

const handleItemChange = (_, option): void => {
  accountSelected.value =
    option.name +
    (option.alias === option.username || option.alias.startsWith('@')
      ? ''
      : '(' + option.username + ')');
  connectData.value.input_username = '';
  connectData.value.input_secret = '';
  connectData.value.asset = props.permedData.id;
  connectData.value.account = option.username;
  showManualPasswdInput.value = !option.has_secret;
  if (option.username === '@INPUT') {
    showManualUsernameInput.value = true;
  } else if (option.username === '@USER') {
    connectData.value.input_username = currentUser.value!.username;
  } else {
    connectData.value.account = option.name;
    showManualUsernameInput.value = false;
  }
};
const connectAsset = (): void => {
  emit('ConnectAsset', connectData.value);
};
</script>

<style lang="scss" scoped>
:deep(.n-form-item-feedback-wrapper) {
  min-height: 10px;
}
</style>
