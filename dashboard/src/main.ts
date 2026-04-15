import { createApp } from 'vue'
import { createPinia } from 'pinia'
import App from './App.vue'
import router from './router'
import i18n from './i18n'
import './theme.css'
import './style.css'

// Naive UI: per-component auto-import via unplugin-vue-components
// CSS is injected by components on demand, no global CSS import needed

const app = createApp(App)

app.use(createPinia())
app.use(router)
app.use(i18n)

app.mount('#app')
