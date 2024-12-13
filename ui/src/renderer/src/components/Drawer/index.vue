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
    <n-drawer-content footer-style="border: unset;" body-content-style="padding: 15px 5px 40px">
      <template #header>
        <n-flex align="center" justify="space-between">
          <n-text depth="1">默认配置</n-text>
          <!--          <n-text depth="1" v-else>资产详情</n-text>-->

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
              <n-collapse :trigger-areas="['extra']" :default-expanded-names="enabledItems">
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
                          :disabled="item.is_internal || platform === 'macos'"
                        />
                        <input
                          type="file"
                          name="filename"
                          :id="item.name"
                          style="display: none"
                          @change="changeFile(item)"
                        />
                        <n-button
                          type="primary"
                          ghost
                          size="small"
                          @click="openFile(item)"
                          :disabled="item.is_internal || platform === 'macos'"
                        >
                          <n-icon :component="Folder28Regular" size="14" />
                        </n-button>
                      </n-input-group>
                    </n-form-item>
                    <n-form-item label="协议:" label-style="font-size: 13px">
                      <n-select
                        v-model:value="item.match_first"
                        multiple
                        @update:value="handleItemChange(item)"
                        :options="item.protocol.map(value => ({ label: value, value: value }))"
                      />
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

          <!--          <n-flex class="mx-[15px]">-->
          <!--            <n-card-->
          <!--              :bordered="false"-->
          <!--              size="small"-->
          <!--              header-style="font-size: 13px;"-->
          <!--              class="rounded-[10px] !bg-secondary"-->
          <!--            >-->
          <!--              <n-form size="small" :label-width="80" :model="formValue" :rules="rules">-->
          <!--                <n-form-item label="名称" path="user.name">-->
          <!--                  <n-input v-model:value="formValue.user.name" placeholder="输入姓名" disabled />-->
          <!--                </n-form-item>-->
          <!--                <n-form-item label="年龄" path="user.age">-->
          <!--                  <n-input v-model:value="formValue.user.age" placeholder="输入年龄" disabled />-->
          <!--                </n-form-item>-->
          <!--                <n-form-item label="电话号码" path="phone">-->
          <!--                  <n-input v-model:value="formValue.phone" placeholder="电话号码" disabled />-->
          <!--                </n-form-item>-->
          <!--              </n-form>-->
          <!--            </n-card>-->
          <!--          </n-flex>-->
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
import { computed, nextTick, onMounted, ref, watch } from 'vue';
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
import type { IItemDetail } from '@renderer/components/MainSection/interface';

const props = withDefaults(defineProps<{ active: boolean; drawerDetailMessage?: IItemDetail }>(), {
  active: false,
  drawerDetailMessage: () => ({}) as IItemDetail
});

const route = useRoute();
const message = useMessage();
const settingStore = useSettingStore();

const charset = ref(settingStore.charset);
const rdp_resolution = ref(settingStore.rdp_resolution);
const is_backspace_as_ctrl_h = ref(settingStore.is_backspace_as_ctrl_h);

const currentOption: Ref<IClient[] | null> = ref(null);

const platform = ref('');

const conf = new Conf();
const rules = {
  user: {
    name: {
      required: true,
      message: '请输入姓名',
      trigger: 'blur'
    },
    age: {
      required: true,
      message: '请输入年龄',
      trigger: ['input', 'blur']
    }
  },
  phone: {
    required: true,
    message: '请输入电话号码',
    trigger: ['input']
  }
};
const formValue = ref({
  user: {
    name: '',
    age: ''
  },
  phone: ''
});

watch([charset, is_backspace_as_ctrl_h, rdp_resolution], () => {
  settingStore.charset = charset.value;
  settingStore.is_backspace_as_ctrl_h = is_backspace_as_ctrl_h.value;
  settingStore.rdp_resolution = rdp_resolution.value;
});

const updateCurrentOptions = (newValue: string | null) => {
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
};

watch(
  () => route.name,
  newValue => {
    if (newValue && typeof newValue === 'string') {
      updateCurrentOptions(newValue);
    }
  },
  { immediate: true }
);

const enabledItems = computed(() => {
  // 获取所有启用状态的 item 标签作为展开的名称
  return currentOption.value
    ? currentOption.value.filter(item => item.is_set).map(item => item.display_name)
    : [];
});

const handleItemChange = async (item: IClient) => {
  const configName = `${platform.value}.${item.type}`;
  const newList =
    currentOption.value
      ?.filter(option => option.type === item.type)
      .map(option => {
        if (option.name !== item.name) {
          // 过滤掉 `item.match_first` 中的匹配项
          option.match_first = option.match_first.filter(i => !item.match_first.includes(i));
        }
        return toRaw(option);
      }) || [];

  await conf.set(configName, newList);
};

const openFile = (item: IClient) => {
  window.document.getElementById(item.name)!.click();
};

const changeFile = (item: IClient) => {
  const fileInput = window.document.getElementById(item.name) as HTMLInputElement;
  item.path = fileInput?.files?.[0]?.path || '';
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
    handleItemChange(item).then(() => {
      message.success('修改成功');
    });
  });
};

const initPlatformData = async () => {
  const platformData = await conf.get(platform.value);
  if (platformData) {
    linuxOptions.value = [...platformData.terminal, ...platformData.filetransfer];
    windowsOptions.value = platformData.remotedesktop;
    databaseOptions.value = platformData.databases;
  }
};

onMounted(() => {
  window.electron.ipcRenderer.send('get-platform');
  window.electron.ipcRenderer.on('platform-response', (_event, _platform) => {
    console.log(_platform);
    platform.value = _platform;
    initPlatformData();
  });
});
</script>

<style lang="scss" scoped>
:deep(.n-form-item-feedback-wrapper) {
  min-height: 15px;
}

:deep(.n-collapse-item-arrow) {
  display: none !important;
}
</style>
