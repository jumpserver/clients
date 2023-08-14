<template>
  <el-row>
    <el-col v-for="(d, index) in listData" :key="index" :span="12">
      <el-card shadow="hover" class="app-card" :body-style="{ padding: '0px' }" @click="handClickItem(d)">
        <div class="app-info">
          <img alt="" :src="get_logo_src(d.name)" class="image">
          <div class="info">
            <div class="title">{{ d.display_name }}</div>
            <div class="comment">{{ d.comment }}</div>
            <div>
              <span :class="d.is_set? 'is-setted':'not-setted'">
                {{ is_set_string(d.is_set) }}
              </span>
              <el-divider v-show="d.is_default" direction="vertical"/>
              <span v-show="d.is_default" class="is-setted">默认应用</span>
              <el-divider v-show="d.match_first.length !== 0" direction="vertical"/>
              <el-text v-show="d.match_first.length !== 0" size="small" class="match_first">{{ join_2_string(d.match_first) }}</el-text>
            </div>
          </div>
        </div>
      </el-card>
    </el-col>
  </el-row>
</template>

<script>
export default {
  name: "ListTable",
  components: {},
  props: {
    listData: {
      type: Array,
      default: () => []
    },
  },
  setup() {
  },
  methods: {
    is_set_string(b) {
      if (b) {
        return "已配置"
      } else {
        return "未配置"
      }
    },
    join_2_string(b) {
      if (b) {
        return b.join(',')
      }
      return ""
    },
    get_logo_src(name) {
      return require(`@/assets/${name}.png`)
    },
    handClickItem(item) {
      this.$emit('appItemClick', item)
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
    padding: 10px 10px;
    height: 100%;
    width: 100%;
    display: flex;
    cursor: pointer;

    .image {
      width: 45px;
      height: 45px;
    }

    .info {
      margin-left: 10px;
      color: #eee;
      display: inline-block;
      width: calc(100% - 70px);

      .title {
        font-size: 16px;
      }

      .comment {
        color: #aaa;
        margin-top: 5px;
        padding-bottom: 5px;
        overflow: hidden;
        font-size: 12px;
        text-overflow: ellipsis;
        word-break: break-all;
        height: 90px;
        line-height: 24px;
      }

      .is-setted {
        font-size: 12px;
        color: #5cb87a;
      }

      .not-setted {
        font-size: 12px;
        color: #e6a23c;
      }

      .match_first {
        color: #5cb87a;
      }
    }
  }
}
</style>
