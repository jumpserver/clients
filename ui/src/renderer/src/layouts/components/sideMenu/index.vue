<template>
  <n-flex vertical justify="space-between" class="py-[15px]" style="height: calc(100% - 30px)">
    <n-flex>
      <n-menu
        :options="options"
        v-model:value="selectedKey"
        class="w-full flex flex-col items-center"
      />
      <n-divider class="!my-[10px]" />
    </n-flex>

    <n-flex align="center" justify="center" class="!flex-nowrap">
      <template v-if="userOptions.length === 0">
        <n-button text strong class="flex w-full h-8" @click="handleOpenLogoModal">
          {{ t('Common.UnLogged') }}
        </n-button>
      </template>
      <template v-else>
        <n-avatar
          v-if="currentUser?.avatar_url"
          round
          size="medium"
          class="cursor-pointer"
          :src="currentUser?.avatar_url"
        />

        <div v-if="userOptions.length > 0" class="flex flex-col w-[60%]">
          <div class="flex w-full">
            <n-text depth="1" strong class="!inline-flex !items-center justify-between w-full">
              {{ currentUser?.username }}

              <n-popselect
                trigger="click"
                placement="right-end"
                class="w-80 rounded-xl py-1"
                :content-style="{
                  width: '100%'
                }"
                :options="userOptions"
                :render-label="renderLabel"
                v-model:value="currentUser!.token"
                @update:value="handleAccountChange"
              >
                <n-popover>
                  <template #trigger>
                    <n-icon
                      size="14"
                      :component="ArrowsHorizontal"
                      class="ml-[10px] cursor-pointer icon-hover"
                    />
                  </template>
                  {{ t('Common.SwitchAccount') }}
                </n-popover>
                <template #action>
                  <n-button text class="w-1/2" @click="handleAddAccount">
                    {{ t('Common.AddAccount') }}
                  </n-button>
                  <n-button text class="w-1/2" @click="handleRemoveAccount">
                    {{ t('Common.RemoveAccount') }}
                  </n-button>
                </template>
              </n-popselect>
            </n-text>
          </div>

          <div style="font-size: 12px">
            <n-popover>
              <template #trigger>
                <!-- 默认只展示第一个 -->
                <n-text depth="2">
                  {{ currentUser?.display_name?.[0] ?? '' }}
                </n-text>
              </template>

              <template #default>
                <span v-for="item of currentUser?.display_name" :key="item">
                  {{ item }}
                </span>
              </template>
            </n-popover>
          </div>
        </div>
      </template>
    </n-flex>
  </n-flex>
</template>

<script setup lang="ts">
import { NAvatar, NText, NPopover, NTag, NEllipsis } from 'naive-ui';
import { ArrowsHorizontal } from '@vicons/carbon';
import { computed, watch } from 'vue';
import { useDebounceFn } from '@vueuse/core';

import mittBus from '@renderer/eventBus';

import { useUserStore } from '@renderer/store/module/userStore';
import { menuOptions } from './config';
import { h, nextTick, ref } from 'vue';
import { storeToRefs } from 'pinia';
import { useI18n } from 'vue-i18n';

import type { SelectOption, SelectRenderLabel } from 'naive-ui';
import type { IUserInfo } from '@renderer/store/interface';

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

const selectedKey = ref('linux-page');

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

const handleOpenLogoModal = () => {
  mittBus.emit('removeAccount');
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
