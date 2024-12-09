<template>
  <n-result
    status="404"
    title="404 资源不存在"
    class="w-full h-full flex flex-col justify-center items-center bg-[#101014]"
  >
    <template #footer>
      <n-button @click="handleGoBack">{{ t('Router.Back') }}</n-button>
    </template>
  </n-result>
</template>

<script setup lang="ts">
import { useI18n } from 'vue-i18n';
import { useRouter } from 'vue-router';

const { t } = useI18n();
const router = useRouter();

const handleGoBack = () => {
  router.go(-1);
};
</script>
