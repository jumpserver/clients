<template>
  <n-flex vertical justify="space-between" class="py-[15px]" style="height: calc(100% - 30px)">
    <n-flex>
      <n-menu
        :options="menuOptions"
        default-value="linux-page"
        class="w-full flex flex-col items-center"
      />
      <n-divider class="!my-[10px]" />
      <n-menu :options="staticOption" class="w-full flex flex-col items-center" />
    </n-flex>

    <n-flex align="center" justify="center" class="!flex-nowrap">
      <n-avatar round size="medium" class="cursor-pointer ml-[-15px]" :src="userInfo?.avatar_url">
        <n-icon>
          <UserAstronaut />
        </n-icon>
      </n-avatar>
      <div>
        <div>
          <n-text depth="2">{{ userInfo?.username || 'xxx' }}</n-text>
        </div>
        <div style="font-size: 12px">
          <n-text depth="3">{{ userInfo?.display_name[0] || 'xxx' }}</n-text>
        </div>
      </div>
    </n-flex>
  </n-flex>
</template>

<script setup lang="ts">
import { UserAstronaut } from '@vicons/fa';
import { menuOptions } from './config';

import { useUserStore } from '@renderer/store/module/userStore';
import { useRouter } from 'vue-router';
import { onMounted } from 'vue';
import { storeToRefs } from 'pinia';

const router = useRouter();
const userStore = useUserStore();

const { userInfo } = storeToRefs(userStore);

// const username = ref('');
// const avatarUrl = ref('');
// const displayName = ref('');

const staticOption = [];

onMounted(() => {
  router.push({ name: 'Linux' });
});
</script>

<style scoped lang="scss">
@use './index.scss';
</style>
