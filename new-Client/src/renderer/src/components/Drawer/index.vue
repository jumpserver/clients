<template>
  <n-drawer
    v-model:show="props.active"
    placement="right"
    to="#drawer-target"
    :width="350"
    :show-mask="false"
    :trap-focus="false"
    :block-scroll="false"
    :mask-closable="false"
    class="!rounded-[unset] bg-primary border-l-primary border-l"
  >
    <n-drawer-content footer-style="border: unset;" body-content-style="padding: 15px 5px">
      <template #header>
        <n-flex align="center" justify="space-between">
          <n-text depth="1">默认配置</n-text>

          <n-flex>
            <n-icon
              size="20"
              :component="ArrowBarRight"
              class="icon-hover rounded-[5px]"
              @click="closeDrawer"
            />
          </n-flex>
        </n-flex>
      </template>

      <n-scrollbar>
        <template #default>
          <n-flex class="mx-[15px]">
            <n-card
              v-for="item of currentOption"
              :bordered="false"
              :key="item"
              size="small"
              header-style="font-size: 13px;"
              class="rounded-[10px] !bg-secondary"
            >
              <n-collapse
                :accordion="true"
                :trigger-areas="['extra']"
                :default-expanded-names="enabledItems"
                :expanded-names="expandedNames"
              >
                <n-collapse-item :name="item.display_name" :title="item.display_name">
                  <template #header-extra>
                    <n-switch
                      size="small"
                      v-model:value="item.is_set"
                      @update:value="handleSwitchValueChange(item)"
                    >
                      <template #checked> 已启用</template>
                      <template #unchecked> 未启用</template>
                    </n-switch>
                  </template>
                  <n-form :model="item">
                    <n-form-item label="应用路径:" label-style="font-size: 13px">
                      <n-input-group>
                        <n-input
                          v-model:value="item.path"
                          size="small"
                          :disabled="item.is_internal || platform === 'darwin'"
                        />
                        <n-button
                          type="primary"
                          ghost
                          size="small"
                          @click="openFile"
                          :disabled="item.is_internal || platform === 'darwin'"
                        >
                          <n-icon :component="Folder28Regular" size="14" />
                        </n-button>
                      </n-input-group>
                    </n-form-item>
                    <n-form-item
                      label="应用说明:"
                      label-style="font-size: 13px"
                      label-placement="left"
                    >
                      <n-popover>
                        <template #trigger>
                          <span class="truncate">
                            {{ item.comment }}
                          </span>
                        </template>
                        {{ item.comment }}
                      </n-popover>
                    </n-form-item>
                    <n-form-item
                      label="下载地址:"
                      label-style="font-size: 13px"
                      feedback-style="min-height: unset"
                      label-placement="left"
                    >
                      <n-popover>
                        <template #trigger>
                          <n-a
                            href="#"
                            class="truncate"
                            @click.prevent="copyToClipboard(item.download_url)"
                          >
                            {{ item.download_url }}
                          </n-a>
                        </template>
                        {{ item.download_url }}
                      </n-popover>
                    </n-form-item>
                    <input
                      type="file"
                      name="filename"
                      id="select-exe"
                      style="display: none"
                      @change="changeFile(item)"
                    />
                  </n-form>
                </n-collapse-item>
              </n-collapse>
            </n-card>

            <n-card
              :bordered="false"
              size="small"
              header-style="font-size: 13px;"
              class="rounded-[10px] !bg-secondary"
            >
              <n-collapse :trigger-areas="['main']">
                <n-collapse-item title="高级选项" name="advanced">
                  <n-form>
                    <n-form-item label="字符集:" label-style="font-size: 13px">
                      <n-select v-model:value="charset" size="small" :options="charsetOptions" />
                    </n-form-item>
                    <n-form-item
                      label="字符终端 Backspace As Ctrl + H"
                      label-style="font-size: 13px"
                    >
                      <n-select
                        v-model:value="is_backspace_as_ctrl_h"
                        size="small"
                        :options="boolOptions"
                      />
                    </n-form-item>
                    <n-form-item label="分辨率:" label-style="font-size: 13px">
                      <n-select
                        v-model:value="rdp_resolution"
                        size="small"
                        :options="resolutionsOptions"
                      />
                    </n-form-item>
                  </n-form>
                </n-collapse-item>
              </n-collapse>
            </n-card>
          </n-flex>
        </template>
      </n-scrollbar>
    </n-drawer-content>
  </n-drawer>
</template>

<script setup lang="ts">
import mittBus from '@renderer/eventBus';
import { Folder28Regular } from '@vicons/fluent';
import { ArrowBarRight } from '@vicons/tabler';

import type { Ref } from 'vue';
import { computed, nextTick, onBeforeUnmount, onMounted, ref, watch } from 'vue';
import { useMessage } from 'naive-ui';
import { useRoute } from 'vue-router';
import { useSettingStore } from '@renderer/store/module/settingStore';
import { Conf } from 'electron-conf/renderer';
import {
  boolOptions,
  charsetOptions,
  databaseOptions,
  IClient,
  linuxOptions,
  resolutionsOptions,
  windowsOptions
} from './config/index';

const props = withDefaults(defineProps<{ active: boolean }>(), { active: false });

const route = useRoute();
const message = useMessage();
const settingStore = useSettingStore();
const currentOption: Ref<IClient[] | null> = ref(null);
const platform = ref();
const charset = ref(settingStore.charset);
const is_backspace_as_ctrl_h = ref(settingStore.is_backspace_as_ctrl_h);
const rdp_resolution = ref(settingStore.rdp_resolution);
const conf = new Conf();

watch([charset, is_backspace_as_ctrl_h, rdp_resolution], () => {
  settingStore.charset = charset.value;
  settingStore.is_backspace_as_ctrl_h = is_backspace_as_ctrl_h.value;
  settingStore.rdp_resolution = rdp_resolution.value;
});

watch(
  () => route.name,
  newValue => {
    switch (newValue) {
      case 'Linux':
        currentOption.value = linuxOptions.value;
        break;
      case 'Windows':
        currentOption.value = windowsOptions.value;
        break;
      case 'Database':
        currentOption.value = databaseOptions.value;
        break;
      default:
        currentOption.value = [];
    }
  },
  {
    immediate: true // Execute immediately on component mount
  }
);

const enabledItems = computed(() => {
  // 获取所有启用状态的 item 标签作为展开的名称
  return currentOption.value
    ? currentOption.value.filter(item => item.is_set).map(item => item.display_name)
    : [];
});

const expandedNames = computed(() => {
  const enabledItem = currentOption.value?.find(item => item.is_set);

  return enabledItem ? [enabledItem.display_name] : [];
});

const handleItemChange = async (item: IClient) => {
  const configName = platform.value + '.' + item.type;
  let newList = [];
  currentOption.value?.forEach(option => {
    if (option.type === item.type) {
      newList.push(toRaw(option));
    }
  });
  await conf.set(configName, newList);
};

const openFile = () => {
  window.document.getElementById('select-exe')!.click();
};

const changeFile = (item: IClient) => {
  const exe = window.document.getElementById('select-exe');
  // @ts-ignore
  item.path = exe.files[0].path;
  handleItemChange(item).then(() => {
    message.success('修改成功');
  });
};

const copyToClipboard = (text: string) => {
  navigator.clipboard
    .writeText(text)
    .then(() => message.success('已复制到剪切板'))
    .catch();
};

/**
 * @description 关闭抽屉
 */
const closeDrawer = () => {
  mittBus.emit('createDrawer');
};

/**
 * @deprecated 当只有一个处于启用状态时不允许关闭当前 Switch，只有点击启用其他 Option 时该状态关闭
 *
 * @param item
 */
const handleSwitchValueChange = (item: IClient) => {
  nextTick(() => {
    const enabledOptions = currentOption.value?.filter(option => option.is_set);

    if (enabledOptions && enabledOptions.length === 0) {
      message.warning('请至少启用一个选项');
      item.is_set = true;
      return;
    }

    // 当前的为 true 其他的为 false
    if (enabledOptions && enabledOptions.length > 0) {
      item.is_set = true;

      currentOption.value?.forEach(option => {
        if (option !== item) {
          option.is_set = false;
        }
      });
    }
  });
};

window.electron.ipcRenderer.on('platform-response', (_event, _platform) => {
  platform.value = _platform;
  conf.get(_platform).then(res => {
    if (res) {
      // @ts-ignore
      linuxOptions.value = [...res?.terminal, ...res?.filetransfer];
      // @ts-ignore
      windowsOptions.value = res?.remotedesktop;
      // @ts-ignore
      databaseOptions.value = res?.databases;
    }
  });
});

onMounted(() => {
  window.electron.ipcRenderer.send('get-platform');
});

onBeforeUnmount(() => {});
</script>

<style lang="scss" scoped>
:deep(.n-form-item-feedback-wrapper) {
  min-height: 15px;
}

:deep(.n-collapse-item-arrow) {
  display: none !important;
}
</style>
