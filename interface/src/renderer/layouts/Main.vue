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
          <el-menu-item index="aboutPage">
            <el-icon>
              <i class="fa fa-users" aria-hidden="true"></i>
            </el-icon>
            <span>{{ $t('Router.AboutUs') }}</span>
          </el-menu-item>
        </el-menu>
        <el-icon style="position:absolute;bottom: 0;padding: 10px;color: #eee; font-size: 18px;"
                 @click="handClickI18n">
          <i class="fa fa-language" aria-hidden="true"></i>
        </el-icon>
      </el-col>
      <el-col
          :span="19"
          class="main-wrapper"
      >
        <section class="app-main">
          <router-view :key="key" :activeItems="activeItems" :os="os" />
        </section>
      </el-col>
    </el-row>
  </div>
</template>

<script>
import {ipcRenderer} from 'electron'
import {computed, getCurrentInstance, onBeforeMount, onBeforeUnmount, ref} from "vue";
import {useRouter} from 'vue-router'
import {ElMessage} from 'element-plus'
import { useI18n } from "vue-i18n";

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

    const t = useI18n();
    let handClickI18n = () => {
      t.locale.value = "en"
      localStorage.setItem('lang', "en")
    }
    return {os, defaultActive, key, activeItems, handleSelect, handClickI18n}
  }
}
</script>

<style lang="scss" scoped>

.fake-title-bar {
  -webkit-app-region: drag;
  height: 25px;
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
    height: calc(100vh - 25px);
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


</style>
