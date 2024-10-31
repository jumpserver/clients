<script setup lang="ts">
import { darkThemeOverrides, lightThemeOverrides } from './overrides';
import { darkTheme, lightTheme, zhCN, enUS } from 'naive-ui';
import { onBeforeUnmount, ref, onMounted } from 'vue';
import { Conf } from 'electron-conf/renderer';
import mittBus from '@renderer/eventBus';

const conf = new Conf();

const iconImage = ref('');
const defaultLang = ref('');
const defaultTheme = ref('');

conf.get('defaultSetting').then(res => {
  if (res) {
    // @ts-ignore
    defaultTheme.value = res?.theme;
    // @ts-ignore
    defaultLang.value = res?.language;

    return;
  }
});

const handleThemeChange = async (theme: string) => {
  const currentSettings = (await conf.get('defaultSetting')) as Record<string, any>;

  switch (theme) {
    case 'light': {
      defaultTheme.value = 'dark';
      break;
    }
    case 'dark': {
      defaultTheme.value = 'light';
      break;
    }
  }

  await conf.set('defaultSetting', {
    ...currentSettings,
    theme: defaultTheme.value
  });
};

const getIconImage = async () => {
  const res = await import('@renderer/assets/Logo.svg');

  iconImage.value = res.default;
};

onMounted(() => {
  getIconImage();
  mittBus.on('changeTheme', handleThemeChange);
});

onBeforeUnmount(() => {
  mittBus.off('changeTheme', handleThemeChange);
});
</script>

<template>
  <n-config-provider
    :locale="defaultLang === 'zhCN' ? zhCN : enUS"
    :theme="defaultTheme === 'dark' ? darkTheme : lightTheme"
    :class="defaultTheme === 'dark' ? 'theme-dark' : 'theme-light'"
    :themeOverrides="defaultTheme === 'dark' ? darkThemeOverrides : lightThemeOverrides"
  >
    <n-modal-provider>
      <n-message-provider placement="top-right">
        <div class="custom-header ele_drag bg-primary border-b-primary border-b">
          <div class="logo">
            <img :src="iconImage" alt="" />
            <span class="title text-primary">JumpServer Client</span>
          </div>
        </div>
        <router-view />
      </n-message-provider>
    </n-modal-provider>
  </n-config-provider>
</template>

<style scoped lang="scss">
.n-config-provider {
  height: 100%;
  width: 100%;
}
</style>
