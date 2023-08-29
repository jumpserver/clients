<template>
  <ListTable :list-data="activeItems" @appItemClick="appItemClick"/>
  <SettingDialog
      v-model="dialogVisible"
      :destroy-on-close="true"
      :title="title"
      :showConfirm="false"
      v-bind="$attrs"
      @cancel="onCancelItem"
      @save="onSaveItem"
  >
    <el-form ref="item-form" :model="selectItem" label-position="right" label-width="90px">
      <el-form-item label="应用说明" prop="comment">
        <el-input v-model="selectItem.comment" readonly rows="3" resize="none" type="textarea"/>
      </el-form-item>
      <el-form-item label="下载地址" prop="download_url">
        <el-input v-model="selectItem.download_url" readonly :autosize="{maxRows: 2}" resize="none" type="textarea"/>
      </el-form-item>
      <el-form-item label="优先匹配" prop="match_first" :rules="[{required: true, message: '协议不能为空', trigger: 'blur'}]">
        <el-select v-model="selectItem.match_first" multiple placeholder="请选择优先匹配的数据库协议" style="width: 100%">
          <el-option v-for="i in selectItem.protocol" :key="i" :label="i" :value="i" />
        </el-select>
      </el-form-item>
      <el-form-item label="应用路径" prop="path" :rules="[{required: true, message: '路径不能为空', trigger: 'blur'}]">
        <el-input
          v-model="selectItem.path"
          placeholder="请选择数据库工具启动程序路径"
          clearable
          :readonly="selectItem.is_internal || os === 'darwin'"
        >
          <template #append>
            <el-button :disabled="selectItem.is_internal || os === 'darwin'" @click="openFile">
              <el-icon>
                <FolderOpened/>
              </el-icon>
            </el-button>
          </template>
        </el-input>
        <el-text size="small">本地客户端连 Oracle 数据库需要使用 21.0 及以上版本 OCI</el-text>
        <input
            type="file"
            name="filename"
            id="select-exe"
            style="display: none"
            @change="changeFile"
        />
      </el-form-item>
    </el-form>
  </SettingDialog>
</template>

<script>
import {ipcRenderer} from 'electron'
import ListTable from "@/components/ListTable"
import SettingDialog from "@/components/Dialog"

export default {
  name: "sshPage",
  components: {
    ListTable,
    SettingDialog
  },
  props: {
    activeItems: {
      type: Array,
      default: () => []
    },
    os: {
      type: String,
      default: ''
    },
  },
  data() {
    return {
      selectItem: {
        protocol: [],
        display_name: '',
        comment: '',
        download_url: '',
        path: '',
        match_first: [],
        is_internal: false,
        is_default: false,
      },
      title: '',
      dialogVisible: false,
    }
  },
  methods: {
    appItemClick(item) {
      this.dialogVisible = true
      this.selectItem = Object.assign({}, item);
      this.title = this.selectItem['display_name']
    },
    openFile() {
      document.getElementById("select-exe").click();
    },
    changeFile() {
      const exe = document.getElementById("select-exe");
      if (this.os === 'darwin') {
        this.selectItem.path = '/Applications/' + exe.files[0].name.replace('.zip', '');
      } else {
        this.selectItem.path = exe.files[0].path;
      }
    },
    onCancelItem() {
      this.clearSelectItem()
    },
    onSaveItem() {
      // this.$refs['item-form'].validate((valid) => {
      //   if (valid) {
      //   } else {
      //     return false;
      //   }
      // });
      this.selectItem.is_set = this.selectItem.path !== ''
      this.writeSelectItem()
      this.clearSelectItem()
    },
    writeSelectItem() {
      ipcRenderer.send('config-set', this.$route.name, JSON.stringify(this.selectItem))
    },
    clearSelectItem() {
      this.dialogVisible = false
      this.selectItem = {
        protocol: [],
        display_name: '',
        comment: '',
        download_url: '',
        path: '',
        match_first: [],
        is_internal: false,
        is_default: false,
      }
    }
  }
}
</script>

<style lang="scss" scoped>
.el-col {
  padding-left: 6px;
  padding-right: 6px;
  margin-bottom: 6px;
  margin-top: 6px;
}

.app-card {
  background: rgba(130, 130, 130, 0.2);
  border: 1px solid transparent;
  box-shadow: 0 2px 4px rgb(0 0 0 / 12%), 0 0 6px rgb(0 0 0 / 4%);


  .el-card__body {
    padding: 2px 0 !important;
  }

  .app-info {
    padding: 5px 3px;
    height: 100%;
    width: 100%;
    display: flex;

    .image {
      width: 45px;
      height: 45px;
    }

    .info {
      margin-left: 5px;
      color: #eee;
      display: inline-block;
      width: 90%;

      .comment {
        margin-top: 5px;
        overflow: hidden;
        font-size: 14px;
        text-overflow: ellipsis;
        min-height: 100px;
        line-height: 24px;
      }

      .is-setted {
        color: #5cb87a;
      }

      .not-setted {
        color: #e6a23c;
      }
    }
  }
}
</style>
