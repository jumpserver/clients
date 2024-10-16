<template>
  <div id="main-page">
    <div
      class="fake-title-bar"
    >
      <div class="fake-title-bar__title">
        {{ $t('Common.JumpServerClient') }}
      </div>
    </div>
    <el-row class="main-content">
      <el-col :span="5" class="side-bar-menu">
        <el-menu
          :default-active="defaultActive"
          @select="handleSelect"
        >
          <el-menu-item index="sshPage">
            <el-icon>
              <i class="fa fa-terminal" aria-hidden="true"></i>
            </el-icon>
            <span>{{ $t('Router.Terminal') }}</span>
          </el-menu-item>
          <el-menu-item index="remotePage">
            <el-icon>
              <i class="fa fa-desktop" aria-hidden="true"></i>
            </el-icon>
            <span>{{ $t('Router.RemoteDesktop') }}</span>
          </el-menu-item>
          <el-menu-item index="fileTransferPage">
            <el-icon>
              <i class="fa fa-folder" aria-hidden="true"></i>
            </el-icon>
            <span>{{ $t('Router.FileTransfer') }}</span>
          </el-menu-item>
          <el-menu-item index="databasesPage">
            <el-icon>
              <i class="fa fa-database" aria-hidden="true"></i>
            </el-icon>
            <span>{{ $t('Router.Database') }}</span>
          </el-menu-item>
          <el-menu-item index="languagePage">
            <el-icon>
              <i class="fa fa-language" aria-hidden="true"></i>
            </el-icon>
            <span>{{ $t('Router.Language') }}</span>
          </el-menu-item>
          <el-menu-item index="aboutPage">
            <el-icon>
              <i class="fa fa-users" aria-hidden="true"></i>
            </el-icon>
            <span>{{ $t('Router.AboutUs') }}</span>
          </el-menu-item>
        </el-menu>
      </el-col>
      <el-col
        :span="19"
        class="main-wrapper"
      >
        <section class="app-main">
          <router-view :key="key" :activeItems="activeItems" :os="os"/>
        </section>
      </el-col>
    </el-row>
    <el-dialog
      top="20%"
      v-model="languagesDialogVisible"
      :title="$t('Router.Language')"
      :destroy-on-close="true"
      :close-on-click-modal="false"
      :close-on-press-escape="false"
      :show-close="false"
    >
      <span class="label" >{{ $t('Language.ChooseLanguage') }}</span>
      <el-select v-model="i18n" @change="handClickI18n" style="float:right;">
        <el-option v-for="i in supportLanguages" :key="i" :label="i.title" :value="i.code" />
      </el-select>
      <template #footer>
        <div class="dialog-footer">
          <el-button type="primary" @click="onConfirm">{{ $t('Dialog.Save') }}</el-button>
        </div>
      </template>
    </el-dialog>
  </div>
</template>

<script>
import {ipcRenderer} from 'electron'
import {computed, getCurrentInstance, onBeforeMount, onBeforeUnmount, ref} from "vue";
import {useRouter} from 'vue-router'
import {ElMessage} from 'element-plus'
import {useI18n} from "vue-i18n";

export default {
  name: 'MainPage',
  components: {},
  setup() {
    const defaultActive = ref('sshPage')
    let activeItems = ref()
    const os = ref()

    const {proxy} = getCurrentInstance();
    let key = computed(() => {
      return proxy.$route.path
    })

    onBeforeMount(() => {
      os.value = process.platform
      ipcRenderer.on('config-reply-get', ipcEventHandler)
    })
    onBeforeUnmount(() => {
      ipcRenderer.removeListener('config-reply-get', ipcEventHandler)
    })

    let config
    let ipcEventHandler = (evt, code, arg) => {
      if (code === 200) {
        config = JSON.parse(arg);
        get_type_config()
      } else {
        ElMessage.error(arg)
      }
    }

    let get_type_config = () => {
      if (config === undefined || config === null) {
        return
      }

      let appItems
      if (process.platform === "win32") {
        appItems = config['windows']
      } else if (process.platform === "darwin") {
        appItems = config['macos']
      } else {
        appItems = config['linux']
      }

      switch (defaultActive.value) {
        case 'sshPage':
          activeItems.value = appItems['terminal']
          break
        case 'remotePage':
          activeItems.value = appItems['remotedesktop']
          break
        case 'fileTransferPage':
          activeItems.value = appItems['filetransfer']
          break
        case 'databasesPage':
          activeItems.value = appItems['databases']
          break
      }
    }
    const router = new useRouter()
    let handleSelect = (item) => {
      defaultActive.value = item
      get_type_config()
      router.push({
        name: item
      })
    }
    ipcRenderer.send('config-get')

    const languagesDialogVisible = ref(false)
    const supportLanguages = [
      {
        title: '中文(简体)',
        code: 'zh',
      },
      {
        title: 'English',
        code: 'en',
      }
    ]
    const t = useI18n();
    const i18n = ref()

    onBeforeMount(() => {
      i18n.value = t.locale.value
      let lang = localStorage.getItem('lang')
      if (lang === undefined || lang == null) {
        languagesDialogVisible.value = true
      }
    })

    let handClickI18n = (value) => {
      t.locale.value = value
    }
    let onConfirm = () => {
      localStorage.setItem('lang', t.locale.value)
      languagesDialogVisible.value = false
    }
    return {os, defaultActive, key, activeItems, handleSelect, i18n, languagesDialogVisible, supportLanguages, handClickI18n, onConfirm}
  }
}
</script>

<style lang="scss" scoped>

.fake-title-bar {
  -webkit-app-region: drag;
  height: 30px;
  width: 100%;
  text-align: center;
  color: #eee;
  font-size: 14px;
  line-height: 25px;
  background: #1f1f1f;

  &__title {
    margin-left: 167px;
  }
}

.main-content {
  width: 100%;
  overflow-x: hidden;
  overflow-y: auto;
  background: #1f1f1f;

  .side-bar-menu {
    height: calc(100vh - 30px);
    overflow-x: hidden;
    overflow-y: auto;
    border-right: solid 1px #4c4d4faf;

    .el-menu {
      border-right: none;
      background: transparent;

      &-item {
        color: #eee;

        &:hover {
          color: #fff;
          background: transparent;
        }

        &.is-active {
          color: #409EFF;
        }
      }
    }
  }
}

.main-wrapper {
  height: calc(100vh - 30px);
  overflow-x: hidden;
  overflow-y: auto;
}

*::-webkit-scrollbar {
  width: 6px;
  height: 6px;
}

*::-webkit-scrollbar-thumb {
  border-radius: 4px;
  background: #5f5f5f;
}

*::-webkit-scrollbar-track {
  background-color: transparent;
}

:deep(.el-dialog) {
  .el-dialog__body {
    padding: 10px 20px;
  }
}

.label {
  line-height: 32px;
}
</style>
