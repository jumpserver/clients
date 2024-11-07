<template>
  <n-flex justify="center" align="center" vertical class="bg-[#101014] h-full">
    <n-image :src="src" width="400" height="200" />
    <n-form
      inline
      ref="formRef"
      class="flex-col !items-center mt-[20px]"
      :label-width="80"
      :model="formValue"
      :rules="rules"
    >
      <n-form-item :label="t('Login.Username')" path="user.username">
        <n-input v-model:value="formValue.user.username" :placeholder="t('Login.EnterUsername')" />
      </n-form-item>
      <n-form-item :label="t('Login.Password')" path="user.password">
        <n-input v-model:value="formValue.user.password" :placeholder="t('Login.EnterPassword')" />
      </n-form-item>
      <n-form-item>
        <n-button
          attr-type="button"
          type="primary"
          color="#4C917D"
          text-color="white"
          @click="handleValidateClick"
          class="w-full"
        >
          {{ t('Login.LoginCheck') }}
        </n-button>
      </n-form-item>
    </n-form>
  </n-flex>
</template>

<script setup lang="ts">
import { ref } from 'vue';
import { useI18n } from 'vue-i18n';

const { t } = useI18n();
const rules = {
  user: {
    username: {
      required: true,
      message: t('Login.EnterUsername'),
      trigger: 'blur'
    },
    password: {
      required: true,
      message: t('Login.EnterPassword'),
      trigger: ['input', 'blur']
    }
  }
};
const src = ref('');
const formValue = ref({
  user: {
    username: '',
    password: ''
  }
});

const handleValidateClick = () => {};
const getImage = async () => {
  try {
    const res = await import('@renderer/assets/JumpServer.png');

    if (res) {
      src.value = res.default;
    }
  } catch (e) {
    console.log(e);
  }
};

getImage();
</script>

<style scoped lang="scss">
:deep(.n-form-item) {
  width: 400px !important;
  margin: unset !important;
}
</style>
