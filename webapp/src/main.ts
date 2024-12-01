import './global.scss'
// import '@fortawesome/fontawesome-free/scss/fontawesome.scss'  // not working
import '@fortawesome/fontawesome-free/css/all.css'

import { mount } from 'svelte'
import App from './App.svelte'

const app = mount(App, {
    target: document.getElementById('app')!,
})

export default app
