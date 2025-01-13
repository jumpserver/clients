<template>
  <n-flex vertical justify="space-between" class="py-4" style="height: calc(100% - 30px)">
    <div>
      <n-menu
        :options="options"
        :collapsed="collapsed"
        :collapsed-width="64"
        :collapsed-icon-size="22"
        v-model:value="selectedKey"
        class="w-full flex flex-col items-center"
      />
      <n-divider class="!my-3" />
    </div>

    <!-- 未登录状态 -->
    <n-flex v-if="userOptions.length === 0" align="center" justify="center">
      <n-button text strong @click="handleAddAccount"> {{ t('Common.UnLogged') }} </n-button>
    </n-flex>

    <n-flex
      v-else
      align="center"
      :justify="collapsed ? 'center' : 'space-between'"
      :style="{ padding: collapsed ? '0' : '0 1rem' }"
      class="w-full !flex-nowrap cursor-pointer transition-all duration-300 ease-in-out"
    >
      <n-flex
        align="center"
        class="!gap-0 w-full"
        :style="{ justifyContent: collapsed ? 'center' : '' }"
      >
        <n-avatar
          v-if="currentUser?.avatar_url"
          round
          size="medium"
          class="cursor-pointer w-8 h-8 transition-all duration-300"
          :src="currentUser?.avatar_url"
        />

        <n-flex
          v-if="!collapsed"
          vertical
          class="ml-3 overflow-hidden transition-all duration-500"
          :style="{
            maxWidth: collapsed ? '0' : '200px',
            opacity: collapsed ? '0' : '1',
            transform: collapsed ? 'translateX(-20px)' : 'translateX(0)',
            gap: '2px'
          }"
        >
          <n-text depth="1" strong class="whitespace-nowrap text-sm">
            {{ currentUser?.username }}
          </n-text>
        </n-flex>
      </n-flex>

      <n-popselect
        v-if="!collapsed"
        show-arrow
        size="small"
        placement="bottom"
        class="rounded-xl"
        :style="{
          width: '16rem',
          marginLeft: '1.5rem'
        }"
        :options="[]"
      >
        <ArrowRightLeft :size="18" @mousedown.prevent />

        <template #header>
          <n-text depth="3" strong> Switch Account </n-text>
        </template>

        <template #empty>
          <n-flex vertical justify="start" class="w-full">
            <template v-for="user of userOptions" :key="user.token">
              <AccountList
                :username="user.label!"
                :user-token="user.value!"
                :user-site="user.currentSite!"
                :user-avator="user.avatar_url!"
                @change-account="handleAccountChange"
              />
            </template>
          </n-flex>
        </template>

        <template #action>
          <n-flex vertical align="center" justify="start" class="w-full">
            <n-button text class="w-full justify-start" @click="handleAddAccount">
              <template #icon>
                <UserRoundPlus />
              </template>
              {{ t('Common.AddAccount') }}
            </n-button>

            <n-button text class="w-full justify-start" @click="handleRemoveAccount">
              <template #icon>
                <LogOut />
              </template>
              {{ t('Common.RemoveAccount') }}
            </n-button>
          </n-flex>
        </template>
      </n-popselect>
    </n-flex>
  </n-flex>
</template>

<script setup lang="ts">
import { useI18n } from 'vue-i18n';
import { storeToRefs } from 'pinia';
import { computed, watch, ref } from 'vue';
import { menuOptions } from './config/index';
import { useDebounceFn } from '@vueuse/core';
import { UserRoundPlus } from 'lucide-vue-next';
import { NAvatar, NText, NFlex } from 'naive-ui';
import { LogOut, ArrowRightLeft } from 'lucide-vue-next';
import { useUserStore } from '@renderer/store/module/userStore';

import mittBus from '@renderer/eventBus';
import AccountList from '../AccountList/index.vue';

import type { IUserInfo } from '@renderer/store/interface';

withDefaults(defineProps<{ collapsed: boolean }>(), {
  collapsed: false
});

const { t } = useI18n();
const options = menuOptions();
const userStore = useUserStore();

const { currentUser: storeCurrentUser } = storeToRefs(userStore);

const currentUser = computed(() => storeCurrentUser?.value);

const userOptions = computed(() => {
  return (
    userStore.userInfo?.map((item: IUserInfo) => {
      return {
        label: item.username,
        value: item.token,
        avatar_url: item.avatar_url,
        display_name: item.display_name,
        currentSite: item.currentSite
      };
    }) || []
  );
});

const selectedKey = ref('linux-page');

/**
 * @description 切换账号的逻辑
 */
const handleAccountChange = (token: string) => {
  if (userStore.userInfo) {
    const user = userStore.userInfo.find((item: IUserInfo) => item.token === token);

    if (user) {
      userStore.setToken(user.token);
      userStore.setCurrentUser({ ...user });
      userStore.setCurrentSit(user.currentSite as string);
    }
  }
};

/**
 * @description 添加账号
 */
const handleAddAccount = () => {
  mittBus.emit('addAccount');
  selectedKey.value = 'linux-page';
};

/**
 * @description 移除账号
 */
const handleRemoveAccount = (token: string) => {
  if (userStore.userInfo) {
    userStore.userInfo = userStore.userInfo.filter(user => user.token !== token);

    // 如果删除的是当前账号，切换到第一个账号
    if (token === userStore.currentUser?.token && userStore.userInfo.length > 0) {
      const firstUser = userStore.userInfo[0];
      userStore.setToken(firstUser.token);
      userStore.setCurrentUser({ ...firstUser });
      userStore.setCurrentSit(firstUser.currentSite as string);
    }
  }

  selectedKey.value = 'linux-page';
};

const debouncedSearch = useDebounceFn(() => {
  mittBus.emit('search', 'reset');
}, 300);

watch(
  () => userStore.currentUser,
  newUser => {
    if (newUser) {
      debouncedSearch();
    }
  }
);
</script>

<style scoped lang="scss">
@use './index.scss';
</style>
