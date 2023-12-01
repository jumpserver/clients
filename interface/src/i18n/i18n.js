import { createI18n } from 'vue-i18n'
import messages from './langs'


const browserLang = navigator.systemLanguage || navigator.language
let lang = browserLang || 'zh'
lang = localStorage.getItem('lang') || lang.slice(0, 2)
const i18n = createI18n({
  locale: lang,
  legacy: false,
  fallbackLocale: 'en',
  silentFallbackWarn: true,
  silentTranslationWarn: true,
  messages
})

export default i18n
