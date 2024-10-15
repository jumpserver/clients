import zhLocale from "element-plus/es/locale/lang/zh-cn";
import enLocale from "element-plus/es/locale/lang/en";
import zh from './zh.json'
import en from './en.json'

export default {
  zh: {
    ...zhLocale,
    ...zh
  },
  en: {
    ...enLocale,
    ...en
  }
}
