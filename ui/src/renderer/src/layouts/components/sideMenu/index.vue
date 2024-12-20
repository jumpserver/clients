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
      <n-button text strong> {{ t('Common.UnLogged') }} </n-button>
    </n-flex>

    <n-popselect
      v-else
      show-arrow
      size="small"
      placement="bottom"
      trigger="click"
      class="w-120 rounded-xl"
      :style="{
        width: '16rem'
      }"
      :options="[]"
      :content-style="{ width: '300px', minWidth: '300px' }"
      v-model:value="value"
    >
      <n-flex
        align="center"
        :justify="collapsed ? 'center' : 'space-between'"
        :style="{ padding: collapsed ? '0' : '0 1rem' }"
        class="w-full cursor-pointer transition-all duration-300 ease-in-out"
      >
        <n-flex
          align="center"
          class="!gap-0"
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
            <n-text depth="1" strong class="whitespace-nowrap">
              {{ currentUser?.username }}
            </n-text>

            <n-popover>
              <template #trigger>
                <n-text
                  depth="2"
                  type="success"
                  style="font-size: 0.85rem"
                  class="cursor-pointer whitespace-nowrap"
                >
                  {{ currentUser?.display_name?.[0] ?? '' }}
                </n-text>
              </template>

              <template #default>
                <span v-for="item of currentUser?.display_name" :key="item">
                  {{ item }}
                </span>
              </template>
            </n-popover>
          </n-flex>
        </n-flex>
      </n-flex>

      <template #header>
        <n-flex align="center">
          <n-avatar
            round
            size="medium"
            class="cursor-pointer w-8 h-8 transition-all duration-300"
            :src="currentUser ? (currentUser.avatar_url as string) : ''"
          />

          <n-flex class="!flex-col">
            <n-text> {{ currentUser?.username }} </n-text>

            <n-tag :bordered="false" size="small" type="info" class="cursor-pointer">
              {{ t('Common.DataSource') }}： 'currentSite'
            </n-tag>
          </n-flex>
        </n-flex>
      </template>

      <template #empty>
        <n-popselect
          scrollable
          show-arrow
          size="small"
          trigger="click"
          placement="right-start"
          v-model:value="value"
          :style="{
            width: '12rem'
          }"
          :content-style="{
            width: '100%'
          }"
          :render-label="accountRenderLabel"
          :options="accountOptions"
        >
          <n-button text class="w-full"> 账号列表 </n-button>
        </n-popselect>
      </template>

      <template #action>
        <n-button text class="w-full" @click="handleAddAccount">
          {{ t('Common.AddAccount') }}
        </n-button>
      </template>
    </n-popselect>
  </n-flex>
</template>

<script setup lang="ts">
import { useI18n } from 'vue-i18n';
import { storeToRefs } from 'pinia';
import { menuOptions } from './config';
import { useDebounceFn } from '@vueuse/core';
import { computed, watch, h, nextTick, ref } from 'vue';
import { useUserStore } from '@renderer/store/module/userStore';
import { NAvatar, NText, NPopover, NTag, NEllipsis, NFlex } from 'naive-ui';

import { accountRenderLabel } from '@renderer/utils/render';

import mittBus from '@renderer/eventBus';

import type { SelectOption, SelectRenderLabel } from 'naive-ui';
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
    // @ts-ignore
    userStore?.userInfo.map((item: IUserInfo) => {
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

const value = ref('');
const selectedKey = ref('linux-page');

const accountOptions = [
  {
    label: 'Drive My Car',
    value: 'Drive My Car'
  },
  {
    label: 'Norwegian Wood',
    value: 'Norwegian Wood'
  }
];

console.log(currentUser.value);

/**
 * @description 切换账号的逻辑
 */
const handleAccountChange = (value: string, _option: SelectOption) => {
  if (userStore.userInfo) {
    const user = userStore.userInfo.find((item: IUserInfo) => item.token === value);

    if (user) {
      userStore.setToken(user.token);
      userStore.setCurrentUser({ ...user });
      userStore.setCurrentSit(user.currentSite as string);
    }
  }
};

/**
 * @description 自定义渲染内容
 * @param option
 */
const renderLabel: SelectRenderLabel = option => {
  return h(
    'div',
    {
      style: {
        display: 'flex',
        alignItems: 'center',
        width: '100%'
      }
    },
    [
      h(NAvatar, {
        src: option.avatar_url as string,
        round: true,
        size: 'medium',
        style: {
          cursor: 'pointer',
          flexShrink: 0
        }
      }),
      h(
        'div',
        {
          style: {
            marginLeft: '12px',
            padding: '4px 0',
            flex: 1,
            minWidth: 0
          }
        },
        [
          h(
            'div',
            {
              style: {
                marginBottom: '4px'
              }
            },
            option.label as string
          ),
          h(
            NTag,
            {
              bordered: false,
              size: 'small',
              type: 'info',
              style: {
                maxWidth: '100%',
                cursor: 'pointer'
              }
            },
            {
              default: () =>
                h(
                  NEllipsis,
                  {
                    style: {
                      maxWidth: '200px'
                    },
                    tooltip: {
                      placement: 'top',
                      showArrow: true
                    }
                  },
                  {
                    default: () => `${t('Common.DataSource')}：${option.currentSite}`
                  }
                )
            }
          )
        ]
      )
    ]
  );
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
const handleRemoveAccount = () => {
  mittBus.emit('removeAccount');
  selectedKey.value = 'linux-page';

  nextTick(() => {
    if (userStore.userInfo && userStore.userInfo.length > 0) {
      const firstUser = userStore.userInfo[0];
      userStore.setToken(firstUser.token);
      userStore.setCurrentUser({ ...firstUser });
      userStore.setCurrentSit(firstUser.currentSite as string);
    }
  });
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
