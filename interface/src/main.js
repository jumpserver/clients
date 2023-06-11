import {createApp} from 'vue'
import App from './App.vue'

import router from './renderer/router'
import ElementUI from 'element-plus'
import 'element-plus/dist/index.css'

const app = createApp(App)

app.use(ElementUI)
app.use(router)

app.mount('#app')
