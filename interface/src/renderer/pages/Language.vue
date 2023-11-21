<template>
  <div class="padding">
    <el-form
      label-position="left"
      label-width="70%"
    >
      <el-form-item :label="$t('Language.ChooseLanguage')" class="label">
        <el-select v-model="i18n" @change="handClickI18n">
          <el-option v-for="i in supportLanguages" :key="i" :label="i.title" :value="i.code" />
        </el-select>
      </el-form-item>
      <el-divider/>
    </el-form>
  </div>
</template>

<script>
import {useI18n} from "vue-i18n";
import {onBeforeMount, ref} from "vue";

export default {
  name: "LanguagePage",
  setup(){
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
    })

    let handClickI18n = (value) => {
      t.locale.value = value
      localStorage.setItem('lang', value)
    }
    return {supportLanguages, handClickI18n, i18n}
  }
}
</script>

<style lang="scss">
.label .el-form-item__label{
  color: #eee !important;
}

.padding {
  margin: 50px 60px;;

  .el-select {
    width: 140px;
  }
}
</style>
