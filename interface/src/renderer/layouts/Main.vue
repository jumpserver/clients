<template>
  <div id="main-page">
    <div
      class="fake-title-bar"
    >
      <div class="fake-title-bar__title">
        本地客户端配置工具
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
            <span>远程终端</span>
          </el-menu-item>
          <el-menu-item index="remotePage">
            <el-icon>
              <i class="fa fa-desktop" aria-hidden="true"></i>
            </el-icon>
            <span>远程桌面</span>
          </el-menu-item>
          <el-menu-item index="fileTransferPage">
            <el-icon>
              <i class="fa fa-folder" aria-hidden="true"></i>
            </el-icon>
            <span>文件传输</span>
          </el-menu-item>
          <el-menu-item index="databasesPage">
            <el-icon>
              <i class="fa fa-database" aria-hidden="true"></i>
            </el-icon>
            <span>数据库工具</span>
          </el-menu-item>
          <el-menu-item index="aboutPage">
            <el-icon>
              <i class="fa fa-telegram" aria-hidden="true"></i>
            </el-icon>
            <span>关于我们</span>
          </el-menu-item>
        </el-menu>
      </el-col>
      <el-col
        :span="19"
        class="main-wrapper"
      >
        <section class="app-main">
          <router-view :key="key" :activeItems="activeItems"/>
        </section>
      </el-col>
    </el-row>
  </div>
</template>

<script>
import {ipcRenderer} from 'electron'
import {computed, getCurrentInstance, onBeforeMount, onBeforeUnmount, ref} from "vue";
import {useRouter} from 'vue-router'
import { ElMessage } from 'element-plus'

export default {
  name: 'MainPage',
  components: {},
  setup() {
    const defaultActive = ref('sshPage')
    let activeItems = ref()
    const os = ref('')

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
      if(code === 200){
        config = JSON.parse(arg);
        get_type_config()
      }else {
        ElMessage.error(arg)
      }
    }

    let get_type_config = () => {
      if(config === undefined || config===null){
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
    return {os, defaultActive, key, activeItems, handleSelect}
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
  background: #0000008e;

  &__title {
    margin-left: 167px;
  }
}

.main-content {
  width: 100%;
  overflow-x: hidden;
  overflow-y: auto;
  background: #0000008e;

  .side-bar-menu {
    height: calc(100vh - 25px);
    overflow-x: hidden;
    overflow-y: auto;

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
