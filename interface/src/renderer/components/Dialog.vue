<template>
  <div>
    <el-dialog
      class="base-dialog"
      :modal-append-to-body="true"
      :title="title"
      :top="top"
      :width="width"
      v-bind="$attrs"
    >
      <slot/>
      <template #footer>
        <div class="dialog-footer">
          <el-button v-if="showCancel && showButtons" @click="onCancel">{{ $t('Dialog.Cancel') }}</el-button>
          <el-button v-if="showSave && showButtons" type="success" @click="onSave">{{ $t('Dialog.Save') }}</el-button>
          <el-button v-if="showConfirm && showButtons" :loading="loadingStatus" type="primary" @click="onConfirm">
            {{ $t('Dialog.Save&Default') }}
          </el-button>
        </div>
      </template>
    </el-dialog>
  </div>
</template>

<script>
export default {
  name: 'SettingDialog',
  props: {
    title: {
      type: String,
      default: 'Title'
    },
    top: {
      type: String,
      default: '12vh'
    },
    width: {
      type: String,
      default: '70%'
    },
    showConfirm: {
      type: Boolean,
      default: true
    },
    showCancel: {
      type: Boolean,
      default: true
    },
    showSave: {
      type: Boolean,
      default: true
    },
    showButtons: {
      type: Boolean,
      default: true
    },
    loadingStatus: {
      type: Boolean,
      default: false
    },
    maxWidth: {
      type: String,
      default: '1200px'
    }
  },
  setup() {
  },
  methods: {
    onCancel() {
      this.$emit('cancel')
    },
    onConfirm() {
      this.$emit('confirm')
    },
    onSave() {
      this.$emit('save')
    }
  }
}
</script>

<style lang="scss" scoped>
:deep(.el-dialog) {
  .el-dialog__body {
    padding: 10px 20px;
  }
}
</style>

